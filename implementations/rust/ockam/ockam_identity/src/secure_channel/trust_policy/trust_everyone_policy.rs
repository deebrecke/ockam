use crate::secure_channel::trust_policy::{SecureChannelTrustInfo, TrustPolicy};
use ockam_core::async_trait;
use ockam_core::compat::boxed::Box;
use ockam_core::{allow, Result};

/// Trust any participant
#[derive(Clone)]
pub struct TrustEveryonePolicy;

#[async_trait]
impl TrustPolicy for TrustEveryonePolicy {
    async fn check(&self, _trust_info: &SecureChannelTrustInfo) -> Result<bool> {
        allow()
    }
}
