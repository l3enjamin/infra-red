//! `ir-secrets` provides cross-platform OS keyring access (`IR-201`) and
//! the Tier-2 loopback HTTP/SOCKS5 secret-swapping egress proxy (`IR-403`).

use async_trait::async_trait;
use ir_core::SecretProvider;
use zeroize::Zeroizing;

/// Read-only secret provider backed by `IR_SECRET_<NORMALIZED_KEY>` environment variables for CI.
#[derive(Debug, Default, Clone)]
pub struct CiEnvSecretProvider;

#[async_trait]
impl SecretProvider for CiEnvSecretProvider {
    fn provider_name(&self) -> &'static str {
        "ci-env"
    }

    async fn get_secret(&self, _key: &str) -> anyhow::Result<Zeroizing<String>> {
        todo!("IR-201: Lookup IR_SECRET_<KEY> from std::env::var and wrap in Zeroizing<String>")
    }

    async fn set_secret(&self, _key: &str, _value: &str) -> anyhow::Result<()> {
        anyhow::bail!("CiEnvSecretProvider is read-only; set secrets via CI environment variables")
    }
}
