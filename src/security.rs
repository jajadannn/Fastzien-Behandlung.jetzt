use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::{header::{self, HeaderName, HeaderValue}, Method},
    middleware::{Logger, Next},
    web, Error, HttpRequest, HttpResponse,
};

use crate::config::Config;
use std::{collections::HashMap, net::IpAddr, sync::Mutex, time::{Duration, Instant}};

#[derive(Default)]
pub struct AuthLimiter(Mutex<HashMap<IpAddr, (Instant, u32)>>);

impl AuthLimiter {
    fn allow(&self, address: IpAddr, now: Instant) -> bool {
        let mut clients = self.0.lock().unwrap_or_else(|e| e.into_inner());
        clients.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        // Bound both memory use and aggregate expensive password work. Never
        // trust arbitrary forwarded IP headers to identify a client.
        if clients.len() >= 4096 || clients.values().map(|(_, count)| *count).sum::<u32>() >= 120 {
            return false;
        }
        let (_, count) = clients.entry(address).or_insert((now, 0));
        if *count >= 20 { return false; }
        *count += 1;
        true
    }
}

// Log route patterns, never URLs: reset, verification, booking and calendar URLs
// contain bearer secrets. Referer and query strings may contain them as well.
pub fn access_logger() -> Logger {
    Logger::new("%{route}xo %s %b %Dms").custom_response_replace("route", |res| {
        format!("{} {}", res.request().method(),
            res.request().match_pattern().unwrap_or_else(|| "<unmatched>".into()))
    })
}

fn same_origin(value: &str, expected: &str) -> bool {
    match (reqwest::Url::parse(value), reqwest::Url::parse(expected)) {
        (Ok(source), Ok(target)) => source.origin() == target.origin()
            && matches!(source.scheme(), "http" | "https")
            && source.username().is_empty() && source.password().is_none(),
        _ => false,
    }
}

fn csrf_allowed(req: &HttpRequest, expected_origin: &str) -> bool {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return true;
    }
    // Fetch metadata is a browser-controlled header. A sibling subdomain is
    // not trusted merely because browsers classify it as same-site.
    if req.headers().get("sec-fetch-site").is_some_and(|v| v == "cross-site" || v == "same-site") {
        return false;
    }
    if let Some(origin) = req.headers().get(header::ORIGIN) {
        // With no-referrer, browsers can serialize a native form's origin as
        // opaque. Fetch Metadata still proves same-origin navigation. A
        // sandboxed/cross-site opaque form has a different site classification.
        if origin == "null" && req.headers().get("sec-fetch-site").is_some_and(|v| v == "same-origin") {
            return true;
        }
        return origin.to_str().is_ok_and(|origin| same_origin(origin, expected_origin));
    }
    if let Some(referer) = req.headers().get(header::REFERER) {
        return referer.to_str().is_ok_and(|referer| same_origin(referer, expected_origin));
    }
    if req.headers().get("sec-fetch-site").is_some_and(|v| v == "same-origin") {
        return true;
    }
    // Non-browser API clients must explicitly supply a bearer credential;
    // ambient cookie credentials never bypass the origin check.
    req.cookie("auth_token").is_none()
        && req.headers().get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok()).is_some_and(|v| v.starts_with("Bearer "))
}

