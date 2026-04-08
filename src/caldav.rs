use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};

#[derive(Debug, Clone)]
pub struct BusyPeriod {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

/// Authentication mode for CalDAV requests.
#[derive(Debug, Clone)]
pub enum CalDavAuth {
    Basic { user: String, pass: String },
    Bearer(String),
}

/// Returns true if a slot [slot_start, slot_end) overlaps with any busy period.
pub fn has_conflict(busy: &[BusyPeriod], slot_start: &NaiveDateTime, slot_end: &NaiveDateTime) -> bool {
    busy.iter().any(|b| b.start < *slot_end && b.end > *slot_start)
}


fn parse_busy_periods(xml: &str, date: &NaiveDate) -> Vec<BusyPeriod> {
    let mut busy = Vec::new();
    let mut remaining = xml;

    while let Some(idx) = remaining.find("BEGIN:VCALENDAR") {
        remaining = &remaining[idx..];
        let end_idx = remaining
            .find("END:VCALENDAR")
            .map(|i| i + "END:VCALENDAR".len())
            .unwrap_or(remaining.len());
        let vcalendar = &remaining[..end_idx];
        remaining = &remaining[end_idx..];

        let mut vevent_remaining = vcalendar;
        while let Some(ve_start) = vevent_remaining.find("BEGIN:VEVENT") {
            vevent_remaining = &vevent_remaining[ve_start..];
            let ve_end = vevent_remaining
                .find("END:VEVENT")
                .map(|i| i + "END:VEVENT".len())
                .unwrap_or(vevent_remaining.len());
            let vevent_raw = &vevent_remaining[..ve_end];
            vevent_remaining = &vevent_remaining[ve_end..];

            // Unfold RFC 5545 line continuations (CRLF + space/tab)
            let vevent = unfold_ical(vevent_raw);

            if let (Some(start), Some(end)) = (
                parse_dt_prop(&vevent, "DTSTART"),
                parse_dt_prop(&vevent, "DTEND"),
            ) {
                // Include if the event touches our target date
                if start.date() <= *date && end.date() >= *date {
                    log::debug!("CalDAV: busy {} – {}", start, end);
                    busy.push(BusyPeriod { start, end });
                }
            }
        }
    }

    busy
}

/// Unfold iCalendar line continuations (CRLF + whitespace or LF + whitespace).
fn unfold_ical(s: &str) -> String {
    s.replace("\r\n ", "")
        .replace("\r\n\t", "")
        .replace("\n ", "")
        .replace("\n\t", "")
}

/// Parse a DTSTART or DTEND property line from an unfolded VEVENT block.
/// Handles: UTC (Z), TZID=Europe/Berlin (local), DATE-only (all-day).
fn parse_dt_prop(vevent: &str, prop: &str) -> Option<NaiveDateTime> {
    for line in vevent.lines() {
        let line = line.trim();
        if !line.starts_with(prop) {
            continue;
        }
        let after = &line[prop.len()..];
        if !after.starts_with(':') && !after.starts_with(';') {
            continue;
        }

        let is_utc = line.ends_with('Z');
        let is_date_only = after.contains("VALUE=DATE") && !after.contains("DATE-TIME");

        // Value is everything after the last ':'
        let value = line.rfind(':').map(|i| &line[i + 1..])?;
        let value = value.trim_end_matches('Z');

        let dt = if is_date_only {
            // All-day event: block the whole day
            let d = NaiveDate::parse_from_str(value, "%Y%m%d").ok()?;
            NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        } else {
            NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S").ok()?
        };

        return Some(if is_utc {
            dt + Duration::hours(germany_utc_offset(&dt))
        } else {
            // TZID or floating — treat as Europe/Berlin local time
            dt
        });
    }
    None
}

/// Approximate UTC→Europe/Berlin offset.
/// CET = UTC+1 (winter), CEST = UTC+2 (Apr–Oct, roughly).
fn germany_utc_offset(utc: &NaiveDateTime) -> i64 {
    let month = utc.month();
    if month < 3 || month > 10 {
        return 1; // Nov–Feb: CET
    }
    if month > 3 && month < 10 {
        return 2; // Apr–Sep: CEST
    }
    // March: DST starts on last Sunday at 02:00 UTC
    if month == 3 {
        let last_sun = last_sunday_of(utc.year(), 3);
        if utc.day() > last_sun || (utc.day() == last_sun && utc.hour() >= 2) {
            return 2;
        }
        return 1;
    }
    // October: DST ends on last Sunday at 03:00 UTC
    let last_sun = last_sunday_of(utc.year(), 10);
    if utc.day() < last_sun || (utc.day() == last_sun && utc.hour() < 3) {
        return 2;
    }
    1
}

fn last_sunday_of(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let last = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap()
        - Duration::days(1);
    last.day() - last.weekday().num_days_from_sunday()
}

// ── Multi-calendar helpers ────────────────────────────────────────────────────

/// Discover all CalDAV calendar URLs for a Nextcloud user via OAuth Bearer token.
///
/// Strategy:
///   1. GET /ocs/v1.php/cloud/user  → get the logged-in username
///   2. PROPFIND /remote.php/dav/calendars/{username}/  Depth:1 → list calendars
///   3. Filter results to real VCALENDAR collections (skip principal/inbox/outbox)
pub async fn discover_calendars(base_url: &str, token: &str) -> Vec<String> {
    let base = base_url.trim_end_matches('/');

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV discover: client error: {}", e); return vec![]; }
    };

