#[cfg(feature = "enable-redis")]
use crate::cache::prelude::*;
use std::net::{IpAddr, Ipv4Addr};
use web_core::prelude::*;

#[derive(Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
    #[cfg(feature = "enable-redis")]
    pub distribute_cache_config: DistributeCacheConfig,
}

impl Server {
    pub fn from_env() -> Result<Self> {
        Ok(Server {
            ip: web_env::var_parsed("IP")?.unwrap_or_else(|| Ipv4Addr::UNSPECIFIED.into()),
            port: web_env::var_parsed("PORT")?.unwrap_or_else(|| 9527),
            #[cfg(feature = "enable-redis")]
            distribute_cache_config: DistributeCacheConfig::from_url(
                web_env::required_var_parsed::<String>("REDIS_URI")?.as_str(),
            )?,
        })
    }
}
