use cron::Schedule as CronSchedule;
use std::str::FromStr;

/// Validated cron schedule value object
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ValidatedSchedule {
    pub expression: String,
    inner: CronSchedule,
}

#[allow(dead_code)]
impl ValidatedSchedule {
    pub fn parse(expr: &str) -> anyhow::Result<Self> {
        let inner = CronSchedule::from_str(expr)
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
}