    // ── Step 1: get username from Nextcloud OCS API ──────────────────────────
    let ocs_url = format!("{}/ocs/v1.php/cloud/user", base);
    let ocs_resp = match client
        .get(&ocs_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("OCS-APIRequest", "true")
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => { log::warn!("CalDAV discover: OCS user request failed: {}", e); return vec![]; }
    };

    let ocs_json: serde_json::Value = match ocs_resp.json().await {
        Ok(j) => j,
        Err(e) => { log::warn!("CalDAV discover: OCS JSON parse failed: {}", e); return vec![]; }
    };

    let username = match ocs_json["ocs"]["data"]["id"].as_str() {
        Some(u) => u.to_string(),
        None => {
            log::warn!("CalDAV discover: could not find username in OCS response: {}", ocs_json);
            return vec![];
        }
    };
    log::info!("CalDAV discover: Nextcloud username = {}", username);

    // ── Step 2: PROPFIND calendar home ───────────────────────────────────────
    let calendars_url = format!("{}/remote.php/dav/calendars/{}/", base, username);
    let propfind_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<D:propfind xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:prop>
    <D:resourcetype/>
    <D:displayname/>
  </D:prop>
</D:propfind>"#;

    let resp = match client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &calendars_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "1")
        .body(propfind_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => { log::warn!("CalDAV discover: PROPFIND failed: {}", e); return vec![]; }
    };

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    log::debug!("CalDAV discover: PROPFIND status={} body_len={}", status, text.len());

    if !status.is_success() && status.as_u16() != 207 {
        log::warn!("CalDAV discover: PROPFIND returned {}", status);
        return vec![];
    }

    extract_calendar_urls(&text, base)
}

