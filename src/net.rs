//! Endpoint construction for the real network.

use anyhow::{Context, Result};
use iroh::{Endpoint, RelayMode, RelayUrl, SecretKey, endpoint::presets};

/// Binds an endpoint on the public iroh infrastructure: n0 DNS/pkarr so peers can be
/// dialed by id alone, and n0 relays as a fallback when hole punching fails.
/// `PAPO_RELAY` points at a self-hosted `iroh-relay` instead.
pub async fn bind_endpoint(secret_key: SecretKey) -> Result<Endpoint> {
    let mut builder = Endpoint::builder(presets::N0).secret_key(secret_key);
    if let Ok(url) = std::env::var("PAPO_RELAY") {
        let url: RelayUrl = url.parse().context("PAPO_RELAY is not a valid URL")?;
        builder = builder.relay_mode(RelayMode::Custom(url.into()));
    }
    builder.bind().await.context("bind iroh endpoint")
}
