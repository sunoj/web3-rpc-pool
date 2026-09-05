// Cooldown for terminated WS endpoints so failover never permanently retires the pool.
// Exports EndpointRetry to ws; tests use injected timestamps without sleeping.
// Deps: std monotonic time and mutex, shared by subscription streams.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const ENDPOINT_RETRY_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(super) struct EndpointRetry {
    failed_at: Mutex<Option<Instant>>,
}

impl EndpointRetry {
    pub(super) fn failed(&self) {
        *self.failed_at.lock().expect("WS endpoint retry lock") = Some(Instant::now());
    }

    pub(super) fn cooling_down(&self) -> bool {
        self.cooling_down_at(Instant::now())
    }

    fn cooling_down_at(&self, now: Instant) -> bool {
        self.failed_at
            .lock()
            .expect("WS endpoint retry lock")
            .is_some_and(|failed| now.saturating_duration_since(failed) < ENDPOINT_RETRY_COOLDOWN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminated_endpoint_becomes_eligible_again_after_cooldown() {
        let endpoint = EndpointRetry::default();
        assert!(!endpoint.cooling_down());
        endpoint.failed();
        let failed = endpoint.failed_at.lock().unwrap().unwrap();
        assert!(endpoint.cooling_down_at(failed + Duration::from_secs(29)));
        assert!(!endpoint.cooling_down_at(failed + ENDPOINT_RETRY_COOLDOWN));
    }

    #[test]
    fn another_endpoint_remains_eligible_during_failover() {
        let failed = EndpointRetry::default();
        let healthy = EndpointRetry::default();
        failed.failed();
        assert!(failed.cooling_down());
        assert!(!healthy.cooling_down());
    }
}
