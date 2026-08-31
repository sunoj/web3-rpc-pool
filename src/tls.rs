//! Process-global rustls `CryptoProvider` installation.
//!
//! Exports: [`ensure_provider`].
//!
//! This crate builds `reqwest` with `rustls-no-provider`, so **the library must
//! install a provider itself** — see the reasoning below.

/// Installs the ring-backed rustls `CryptoProvider` as the process default,
/// exactly once. Idempotent, thread-safe, and cheap to call repeatedly.
///
/// # Why the library does this instead of the caller
///
/// `reqwest` 0.13 offers only two rustls options:
///
/// * `rustls` — selects `aws-lc-rs`, which pulls `aws-lc-sys`, a C/asm build
///   that is hostile to cross-compilation (we cross-compile macOS → Linux).
/// * `rustls-no-provider` — no provider; **the process must install one before
///   the first `Client` is built, or `Client::builder().build()` panics.**
///
/// So this crate takes `rustls-no-provider` to stay cross-compile clean, and
/// then installs `ring` here. Requiring *callers* to do it would be a breaking
/// change: `RpcPool::new` builds a `reqwest::Client` internally, so every
/// existing consumer would start panicking at pool construction. `liquidation-engine`
/// in particular documents that its provider comes from this crate.
///
/// Note that reqwest 0.13 also **removed** `rustls-tls-webpki-roots`; both
/// remaining options use `rustls-platform-verifier` (the OS trust store), so
/// hosts need their CA bundle present. That is the same path smart-router's
/// services already use in production.
pub fn ensure_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // Errs only when a provider is already installed — e.g. a consumer
        // installed one in `main`, or another library got here first. Either is
        // fine; we only need *a* provider to exist.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Returns a process-wide HTTP client for direct-pool health probes.
pub(crate) fn probe_client() -> reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            ensure_provider();
            reqwest::Client::builder()
                .pool_max_idle_per_host(1)
                .pool_idle_timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build health-check HTTP client")
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::{ensure_provider, probe_client};

    #[test]
    fn is_idempotent_and_concurrency_safe() {
        let handles: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(ensure_provider))
            .collect();
        for h in handles {
            h.join().expect("ensure_provider must not panic");
        }
        ensure_provider();
        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "a default CryptoProvider must be installed"
        );
    }

    /// The regression this module exists to prevent: building a client without
    /// a caller-installed provider must work, not panic.
    #[test]
    fn client_builds_without_caller_installing_a_provider() {
        ensure_provider();
        reqwest::Client::builder()
            .build()
            .expect("client must build once the provider is installed");
    }

    #[test]
    fn probe_client_is_reusable() {
        let first = probe_client();
        let second = probe_client();
        drop((first, second));
    }
}
