use chrono::{Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Berlin;

const DAV: &str = "DAV:";
const CALDAV: &str = "urn:ietf:params:xml:ns:caldav";
const CALENDAR_UNAVAILABLE: &str = "Der Kalender kann gerade nicht zuverlässig geprüft werden.";
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct BusyPeriod {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub uid: Option<String>,
}

#[derive(Clone)]
pub enum CalDavAuth {
    Basic { user: String, pass: String },
    Bearer(String),
}

pub fn has_conflict(busy: &[BusyPeriod], slot_start: &NaiveDateTime, slot_end: &NaiveDateTime) -> bool {
    busy.iter().any(|b| b.start < *slot_end && b.end > *slot_start)
}

fn unfold_ical(s: &str) -> String {
    s.replace("\r\n ", "").replace("\r\n\t", "").replace("\n ", "").replace("\n\t", "")
}

fn prop_value<'a>(event: &'a str, property: &str) -> Option<(&'a str, &'a str)> {
    event.lines().find_map(|line| {
        let (key, value) = line.trim().split_once(':')?;
        (key.split(';').next()? == property).then_some((key, value))
    })
}

fn parse_dt_prop(event: &str, property: &str) -> Option<NaiveDateTime> {
    let (key, value) = prop_value(event, property)?;
    if key.split(';').any(|p| p == "VALUE=DATE") {
        return NaiveDate::parse_from_str(value, "%Y%m%d").ok()?.and_hms_opt(0, 0, 0);
    }
    let dt = NaiveDateTime::parse_from_str(value.trim_end_matches('Z'), "%Y%m%dT%H%M%S").ok()?;
    if value.ends_with('Z') {
        return Some(dt.and_utc().with_timezone(&Berlin).naive_local());
    }
    match key.split(';').find_map(|p| p.strip_prefix("TZID=")) {
        Some(zone) => {
            let zone: chrono_tz::Tz = zone.trim_matches('"').parse().ok()?;
            Some(zone.from_local_datetime(&dt).single()?.with_timezone(&Berlin).naive_local())
        }
        None => Some(dt), // Floating time follows the practice's Europe/Berlin calendar.
    }
}

fn parse_duration(value: &str) -> Option<Duration> {
    let mut chars = value.strip_prefix('P')?.chars();
    let mut seconds = 0i64;
    let mut number = String::new();
    let mut time = false;
    let mut components = 0;
    for c in chars.by_ref() {
        if c == 'T' && number.is_empty() && !time { time = true; continue; }
        if c.is_ascii_digit() { number.push(c); continue; }
        let factor = match (c, time) {
            ('W', false) => 604800, ('D', false) => 86400,
            ('H', true) => 3600, ('M', true) => 60, ('S', true) => 1,
            _ => return None,
        };
        seconds = seconds.checked_add(number.parse::<i64>().ok()?.checked_mul(factor)?)?;
        number.clear();
        components += 1;
    }
    if !number.is_empty() || components == 0 || seconds <= 0 { return None; }
    Duration::try_seconds(seconds)
}