/// Parse calendar URLs from a PROPFIND Depth:1 response.
/// Only returns entries that have `<cal:calendar/>` in their resourcetype.
fn extract_calendar_urls(xml: &str, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    // Work on lowercase copy for case-insensitive matching, but extract from original
    let lower = xml.to_lowercase();
    let mut search_from = 0usize;

    loop {
        // Find the next <d:response> block
        let block_start = match lower[search_from..].find("<d:response>") {
            Some(i) => search_from + i,
            None => break,
        };
        let inner_start = block_start + "<d:response>".len();
        let block_end = match lower[inner_start..].find("</d:response>") {
            Some(i) => inner_start + i,
            None => break,
        };
        let block_lower = &lower[inner_start..block_end];
        let block_orig  = &xml[inner_start..block_end];
        search_from = block_end + "</d:response>".len();

        // Must contain <cal:calendar/> or <x1:calendar/> in resourcetype
        // Skip the calendar home collection itself, inbox, outbox, notification, etc.
        if !block_lower.contains(":calendar") && !block_lower.contains("\"calendar\"") {
            continue;
        }
        // Exclude known non-calendar collections
        if block_lower.contains("principal")
            || block_lower.contains("calendar-inbox")
            || block_lower.contains("calendar-outbox")
            || block_lower.contains("notification")
            || block_lower.contains("schedule-inbox")
            || block_lower.contains("schedule-outbox")
        {
            continue;
        }

        // Extract <d:href>
        let href = match block_lower.find("<d:href>") {
            Some(i) => {
                let content = &block_orig[i + 8..];
                let end = content.find('<').unwrap_or(content.len());
                content[..end].trim().to_string()
            }
            None => continue,
        };
        if href.is_empty() { continue; }

        // Build absolute URL
        let url = if href.starts_with("http") {
            href
        } else {
            format!("{}{}", base_url, href)
        };

        if !urls.contains(&url) {
            log::info!("CalDAV discover: found calendar {}", url);
            urls.push(url);
        }
    }

    urls
}

/// Fetch busy periods from multiple calendar URLs, merging results.
pub async fn fetch_busy_periods_multi(
    urls: &[String],
    auth: &CalDavAuth,
    date: &NaiveDate,
) -> Vec<BusyPeriod> {
    let query_start = date.format("%Y%m%dT000000Z").to_string();
    let query_end = (*date + Duration::days(1)).format("%Y%m%dT000000Z").to_string();

    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<C:calendar-query xmlns:C="urn:ietf:params:xml:ns:caldav" xmlns:D="DAV:">
  <D:prop><D:getetag/><C:calendar-data/></D:prop>
  <C:filter>
    <C:comp-filter name="VCALENDAR">
      <C:comp-filter name="VEVENT">
        <C:time-range start="{}" end="{}"/>
      </C:comp-filter>
    </C:comp-filter>
  </C:filter>
</C:calendar-query>"#,
        query_start, query_end
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => { log::warn!("CalDAV multi: client error: {}", e); return vec![]; }
    };

    let mut all_busy = Vec::new();

    for url in urls {
        let url_with_slash = format!("{}/", url.trim_end_matches('/'));
        let mut req = client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), &url_with_slash)
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.clone());

        req = match auth {
            CalDavAuth::Basic { user, pass } => req.basic_auth(user, Some(pass)),
            CalDavAuth::Bearer(token) => req.header("Authorization", format!("Bearer {}", token)),
        };

        match req.send().await {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 207 => {
                if let Ok(text) = r.text().await {
                    let busy = parse_busy_periods(&text, date);
                    log::debug!("CalDAV multi: {} busy periods from {}", busy.len(), url);
                    all_busy.extend(busy);
                }
            }
            Ok(r) => log::warn!("CalDAV multi: status {} from {}", r.status(), url),
            Err(e) => log::warn!("CalDAV multi: error from {}: {}", url, e),
        }
    }

    all_busy
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
        "Sulgauer Straße 24, 78655 Sulgen".to_string()
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
        Err(e) => log::warn!("CalDAV push: request failed for event {}: {}", appointment_id, e),
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
        Err(e) => log::warn!("CalDAV delete: request failed for event {}: {}", appointment_id, e),
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

    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(8)).build() {
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
        Err(e) => log::warn!("CalDAV push_bearer: failed for event {}: {}", appointment_id, e),
    }
}

/// Delete a VEVENT using OAuth2 Bearer token authentication.
pub async fn delete_event_bearer(caldav_url: &str, token: &str, appointment_id: i64) {
    let url = format!("{}/{}", caldav_url.trim_end_matches('/'), event_filename(appointment_id));

    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(8)).build() {
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
        Err(e) => log::warn!("CalDAV delete_bearer: failed for event {}: {}", appointment_id, e),
    }
}
