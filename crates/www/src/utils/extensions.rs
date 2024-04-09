// pub trait RequestPersistentCache {
//     fn persistent_cache(&self) -> &PersistentCacheExtension;
// }

// impl<T: ntex::http::HttpMessage> RequestPersistentCache for T {
//     fn persistent_cache(&self) -> &PersistentCacheExtension {
//         self.message_extensions().get::<PersistentCacheExtension>().expect(crate::error::ExtensionError::PersistentCacheMissing.to_string())
//     }
// }

// pub trait RequestDistributeCache {
//     fn distribute_cache(&self) -> &DistributeCacheExtension;
// }

// impl<T: ntex::http::HttpMessage> RequestDistributeCache for T {
//     fn distribute_cache(&self) -> &DistributeCacheExtension {
//         self.message_extensions().get::<DistributeCacheExtension>().expect(crate::error::ExtensionError::DistributeCacheMissing.to_string())
//     }
// }
