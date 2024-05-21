use std::net::{IpAddr, Ipv4Addr};
use web_cache::prelude::*;
use web_core::prelude::*;

#[derive(Clone)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
    pub distribute_cache_config: DistributeCacheConfig,
    pub async_op_guard_config: web_guard::async_op::AsyncOpGuardConfig,
}

impl Server {
    pub fn from_env() -> Result<Self> {
        let redis_config = web_env::required_var_parsed::<String>("REDIS_URI")?.leak();

        Ok(Server {
            ip: web_env::var_parsed("IP")?.unwrap_or(Ipv4Addr::UNSPECIFIED.into()),
            port: web_env::var_parsed("PORT")?.unwrap_or(9527),
            distribute_cache_config: DistributeCacheConfig::from_url(redis_config)?,
            async_op_guard_config: redis_config,
        })
    }
}
