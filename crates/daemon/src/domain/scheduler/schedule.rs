use cron::Schedule as CronSchedule;
use std::str::FromStr;

/// Validated cron schedule value object.
///
/// Accepts both 5-field standard cron (`min hour dom month dow`) and the
/// 6-field form required by the `cron` crate (`sec min hour dom month dow`).
/// 5-field input is normalized by prepending a `0` seconds field before
/// parsing; `self.expression` always stores the ORIGINAL input string.
/// Day-of-week follows the `cron` crate's interpretation; no DOW remapping
/// is performed here.
#[derive(Debug, Clone)]
pub struct ValidatedSchedule {
    pub expression: String,
    inner: CronSchedule,
}

/// Normalize a cron expression from 5-field to 6-field format.
///
/// The `cron` crate requires 6 fields (`sec min hour dom month dow`).
/// Classic cron uses 5 fields (`min hour dom month dow`). If `expr` has
/// exactly 5 whitespace-delimited fields, `"0 "` is prepended so the seconds
/// field is fixed at zero. Any other field count is returned unchanged.
pub(crate) fn normalize_cron(expr: &str) -> String {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    }
}

impl ValidatedSchedule {
    pub fn parse(expr: &str) -> anyhow::Result<Self> {
        let normalized = normalize_cron(expr);
        let inner = CronSchedule::from_str(&normalized)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", expr, e))?;
        Ok(Self {
            expression: expr.to_string(),
            inner,
        })
    }

    /// Returns the next scheduled instant after now
    pub fn next_run(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.inner.upcoming(chrono::Utc).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_every_minute() {
        let schedule = ValidatedSchedule::parse("* * * * * *").unwrap();
        assert_eq!(schedule.expression, "* * * * * *");
        assert!(schedule.next_run().is_some());
    }

    #[test]
    fn test_parse_valid_hourly() {
        let schedule = ValidatedSchedule::parse("0 0 * * * *").unwrap();
        assert_eq!(schedule.expression, "0 0 * * * *");
        let next = schedule.next_run().unwrap();
        assert!(next > chrono::Utc::now());
    }

    #[test]
    fn test_parse_valid_daily() {
        let schedule = ValidatedSchedule::parse("0 0 0 * * *").unwrap();
        assert_eq!(schedule.expression, "0 0 0 * * *");
        let next = schedule.next_run().unwrap();
        assert!(next > chrono::Utc::now());
    }

    #[test]
    fn test_parse_valid_weekly() {
        let schedule = ValidatedSchedule::parse("0 0 0 * * 7").unwrap();
        assert_eq!(schedule.expression, "0 0 0 * * 7");
        let next = schedule.next_run().unwrap();
        assert!(next > chrono::Utc::now());
    }

    #[test]
    fn test_parse_valid_specific_time() {
        let schedule = ValidatedSchedule::parse("30 14 0 * * 1-5").unwrap();
        assert_eq!(schedule.expression, "30 14 0 * * 1-5");
        let next = schedule.next_run().unwrap();
        assert!(next > chrono::Utc::now());
    }

    #[test]
    fn test_parse_invalid_cron_too_few_fields() {
        let result = ValidatedSchedule::parse("* * * *");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid cron expression"));
    }

    #[test]
    fn test_parse_invalid_cron_out_of_range() {
        let result = ValidatedSchedule::parse("99 99 99 * * *");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_cron_malformed() {
        let result = ValidatedSchedule::parse("not a valid cron");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_cron_empty() {
        let result = ValidatedSchedule::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_next_run_always_in_future() {
        let schedule = ValidatedSchedule::parse("* * * * * *").unwrap();
        let next = schedule.next_run().unwrap();
        let now = chrono::Utc::now();
        assert!(next > now);
        // Should be within the next 60 seconds (for second-level cron)
        assert!(next < now + chrono::Duration::seconds(60));
    }

    #[test]
    fn test_expression_preserved() {
        let expressions = vec!["*/5 * * * * *", "0 0 1 * * *", "0 12 0 * * 1,3,5"];

        for expr in expressions {
            let schedule = ValidatedSchedule::parse(expr).unwrap();
            assert_eq!(schedule.expression, expr);
        }
    }

    // New tests for 5-field support

    #[test]
    fn test_5field_every_5_minutes_parses() {
        let result = ValidatedSchedule::parse("*/5 * * * *");
        assert!(result.is_ok(), "5-field '*/5 * * * *' should parse OK");
        let schedule = result.unwrap();
        assert_eq!(schedule.expression, "*/5 * * * *");
        assert!(schedule.next_run().is_some());
    }

    #[test]
    fn test_5field_weekdays_9am_parses() {
        let result = ValidatedSchedule::parse("0 9 * * 1-5");
        assert!(result.is_ok(), "5-field '0 9 * * 1-5' should parse OK");
        let schedule = result.unwrap();
        assert_eq!(schedule.expression, "0 9 * * 1-5");
        assert!(schedule.next_run().is_some());
    }

    #[test]
    fn test_6field_still_parses() {
        let result = ValidatedSchedule::parse("0 */5 * * * *");
        assert!(result.is_ok(), "6-field should still parse");
    }

    #[test]
    fn test_4field_still_errors() {
        let result = ValidatedSchedule::parse("* * * *");
        assert!(result.is_err(), "4-field should still error");
    }

    #[test]
    fn test_empty_still_errors() {
        let result = ValidatedSchedule::parse("");
        assert!(result.is_err(), "empty string should still error");
    }

    // Direct normalize_cron tests

    #[test]
    fn normalize_5field_prepends_zero_seconds() {
        let result = normalize_cron("*/5 * * * *");
        assert_eq!(result, "0 */5 * * * *");
    }

    #[test]
    fn normalize_6field_unchanged() {
        let expr = "0 */5 * * * *";
        let result = normalize_cron(expr);
        assert_eq!(result, expr);
    }
}
