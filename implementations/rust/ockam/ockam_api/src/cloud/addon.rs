use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

use ockam_core::CowStr;
#[cfg(feature = "tag")]
use ockam_core::TypeTag;

#[derive(Encode, Decode, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Clone))]
#[cbor(map)]
pub struct Addon<'a> {
    #[cfg(feature = "tag")]
    #[serde(skip)]
    #[n(0)]
    pub tag: TypeTag<1530077>,
    #[b(1)]
    #[serde(borrow)]
    pub id: CowStr<'a>,
    #[b(2)]
    #[serde(borrow)]
    pub description: CowStr<'a>,
    #[n(3)]
    pub enabled: bool,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug)]
#[rustfmt::skip]
#[cbor(map)]
pub struct ConfluentConfig<'a> {
    #[cfg(feature = "tag")]
    #[serde(skip)]
    #[cbor(n(0))] pub tag: TypeTag<1697996>,

    #[serde(borrow)]
    #[cbor(b(1))] pub bootstrap_server: CowStr<'a>,
}

impl<'a> ConfluentConfig<'a> {
    pub fn new<S: Into<CowStr<'a>>>(bootstrap_server: S) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            bootstrap_server: bootstrap_server.into(),
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
#[rustfmt::skip]
#[cbor(map)]
pub struct ConfluentConfigResponse {
    #[cfg(feature = "tag")]
    #[serde(skip)]
    #[cbor(n(0))] pub tag: TypeTag<6434816>,

    #[cbor(n(1))] pub bootstrap_server: String,
}

impl ConfluentConfigResponse {
    pub fn new<S: ToString>(bootstrap_server: S) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            bootstrap_server: bootstrap_server.to_string(),
        }
    }
}

mod node {
    use std::time::Duration;

    use minicbor::{Decode, Decoder, Encode};
    use tracing::trace;

    use ockam_core::api::Request;
    use ockam_core::{self, Result};
    use ockam_node::Context;

    use crate::cloud::addon::ConfluentConfig;
    use crate::cloud::project::{InfluxDBTokenLeaseManagerConfig, OktaConfig};
    use crate::cloud::{
        BareCloudRequestWrapper, CloudRequestWrapper, ORCHESTRATOR_RESTART_TIMEOUT,
    };
    use crate::error::ApiError;
    use crate::nodes::NodeManagerWorker;

    const TARGET: &str = "ockam_api::cloud::addon";
    const API_SERVICE: &str = "projects";

    impl NodeManagerWorker {
        pub(crate) async fn list_addons(
            &mut self,
            ctx: &mut Context,
            dec: &mut Decoder<'_>,
            project_id: &str,
        ) -> Result<Vec<u8>> {
            let req_wrapper: BareCloudRequestWrapper = dec.decode()?;
            let cloud_multiaddr = req_wrapper.multiaddr()?;

            let label = "list_addons";
            trace!(target: TARGET, project_id, "listing addons");

            let req_builder = Request::get(format!("/v0/{project_id}/addons"));

            self.request_controller(
                ctx,
                label,
                None,
                &cloud_multiaddr,
                API_SERVICE,
                req_builder,
                None,
            )
            .await
        }

        pub(crate) async fn configure_addon(
            &mut self,
            ctx: &mut Context,
            dec: &mut Decoder<'_>,
            project_id: &str,
            addon_id: &str,
        ) -> Result<Vec<u8>> {
            // TODO: Add on ids should not be magic strings
            match addon_id {
                "okta" => {
                    self.configure_addon_impl::<OktaConfig>(ctx, dec, project_id, addon_id)
                        .await
                }
                "influxdb_token_lease_manager" => {
                    self.configure_addon_impl::<InfluxDBTokenLeaseManagerConfig>(
                        ctx, dec, project_id, addon_id,
                    )
                    .await
                }
                "confluent" => {
                    self.configure_addon_impl::<ConfluentConfig>(ctx, dec, project_id, addon_id)
                        .await
                }
                _ => Err(ApiError::generic(&format!("Unknown addon: {addon_id}"))),
            }
        }

        async fn configure_addon_impl<'a, T: Encode<()> + Decode<'a, ()>>(
            &mut self,
            ctx: &mut Context,
            dec: &mut Decoder<'a>,
            project_id: &str,
            addon_id: &str,
        ) -> Result<Vec<u8>> {
            let label = "configure_addon";
            trace!(target: TARGET, project_id, addon_id, "configuring addon");

            let req_wrapper: CloudRequestWrapper<T> = dec.decode()?;
            let cloud_multiaddr = req_wrapper.multiaddr()?;
            let req_body = req_wrapper.req;

            let req_builder =
                Request::put(format!("/v0/{project_id}/addons/{addon_id}")).body(req_body);

            self.request_controller_with_timeout(
                ctx,
                label,
                None,
                &cloud_multiaddr,
                API_SERVICE,
                req_builder,
                None,
                Duration::from_secs(ORCHESTRATOR_RESTART_TIMEOUT),
            )
            .await
        }

        pub(crate) async fn disable_addon(
            &mut self,
            ctx: &mut Context,
            dec: &mut Decoder<'_>,
            project_id: &str,
            addon_id: &str,
        ) -> Result<Vec<u8>> {
            let req_wrapper: BareCloudRequestWrapper = dec.decode()?;
            let cloud_multiaddr = req_wrapper.multiaddr()?;

            let label = "disable_addon";
            trace!(target: TARGET, project_id, addon_id, "disabling addon");

            let req_builder = Request::delete(format!("/v0/{project_id}/addons/{addon_id}"));

            self.request_controller(
                ctx,
                label,
                None,
                &cloud_multiaddr,
                API_SERVICE,
                req_builder,
                None,
            )
            .await
        }
    }
}