fn parse_busy_periods(xml: &str, date: &NaiveDate) -> Result<Vec<BusyPeriod>, &'static str> {
    let doc = roxmltree::Document::parse(xml).map_err(|_| CALENDAR_UNAVAILABLE)?;
    if !doc.root_element().has_tag_name((DAV, "multistatus")) { return Err(CALENDAR_UNAVAILABLE); }
    let mut busy = Vec::new();
    for response in doc.root_element().children().filter(|n| n.has_tag_name((DAV, "response"))) {
        if response.descendants().filter(|n| n.has_tag_name((DAV, "status")))
            .any(|n| n.text().and_then(|v| v.split_whitespace().nth(1)) != Some("200")) {
            return Err(CALENDAR_UNAVAILABLE);
        }
        let data: Vec<_> = response.descendants().filter(|n| n.has_tag_name((CALDAV, "calendar-data"))).collect();
        if data.is_empty() { return Err(CALENDAR_UNAVAILABLE); }
        for node in data {
            let calendar = unfold_ical(node.text().ok_or(CALENDAR_UNAVAILABLE)?);
            if !calendar.contains("BEGIN:VCALENDAR") || !calendar.contains("END:VCALENDAR") {
                return Err(CALENDAR_UNAVAILABLE);
            }
            for raw_event in calendar.split("BEGIN:VEVENT").skip(1) {
                let (event, _) = raw_event.split_once("END:VEVENT").ok_or(CALENDAR_UNAVAILABLE)?;
                if prop_value(event, "STATUS").is_some_and(|(_, v)| v == "CANCELLED")
                    || prop_value(event, "TRANSP").is_some_and(|(_, v)| v == "TRANSPARENT") { continue; }
                // The REPORT requests expanded instances; an unexpanded rule must
                // never silently make future occurrences appear available.
                if prop_value(event, "RRULE").is_some() || prop_value(event, "RDATE").is_some() {
                    return Err(CALENDAR_UNAVAILABLE);
                }
                let start = parse_dt_prop(event, "DTSTART").ok_or(CALENDAR_UNAVAILABLE)?;
                let end = if prop_value(event, "DTEND").is_some() {
                    parse_dt_prop(event, "DTEND").ok_or(CALENDAR_UNAVAILABLE)?
                } else if let Some((_, duration)) = prop_value(event, "DURATION") {
                    start.checked_add_signed(parse_duration(duration).ok_or(CALENDAR_UNAVAILABLE)?).ok_or(CALENDAR_UNAVAILABLE)?
                } else if prop_value(event, "DTSTART").is_some_and(|(key, _)| key.contains("VALUE=DATE")) {
                    start.checked_add_signed(Duration::days(1)).ok_or(CALENDAR_UNAVAILABLE)?
                } else {
                    start // RFC 5545: a date-time event without DTEND/DURATION has zero duration.
                };
                if end < start { return Err(CALENDAR_UNAVAILABLE); }
                if start.date() <= *date && end.date() >= *date {
                    busy.push(BusyPeriod { start, end, uid: prop_value(event, "UID").map(|(_, v)| v.to_owned()) });
                }
            }
        }
    }
    Ok(busy)
}

async fn limited_text(mut response: reqwest::Response) -> Result<String, &'static str> {
    if response.content_length().is_some_and(|len| len > MAX_RESPONSE_BYTES as u64) { return Err(CALENDAR_UNAVAILABLE); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| CALENDAR_UNAVAILABLE)? {
        if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES { return Err(CALENDAR_UNAVAILABLE); }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| CALENDAR_UNAVAILABLE)
}

/// Parse namespaces and entities without enabling DTD/external entities. Only
/// same-origin collections can receive the Nextcloud bearer credential.
fn extract_calendar_urls(xml: &str, base_url: &str) -> Vec<String> {
    let Ok(base) = reqwest::Url::parse(base_url) else { return vec![]; };
    let Ok(doc) = roxmltree::Document::parse(xml) else { return vec![]; };
    let mut urls = Vec::new();
    for response in doc.descendants().filter(|n| n.has_tag_name((DAV, "response"))) {
        if !response.descendants().any(|n| n.has_tag_name((CALDAV, "calendar"))) { continue; }
        let Some(href) = response.children().find(|n| n.has_tag_name((DAV, "href"))).and_then(|n| n.text()) else { continue; };
        let Ok(url) = base.join(href.trim()) else { continue; };
        if url.origin() != base.origin() || !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty() || url.password().is_some()
            || url.query().is_some() || url.fragment().is_some() { continue; }
        let url = url.as_str().trim_end_matches('/').to_owned();
        if !urls.contains(&url) { urls.push(url); }
    }
    urls
}

pub async fn discover_calendars(base_url: &str, token: &str) -> Vec<String> {
    let base = base_url.trim_end_matches('/');
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).build() {
        Ok(client) => client, Err(_) => return vec![],
    };
    let response = match client.get(format!("{base}/ocs/v1.php/cloud/user"))
        .bearer_auth(token).header("OCS-APIRequest", "true").header("Accept", "application/json").send().await {
        Ok(r) if r.status().is_success() => r, _ => return vec![],
    };
    let Ok(text) = limited_text(response).await else { return vec![]; };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { return vec![]; };
    let Some(username) = json["ocs"]["data"]["id"].as_str() else { return vec![]; };
    let url = format!("{base}/remote.php/dav/calendars/{}/", urlencoding::encode(username));
    let response = match client.request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .bearer_auth(token).header("Content-Type", "application/xml; charset=utf-8").header("Depth", "1")
        .body(r#"<D:propfind xmlns:D="DAV:"><D:prop><D:resourcetype/><D:displayname/></D:prop></D:propfind>"#)
        .send().await {
        Ok(r) if r.status().is_success() => r, _ => return vec![],
    };
    match limited_text(response).await {
        Ok(text) => extract_calendar_urls(&text, &url), Err(_) => vec![],
    }
}

