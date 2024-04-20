use crate::error::ExtensionError;
use fred::prelude::*;
use ntex::{
    http::{Payload, RequestHead},
    util::Extensions,
    web::{FromRequest, HttpRequest, WebRequest},
};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use web_core::prelude::*;

pub mod prelude {
    pub use crate::impls::distribute::DistributeCacheExt;
    pub use fred::prelude::*;
}

pub type DistributeCacheKey = &'static str;
pub type DistributeCacheGlobal = Arc<DistributeCache>;
pub type DistributeCacheConfig = RedisConfig;

pub trait DistributeCacheExt {
    fn distribute_cache(&self) -> std::result::Result<DistributeCacheExtension, ExtensionError>;
}

pub struct DistributeCache {
    client: RedisClient,
}

impl Deref for DistributeCache {
    type Target = RedisClient;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for DistributeCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub struct DistributeCacheExtension(DistributeCacheGlobal);

impl Deref for DistributeCacheExtension {
    type Target = DistributeCacheGlobal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DistributeCacheExtension {
    #[inline]
    pub fn set_into_req(extensions: &mut Extensions, global: DistributeCacheGlobal) {
        if !extensions.contains::<DistributeCacheExtension>() {
            extensions.insert(DistributeCacheExtension(global))
        }
    }

    #[inline]
    fn get_from_req(extensions: &mut Extensions) -> std::result::Result<DistributeCacheExtension, ExtensionError> {
        extensions
            .get::<DistributeCacheExtension>()
            .ok_or(ExtensionError::DistributeCacheMissing)
            .map(|ext| DistributeCacheExtension(Arc::clone(&ext.0)))
    }
}

impl<Err> FromRequest<Err> for DistributeCacheExtension {
    type Error = ExtensionError;

    #[inline]
    async fn from_request(req: &HttpRequest, _payload: &mut Payload) -> std::result::Result<Self, Self::Error> {
        DistributeCacheExtension::get_from_req(&mut req.extensions_mut())
    }
}

pub async fn generate(config: DistributeCacheConfig) -> Result<DistributeCacheGlobal> {
    let client = DistributeCache {
        client: RedisClient::new(
            config,
            Some(PerformanceConfig {
                // 10s.
                default_command_timeout: Duration::from_secs(10),
                ..Default::default()
            }),
            None,
            Some(ReconnectPolicy::new_constant(u32::MAX, 8_000)),
        ),
    };

    debug!("Connecting to the redis cache.");

    // No need to wait for being connected.
    #[allow(clippy::let_underscore_future)]
    let _ = client.connect();

    // No need to use the `?` to wait for being connected.
    let _ = client.wait_for_connect().await;

    Ok(Arc::new(client))
}

macro_rules! impl_ext {
    ($ident: ident) => {
        impl DistributeCacheExt for $ident {
            #[inline]
            fn distribute_cache(&self) -> std::result::Result<DistributeCacheExtension, ExtensionError> {
                DistributeCacheExtension::get_from_req(&mut self.extensions_mut())
            }
        }
    };

    ($ident: ident<$($genetic: tt),+>) => {
        impl<$($genetic)+> DistributeCacheExt for $ident<$($genetic)+> {
            #[inline]
            fn distribute_cache(&self) -> std::result::Result<DistributeCacheExtension, ExtensionError> {
                DistributeCacheExtension::get_from_req(&mut self.extensions_mut())
            }
        }
    }
}

impl_ext!(HttpRequest);
impl_ext!(WebRequest<Err>);
impl_ext!(RequestHead);
