use crate::error::ExtensionError;
use moka::future::Cache;
use ntex::http::{Payload, RequestHead};
use ntex::util::Extensions;
use ntex::web::{FromRequest, HttpRequest, WebRequest};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub mod prelude {
    pub use crate::impls::memory::MemoryCacheExt;
}

pub type MemoryCacheKey = &'static str;
pub type MemoryCacheValue = serde_json::Value;
pub type MemoryCacheGlobal = Arc<RwLock<MemoryCache>>;

pub trait MemoryCacheExt {
    fn memory_cache(&self) -> std::result::Result<MemoryCacheExtension, ExtensionError>;
}

pub struct MemoryCache {
    client: Cache<MemoryCacheKey, MemoryCacheValue>,
}

impl Deref for MemoryCache {
    type Target = Cache<MemoryCacheKey, MemoryCacheValue>;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for MemoryCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub struct MemoryCacheExtension(MemoryCacheGlobal);

impl Deref for MemoryCacheExtension {
    type Target = MemoryCacheGlobal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MemoryCacheExtension {
    #[inline]
    pub fn set_into_req(extensions: &mut Extensions, global: MemoryCacheGlobal) {
        if !extensions.contains::<MemoryCacheExtension>() {
            extensions.insert(MemoryCacheExtension(global))
        }
    }

    #[inline]
    fn get_from_req(extensions: &mut Extensions) -> std::result::Result<Self, ExtensionError> {
        extensions
            .get::<MemoryCacheExtension>()
            .ok_or(ExtensionError::MemoryCacheMissing)
            .map(|ext| MemoryCacheExtension(Arc::clone(&ext.0)))
    }
}

impl<Err> FromRequest<Err> for MemoryCacheExtension {
    type Error = ExtensionError;

    #[inline]
    async fn from_request(req: &HttpRequest, _payload: &mut Payload) -> std::result::Result<Self, Self::Error> {
        MemoryCacheExtension::get_from_req(&mut req.extensions_mut())
    }
}

pub fn generate() -> MemoryCacheGlobal {
    debug!("Generating the memory cache.");

    Arc::new(RwLock::new(MemoryCache {
        client: Cache::builder()
            // Time to live (TTL): 30 minutes
            .time_to_live(Duration::from_secs(30 * 60))
            // Time to idle (TTI):  5 minutes
            .time_to_idle(Duration::from_secs(5 * 60))
            // This cache will hold up to 32MiB of values.
            .max_capacity(32 * 1024 * 1024)
            .build(),
    }))
}

macro_rules! impl_ext {
    ($ident: ident) => {
        impl MemoryCacheExt for $ident {
            #[inline]
            fn memory_cache(&self) -> std::result::Result<MemoryCacheExtension, ExtensionError> {
                MemoryCacheExtension::get_from_req(&mut self.extensions_mut())
            }
        }
    };

    ($ident: ident<$($genetic: tt),+>) => {
        impl<$($genetic)+> MemoryCacheExt for $ident<$($genetic)+> {
            #[inline]
            fn memory_cache(&self) -> std::result::Result<MemoryCacheExtension, ExtensionError> {
                MemoryCacheExtension::get_from_req(&mut self.extensions_mut())
            }
        }
    }
}

impl_ext!(HttpRequest);
impl_ext!(WebRequest<Err>);
impl_ext!(RequestHead);

#[macro_export]
macro_rules! memory_cache_make_sure {
    ($cache: expr, $task: expr) => {
        $task;

        // This make sure what we changed in `$task` will be effective "immediately".
        $cache.run_pending_tasks().await;
    };
}