pub async fn protect<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<EitherBody<B>>, Error> {
    let config = req.app_data::<web::Data<Config>>();
    let allowed = config.is_some_and(|c| csrf_allowed(req.request(), &c.base_url));
    let secure = config.is_some_and(|c| reqwest::Url::parse(&c.base_url).is_ok_and(|u| u.scheme() == "https"));
    let is_static = req.path().starts_with("/static/") || req.path().starts_with("/fonts/");
    let private_page = ["/portal", "/admin", "/login", "/register", "/reset-password", "/verify-email", "/logout", "/widerruf"].iter().any(|p| req.path().starts_with(p)) || req.path().starts_with("/buchen/bestaetigen/");
    let limited_endpoint = req.method() == Method::POST && (req.path().starts_with("/api/auth/") || req.path().starts_with("/widerruf/") || req.path().starts_with("/api/guest-booking/"));
    let rate_allowed = !limited_endpoint || req.app_data::<web::Data<AuthLimiter>>()
        .zip(req.peer_addr()).is_some_and(|(limiter, peer)| limiter.allow(peer.ip(), Instant::now()));
    let mut response = if allowed && !rate_allowed {
        req.into_response(HttpResponse::TooManyRequests().insert_header((header::RETRY_AFTER, "60"))
            .json(serde_json::json!({"error": "Zu viele Anmeldeanfragen. Bitte versuche es in einer Minute erneut."})))
            .map_into_right_body()
    } else if allowed {
        next.call(req).await?.map_into_left_body()
    } else {
        req.into_response(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Die Herkunft der Anfrage konnte nicht bestätigt werden. Bitte lade die Seite erneut."
        }))).map_into_right_body()
    };
    let headers = response.headers_mut();
    if private_page { headers.insert(HeaderName::from_static("x-robots-tag"), HeaderValue::from_static("noindex, nofollow")); }
    headers.insert(HeaderName::from_static("permissions-policy"), HeaderValue::from_static("camera=(), microphone=(), geolocation=()"));
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "no-referrer"),
        ("content-security-policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; object-src 'none'; form-action 'self'"),
    ] {
        headers.insert(HeaderName::from_static(name), HeaderValue::from_static(value));
    }
    if secure {
        headers.insert(header::STRICT_TRANSPORT_SECURITY, HeaderValue::from_static("max-age=31536000"));
    }
    if is_static {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600, must-revalidate"));
    }
    if !is_static {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    const BASE: &str = "https://example.com";

    #[test]
    fn forged_and_missing_browser_origins_are_rejected() {
        for origin in ["https://evil.example", "https://example.com.evil.example", "https://sub.example.com", "http://example.com", "null"] {
            let req = TestRequest::post().insert_header(("Origin", origin)).to_http_request();
            assert!(!csrf_allowed(&req, BASE), "{origin}");
        }
        assert!(!csrf_allowed(&TestRequest::post().to_http_request(), BASE));
        let req = TestRequest::post().insert_header(("Host", "evil.example"))
            .insert_header(("X-Forwarded-Host", "evil.example"))
            .insert_header(("Origin", "https://evil.example")).to_http_request();
        assert!(!csrf_allowed(&req, BASE));
    }

    #[test]
    fn same_origin_forms_and_explicit_api_credentials_work() {
        assert!(csrf_allowed(&TestRequest::post().insert_header(("Origin","null")).insert_header(("Sec-Fetch-Site","same-origin")).to_http_request(), BASE));
        assert!(!csrf_allowed(&TestRequest::post().insert_header(("Origin","null")).insert_header(("Sec-Fetch-Site","cross-site")).to_http_request(), BASE));
        for header in [("Origin", BASE), ("Referer", "https://example.com/buchen"), ("Sec-Fetch-Site", "same-origin"), ("Authorization", "Bearer example")] {
            assert!(csrf_allowed(&TestRequest::post().insert_header(header).to_http_request(), BASE));
        }
        let req = TestRequest::post().insert_header(("Authorization", "Bearer example"))
            .cookie(actix_web::cookie::Cookie::new("auth_token", "cookie"))
            .to_http_request();
        assert!(!csrf_allowed(&req, BASE));
        assert!(csrf_allowed(&TestRequest::get().uri("/admin/nextcloud/callback?code=secret").to_http_request(), BASE));
    }

    #[test]
    fn auth_limits_are_shared_bounded_and_expire() {
        let limiter = AuthLimiter::default();
        let peer = "127.0.0.1".parse().unwrap();
        let now = Instant::now();
        for _ in 0..20 { assert!(limiter.allow(peer, now)); }
        assert!(!limiter.allow(peer, now));
        assert!(limiter.allow("127.0.0.2".parse().unwrap(), now));
        assert!(limiter.allow(peer, now + Duration::from_secs(61)));
    }

    #[actix_web::test]
    async fn protection_covers_success_errors_and_token_responses() {
        let config = crate::config::test_config();
        let app = actix_web::test::init_service(actix_web::App::new()
            .app_data(web::Data::new(config))
            .wrap(actix_web::middleware::from_fn(protect))
            .route("/reset-password/{token}", web::get().to(|| async { HttpResponse::Ok().body("reset") }))
            .route("/api/save", web::post().to(|| async { HttpResponse::Ok().finish() }))).await;
        for request in [TestRequest::get().uri("/reset-password/private-token"),
            TestRequest::get().uri("/missing"),
            TestRequest::post().uri("/api/save").insert_header(("Origin", "https://evil.example"))] {
            let response = actix_web::test::call_service(&app, request.to_request()).await;
            assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
            assert_eq!(response.headers().get("referrer-policy").unwrap(), "no-referrer");
            assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
            assert_eq!(response.headers().get("x-content-type-options").unwrap(), "nosniff");
        }
        let response = actix_web::test::call_service(&app, TestRequest::post().uri("/api/save")
            .insert_header(("Origin", "https://example.com")).to_request()).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }
}