fn query_range(date: &NaiveDate) -> Result<(String, String), &'static str> {
    let next = date.succ_opt().ok_or(CALENDAR_UNAVAILABLE)?;
    let midnight = |day: NaiveDate| Berlin.from_local_datetime(&day.and_hms_opt(0, 0, 0).unwrap())
        .single().map(|dt| dt.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string())
        .ok_or(CALENDAR_UNAVAILABLE);
    Ok((midnight(*date)?, midnight(next)?))
}

/// Any failed configured calendar makes availability unknown, never free.
pub async fn fetch_busy_periods_multi(urls: &[String], auth: &CalDavAuth, date: &NaiveDate)
    -> Result<Vec<BusyPeriod>, &'static str> {
    let (start, end) = query_range(date)?;
    let body = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<C:calendar-query xmlns:C="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
<D:prop><C:calendar-data><C:expand start="{start}" end="{end}"/></C:calendar-data></D:prop>
<C:filter><C:comp-filter name="VCALENDAR"><C:comp-filter name="VEVENT">
<C:time-range start="{start}" end="{end}"/>
</C:comp-filter></C:comp-filter></C:filter></C:calendar-query>"#);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none()).build().map_err(|_| CALENDAR_UNAVAILABLE)?;
    let mut busy = Vec::new();
    for url in urls {
        let mut request = client.request(reqwest::Method::from_bytes(b"REPORT").unwrap(), format!("{}/", url.trim_end_matches('/')))
            .header("Content-Type", "application/xml; charset=utf-8").header("Depth", "1").body(body.clone());
        request = match auth {
            CalDavAuth::Basic { user, pass } => request.basic_auth(user, Some(pass)),
            CalDavAuth::Bearer(token) => request.bearer_auth(token),
        };
        let response = request.send().await.map_err(|_| CALENDAR_UNAVAILABLE)?;
        if !response.status().is_success() { return Err(CALENDAR_UNAVAILABLE); }
        busy.extend(parse_busy_periods(&limited_text(response).await?, date)?);
    }
    Ok(busy)
}

// ── Write helpers ─────────────────────────────────────────────────────────────

/// Deterministic UID and filename for an appointment.
fn event_uid(appointment_id: i64) -> String {
    format!("faszien-{}@faszienbehandlung.jetzt", appointment_id)
}
fn event_filename(appointment_id: i64) -> String {
    format!("faszien-{}.ics", appointment_id)
}

/// Escape special iCalendar characters in text values.
fn ical_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn build_ical(
    appointment_id: i64,
    start: &NaiveDateTime,
    end: &NaiveDateTime,
    customer_name: &str,
    customer_phone: &str,
    customer_address: &str,
    notes: &str,
    is_home_visit: bool,
) -> String {
    let summary = if is_home_visit {
        format!("{} – Hausbesuch", customer_name)
    } else {
        format!("{} – Faszienbehandlung", customer_name)
    };
    let location = if is_home_visit {
        format!("Hausbesuch: {}", customer_address)
    } else {
        "Sulgauer Straße 24, 78713 Schramberg-Sulgen".to_string()
    };
    let mut desc_parts = vec![
        format!("Kunde: {}", customer_name),
        format!("Telefon: {}", customer_phone),
    ];
    if !customer_address.is_empty() { desc_parts.push(format!("Adresse: {}", customer_address)); }
    if !notes.is_empty()            { desc_parts.push(format!("Notizen: {}", notes)); }
    let description = ical_escape(&desc_parts.join("\n"));
    let dtstamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Faszienbehandlung.jetzt//Buchungssystem//DE\r\n\
         BEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:{dtstamp}\r\n\
         DTSTART;TZID=Europe/Berlin:{dtstart}\r\n\
         DTEND;TZID=Europe/Berlin:{dtend}\r\n\
         SUMMARY:{summary}\r\n\
         DESCRIPTION:{description}\r\n\
         LOCATION:{location}\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n",
        uid         = event_uid(appointment_id),
        dtstamp     = dtstamp,
        dtstart     = start.format("%Y%m%dT%H%M%S"),
        dtend       = end.format("%Y%m%dT%H%M%S"),
        summary     = ical_escape(&summary),
        description = description,
        location    = ical_escape(&location),
    )
}

