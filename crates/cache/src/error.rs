use web_core::error_prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum ExtensionError {
    #[error("Distribute cache missing.")]
    DistributeCacheMissing,
    #[error("Memory cache missing.")]
    MemoryCacheMissing,
}

app_error_impl!(ExtensionError);
