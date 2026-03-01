/// Log collection is handled inline in start_task.rs via tokio::spawn.
/// This module provides utilities for log buffer management.
use std::collections::VecDeque;
use labalaba_shared::api::LogEntry;

/// A fixed-size circular buffer for recent log lines.
#[allow(dead_code)]
pub struct LogBuffer {
    max_lines: usize,
    lines: VecDeque<LogEntry>,
}

#[allow(dead_code)]
impl LogBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self { max_lines, lines: VecDeque::with_capacity(max_lines) }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(entry);
    }

    pub fn recent(&self, n: usize) -> Vec<LogEntry> {
        self.lines.iter().rev().take(n).cloned().collect::<Vec<_>>()
            .into_iter().rev().collect()
    }
}
