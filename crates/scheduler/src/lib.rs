//! Deterministic, pure SRS scheduler (SM-2).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Result of a single card review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOutcome {
    pub card_id: i64,
    pub passed: bool,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct CardState {
    efactor: f32,
    interval: u32, // days
}

impl Default for CardState {
    fn default() -> Self {
        Self {
            efactor: 2.5,
            interval: 1,
        }
    }
}

pub struct Scheduler {
    db: data::DbPool,
    outcome_tx: tokio::sync::mpsc::Sender<ReviewOutcome>,
}

impl Scheduler {
    pub fn new(
        db: data::DbPool,
        outcome_tx: tokio::sync::mpsc::Sender<ReviewOutcome>,
    ) -> Self {
        Self { db, outcome_tx }
    }

    /// Periodically scans for due cards and notifies the UI layer.
    pub async fn run(self) {
        loop {
            let due_cards = data::fetch_due_cards(&self.db, Utc::now())
                .unwrap_or_default();

            if !due_cards.is_empty() {
                debug!("{} cards due", due_cards.len());
                // TODO: IPC → UI notification; placeholder emits debug.
            }

            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }

    /// Evaluate the next interval for a card given its state and pass/fail.
    pub fn next(state: &mut CardState, passed: bool) -> Duration {
        if !passed {
            state.interval = 1;
            return Duration::days(1);
        }

        // We only record pass/fail; treat pass as quality 5 in SM-2.
        let quality = 5.0;
        let ef = state.efactor + 0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02);
        state.efactor = ef.max(1.3);

        state.interval = match state.interval {
            1 => 6,
            i => (i as f32 * state.efactor).round() as u32,
        };
        Duration::days(state.interval as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sm2_interval_progression() {
        let mut st = CardState::default();
        // First pass: 1 → 6 days
        assert_eq!(Scheduler::next(&mut st, true).num_days(), 6);
        let next = Scheduler::next(&mut st, true).num_days();
        assert!(next > 6);
    }
} 