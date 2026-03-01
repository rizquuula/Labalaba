use cron::Schedule as CronSchedule;
use std::str::FromStr;

/// Validated cron schedule value object
#[derive(Debug, Clone)]
pub struct ValidatedSchedule {
    pub expression: String,
    inner: CronSchedule,
}

impl ValidatedSchedule {
    pub fn parse(expr: &str) -> anyhow::Result<Self> {
        let inner = CronSchedule::from_str(expr)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression '{}': {}", expr, e))?;
        Ok(Self { expression: expr.to_string(), inner })
    }

    /// Returns the next scheduled instant after now
    pub fn next_run(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.inner.upcoming(chrono::Utc).next()
    }
}
