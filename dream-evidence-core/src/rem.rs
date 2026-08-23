// src/rem.rs
//! Módulo de janela REM — validação de estimulação

use crate::{types::*, Result};

#[derive(Debug, thiserror::Error)]
pub enum RemWindowError {
    #[error("Invalid REM window: start {0} >= end {1}")]
    InvalidWindow(Timestamp, Timestamp),
    #[error("REM window duration {0} not in [{1}, {2}]")]
    InvalidDuration(u64, u64, u64),
    #[error("REM window overlaps with previous")]
    OverlappingWindow,
    #[error("REM window end in the future: {0} > now + skew")]
    EndInFuture(Timestamp),
}

#[derive(Debug, Clone, Default)]
pub struct RemWindow {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

impl RemWindow {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    pub fn update(
        &mut self,
        start: Timestamp,
        end: Timestamp,
        now: Timestamp,
        config: &Config,
    ) -> Result<()> {
        if start >= end {
            return Err(RemWindowError::InvalidWindow(start, end).into());
        }

        let duration = end - start;
        if duration < config.min_rem_duration || duration > config.max_rem_duration {
            return Err(RemWindowError::InvalidDuration(
                duration,
                config.min_rem_duration,
                config.max_rem_duration,
            ).into());
        }

        if let Some(prev_end) = self.end {
            if start < prev_end {
                return Err(RemWindowError::OverlappingWindow.into());
            }
        }

        if end > now + config.max_clock_skew {
            return Err(RemWindowError::EndInFuture(end).into());
        }

        self.start = Some(start);
        self.end = Some(end);

        Ok(())
    }

    pub fn is_active(&self, ts: Timestamp) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => ts >= start && ts <= end,
            _ => false,
        }
    }

    pub fn start(&self) -> Option<Timestamp> {
        self.start
    }

    pub fn end(&self) -> Option<Timestamp> {
        self.end
    }

    pub fn is_defined(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}