/// Push (create or update) a VEVENT on the Nextcloud CalDAV calendar.
/// Called after a booking is confirmed. Runs silently on error.
pub async fn push_event(
    caldav_url: &str,
    username: &str,
    password: &str,
    appointment_id: i64,
    start: &NaiveDateTime,
    end: &NaiveDateTime,
    customer_name: &str,
    customer_phone: &str,
    customer_address: &str,
    notes: &str,
    is_home_visit: bool,
) {
    let ical = build_ical(appointment_id, start, end, customer_name, customer_phone, customer_address, notes, is_home_visit);
    let url = format!("{}/{}", caldav_url.trim_end_matches('/'), event_filename(appointment_id));

    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV push: client error: {}", e); return; }
    };

    match client
        .put(&url)
        .basic_auth(username, Some(password))
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(ical)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 201 || r.status().as_u16() == 204 => {
            log::info!("CalDAV push: created event {} (status {})", appointment_id, r.status());
        }
        Ok(r) => log::warn!("CalDAV push: unexpected status {} for event {}", r.status(), appointment_id),
        Err(e) => log::warn!("CalDAV push: request failed for event {}: {}", appointment_id, e.without_url()),
    }
}

/// Delete a VEVENT from the Nextcloud CalDAV calendar.
/// Called when an appointment is cancelled. Runs silently on error.
pub async fn delete_event(
    caldav_url: &str,
    username: &str,
    password: &str,
    appointment_id: i64,
) {
    let url = format!("{}/{}", caldav_url.trim_end_matches('/'), event_filename(appointment_id));

    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV delete: client error: {}", e); return; }
    };

    match client
        .delete(&url)
        .basic_auth(username, Some(password))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 204 || r.status().as_u16() == 404 => {
            log::info!("CalDAV delete: removed event {} (status {})", appointment_id, r.status());
        }
        Ok(r) => log::warn!("CalDAV delete: unexpected status {} for event {}", r.status(), appointment_id),
        Err(e) => log::warn!("CalDAV delete: request failed for event {}: {}", appointment_id, e.without_url()),
    }
}

/// Push a VEVENT using OAuth2 Bearer token authentication.
pub async fn push_event_bearer(
    caldav_url: &str,
    token: &str,
    appointment_id: i64,
    start: &NaiveDateTime,
    end: &NaiveDateTime,
    customer_name: &str,
    customer_phone: &str,
    customer_address: &str,
    notes: &str,
    is_home_visit: bool,
) {
    let ical = build_ical(appointment_id, start, end, customer_name, customer_phone, customer_address, notes, is_home_visit);
    let url = format!("{}/{}", caldav_url.trim_end_matches('/'), event_filename(appointment_id));

    let client = match reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV push_bearer: client error: {}", e); return; }
    };

    match client
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(ical)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 201 || r.status().as_u16() == 204 =>
            log::info!("CalDAV push_bearer: created event {} ({})", appointment_id, r.status()),
        Ok(r) => log::warn!("CalDAV push_bearer: status {} for event {}", r.status(), appointment_id),
        Err(e) => log::warn!("CalDAV push_bearer: failed for event {}: {}", appointment_id, e.without_url()),
    }
}

