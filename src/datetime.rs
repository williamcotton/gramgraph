use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

pub const DEFAULT_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub fn parse_datetime_value(value: &str) -> Result<f64> {
    let value = value.trim();

    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.timestamp() as f64);
    }

    for format in [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(dt.and_utc().timestamp() as f64);
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("Failed to build midnight datetime for '{}'", value))?;
        return Ok(dt.and_utc().timestamp() as f64);
    }

    Err(anyhow!(
        "Failed to parse datetime value '{}'. Expected ISO/RFC3339-like values such as 2026-05-18T00:00",
        value
    ))
}

pub fn parse_datetime_interval_seconds(value: &str) -> Result<f64> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return Err(anyhow!("Datetime interval cannot be empty"));
    }

    let split_at = value
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .ok_or_else(|| anyhow!("Datetime interval '{}' is missing a unit", value))?;
    let (amount_str, unit_str) = value.split_at(split_at);
    let amount: f64 = amount_str
        .trim()
        .parse()
        .map_err(|_| anyhow!("Datetime interval '{}' has an invalid number", value))?;
    if amount <= 0.0 || !amount.is_finite() {
        return Err(anyhow!("Datetime interval '{}' must be positive", value));
    }

    let unit = unit_str.trim();
    let multiplier = match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
        "m" | "min" | "mins" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 60.0 * 60.0,
        "d" | "day" | "days" => 24.0 * 60.0 * 60.0,
        "w" | "week" | "weeks" => 7.0 * 24.0 * 60.0 * 60.0,
        _ => {
            return Err(anyhow!(
                "Unsupported datetime interval unit '{}'. Use s, m, h, d, or w",
                unit
            ))
        }
    };

    Ok(amount * multiplier)
}

pub fn format_datetime_tick(seconds: f64, format: &str) -> String {
    let rounded = seconds.round();
    if !rounded.is_finite() {
        return String::new();
    }

    let Some(dt) = DateTime::<Utc>::from_timestamp(rounded as i64, 0) else {
        return String::new();
    };

    dt.format(format).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_open_meteo_hourly_timestamp() {
        let timestamp = parse_datetime_value("2026-05-18T00:00").unwrap();
        assert_eq!(
            format_datetime_tick(timestamp, "%Y-%m-%d %H:%M"),
            "2026-05-18 00:00"
        );
    }

    #[test]
    fn parses_compact_hour_interval() {
        assert_eq!(parse_datetime_interval_seconds("20h").unwrap(), 72_000.0);
        assert_eq!(
            parse_datetime_interval_seconds("20 hours").unwrap(),
            72_000.0
        );
    }
}
