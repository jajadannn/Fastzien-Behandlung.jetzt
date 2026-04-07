use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

#[derive(Debug, Clone)]
pub struct BusyPeriod {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

/// Returns true if a slot [slot_start, slot_end) overlaps with any busy period.
pub fn has_conflict(busy: &[BusyPeriod], slot_start: &NaiveDateTime, slot_end: &NaiveDateTime) -> bool {
    busy.iter().any(|b| b.start < *slot_end && b.end > *slot_start)
}

/// Fetch busy time periods from a Nextcloud CalDAV calendar for the given date.
/// Returns an empty vec on any error (connection failure, auth error, parse error).
pub async fn fetch_busy_periods(
    caldav_url: &str,
    username: &str,
    password: &str,
    date: &NaiveDate,
) -> Vec<BusyPeriod> {
    // Query the full UTC day so we capture all German-timezone events
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
        Err(e) => {
            log::warn!("CalDAV: failed to build HTTP client: {}", e);
            return vec![];
        }
    };

    // Ensure URL ends with /
    let url = format!("{}/", caldav_url.trim_end_matches('/'));

    let response = match client
        .request(
            reqwest::Method::from_bytes(b"REPORT").expect("REPORT is a valid method"),
            &url,
        )
        .basic_auth(username, Some(password))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "1")
        .body(body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log::warn!("CalDAV: request to {} failed: {}", url, e);
            return vec![];
        }
    };

    let status = response.status();
    // CalDAV returns 207 Multi-Status on success
    if !status.is_success() && status.as_u16() != 207 {
        log::warn!("CalDAV: unexpected HTTP status {} from {}", status, url);
        return vec![];
    }

    let text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            log::warn!("CalDAV: failed to read response body: {}", e);
            return vec![];
        }
    };

    log::debug!("CalDAV: got {} bytes for {}", text.len(), date);
    parse_busy_periods(&text, date)
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