/// Delete a VEVENT using OAuth2 Bearer token authentication.
pub async fn delete_event_bearer(caldav_url: &str, token: &str, appointment_id: i64) {
    let url = format!("{}/{}", caldav_url.trim_end_matches('/'), event_filename(appointment_id));

    let client = match reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV delete_bearer: client error: {}", e); return; }
    };

    match client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 204 || r.status().as_u16() == 404 =>
            log::info!("CalDAV delete_bearer: removed event {} ({})", appointment_id, r.status()),
        Ok(r) => log::warn!("CalDAV delete_bearer: status {} for event {}", r.status(), appointment_id),
        Err(e) => log::warn!("CalDAV delete_bearer: failed for event {}: {}", appointment_id, e.without_url()),
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    fn report(event: &str) -> String {
        format!(r#"<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav"><d:response><d:propstat><d:prop><c:calendar-data><![CDATA[BEGIN:VCALENDAR
BEGIN:VEVENT
{event}
END:VEVENT
END:VCALENDAR]]></c:calendar-data></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#)
    }

    #[test]
    fn timezone_changes_use_iana_rules_and_local_day_boundaries() {
        assert_eq!(parse_dt_prop("DTSTART:20260329T010000Z", "DTSTART").unwrap().to_string(), "2026-03-29 03:00:00");
        assert_eq!(parse_dt_prop("DTSTART:20261025T010000Z", "DTSTART").unwrap().to_string(), "2026-10-25 02:00:00");
        assert_eq!(parse_dt_prop("DTSTART;TZID=America/New_York:20261008T100000", "DTSTART").unwrap().to_string(), "2026-10-08 16:00:00");
        let date = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        assert_eq!(query_range(&date).unwrap(), ("20261024T220000Z".into(), "20261025T230000Z".into()));
    }

    #[test]
    fn calendar_discovery_handles_unicode_namespaces_and_rejects_foreign_origins() {
        let xml = r#"<x:multistatus xmlns:x="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
<x:response><x:href>/cloud/dav/İstanbul/</x:href><x:propstat><x:prop><x:displayname>İstanbul</x:displayname><x:resourcetype><c:calendar/></x:resourcetype></x:prop></x:propstat></x:response>
<x:response><x:href>https://evil.example/calendar/</x:href><x:propstat><x:prop><x:resourcetype><c:calendar/></x:resourcetype></x:prop></x:propstat></x:response>
<x:response><x:href>//evil.example/calendar/</x:href><x:propstat><x:prop><x:resourcetype><c:calendar/></x:resourcetype></x:prop></x:propstat></x:response>
<x:response><x:href>https://user:pass@example.com/calendar/</x:href><x:propstat><x:prop><x:resourcetype><c:calendar/></x:resourcetype></x:prop></x:propstat></x:response>
</x:multistatus>"#;
        assert_eq!(extract_calendar_urls(xml, "https://example.com/cloud/"), vec!["https://example.com/cloud/dav/%C4%B0stanbul"]);
    }

    #[test]
    fn calendar_errors_and_unexpanded_recurrences_are_not_free_time() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 8).unwrap();
        for xml in ["<html>Login required</html>".to_owned(), "invalid".into(),
            report("DTSTART:20261008T100000Z\nDTEND:not-a-date"),
            report("DTSTART:20261008T100000Z\nRRULE:FREQ=DAILY"),
            report("DTSTART:20261008T100000Z").replace("200 OK", "403 Forbidden")] {
            assert!(parse_busy_periods(&xml, &date).is_err());
        }
        assert!(parse_busy_periods(r#"<multistatus xmlns="DAV:"/>"#, &date).unwrap().is_empty());
    }

    #[test]
    fn duration_all_day_and_uids_are_preserved() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 8).unwrap();
        let events = parse_busy_periods(&report("UID:faszien-7@faszienbehandlung.jetzt\nDTSTART:20261008T140000Z\nDURATION:PT90M"), &date).unwrap();
        assert_eq!(events[0].uid.as_deref(), Some("faszien-7@faszienbehandlung.jetzt"));
        assert_eq!(events[0].start.to_string(), "2026-10-08 16:00:00");
        assert_eq!(events[0].end.to_string(), "2026-10-08 17:30:00");
        let events = parse_busy_periods(&report("DTSTART;VALUE=DATE:20261008"), &date).unwrap();
        assert_eq!(events[0].end.to_string(), "2026-10-09 00:00:00");
        assert!(parse_busy_periods(&report("DTSTART;VALUE=DATE:20261008\nTRANSP:TRANSPARENT"), &date).unwrap().is_empty());
    }

    #[actix_web::test]
    async fn http_errors_redirects_and_malformed_responses_fail_closed() {
        use std::{io::{Read, Write}, net::TcpListener};
        for (status, body) in [("401 Unauthorized", ""), ("302 Found", ""), ("207 Multi-Status", "<html>Login</html>"), ("207 Multi-Status", r#"<multistatus xmlns="DAV:"/>"#)] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let reply = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\nLocation: http://{address}/redirect\r\n\r\n{body}", body.len());
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                stream.write_all(reply.as_bytes()).unwrap();
            });
            let result = fetch_busy_periods_multi(&[format!("http://{address}/calendar")],
                &CalDavAuth::Bearer("test-token".into()), &NaiveDate::from_ymd_opt(2026, 10, 8).unwrap()).await;
            server.join().unwrap();
            assert_eq!(result.is_ok(), body.contains("multistatus"), "{status} {body}");
        }
    }
}
