//! Endpoint construction for the real network.

use anyhow::{Context, Result};
use iroh::{Endpoint, RelayMode, RelayUrl, SecretKey, endpoint::presets};

/// Binds an endpoint on the public iroh infrastructure: n0 DNS/pkarr so peers can be
/// dialed by id alone, and n0 relays as a fallback when hole punching fails.
/// `PAPO_RELAY` points at a self-hosted `iroh-relay` instead.
pub async fn bind_endpoint(secret_key: SecretKey) -> Result<Endpoint> {
    #[cfg(feature = "test-network")]
    if let Some(endpoint) = local::bind_from_env(&secret_key).await? {
        return Ok(endpoint);
    }
    let mut builder = Endpoint::builder(presets::N0).secret_key(secret_key);
    if let Some(url) = relay_override(std::env::var("PAPO_RELAY").ok().as_deref())? {
        builder = builder.relay_mode(RelayMode::Custom(url.into()));
    }
    builder.bind().await.context("bind iroh endpoint")
}

/// `PAPO_RELAY` as a relay URL. Unset, empty or blank means the public relays: shells
/// and container env files often export a variable with no value.
fn relay_override(value: Option<&str>) -> Result<Option<RelayUrl>> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(url) => url.parse().map(Some).context("PAPO_RELAY is not a valid URL"),
    }
}

/// Hermetic network for end-to-end tests of the real binary, compiled only with the
/// `test-network` feature so release builds never contain it.
///
/// `PAPO_TEST_RELAY` is the URL of a local relay (self-signed, hence the skipped
/// certificate check) and `PAPO_TEST_PEERS` a comma-separated list of endpoint ids
/// reachable through it. Address lookup is a static in-memory table, so nothing
/// touches public DNS or relays.
#[cfg(feature = "test-network")]
mod local {
    use anyhow::{Context, Result};
    use iroh::{
        Endpoint, EndpointAddr, EndpointId, RelayMode, RelayUrl, SecretKey, address_lookup::memory::MemoryLookup,
        endpoint::presets, tls::CaTlsConfig,
    };

    pub(super) async fn bind_from_env(secret_key: &SecretKey) -> Result<Option<Endpoint>> {
        let Ok(relay) = std::env::var("PAPO_TEST_RELAY") else {
            return Ok(None);
        };
        let relay: RelayUrl = relay.parse().context("PAPO_TEST_RELAY is not a valid URL")?;
        let peers = std::env::var("PAPO_TEST_PEERS").unwrap_or_default();
        let lookup = MemoryLookup::new();
        for id in peers.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let id: EndpointId = id.parse().context("PAPO_TEST_PEERS has an invalid endpoint id")?;
            lookup.add_endpoint_info(EndpointAddr::new(id).with_relay_url(relay.clone()));
        }
        let endpoint = Endpoint::builder(presets::Minimal)
            .secret_key(secret_key.clone())
            .relay_mode(RelayMode::Custom(relay.into()))
            .ca_tls_config(CaTlsConfig::insecure_skip_verify())
            .bind()
            .await
            .context("bind test endpoint")?;
        endpoint.address_lookup().context("endpoint closed while binding")?.add(lookup);
        Ok(Some(endpoint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_override_treats_empty_as_unset_and_rejects_garbage() {
        assert!(relay_override(None).unwrap().is_none());
        assert!(relay_override(Some("")).unwrap().is_none());
        assert!(relay_override(Some("  \t")).unwrap().is_none());
        let url = relay_override(Some(" https://relay.example.com ")).unwrap().unwrap();
        assert_eq!(url.to_string(), "https://relay.example.com/");
        assert!(relay_override(Some("not a url")).unwrap_err().to_string().contains("PAPO_RELAY"));
    }
}
