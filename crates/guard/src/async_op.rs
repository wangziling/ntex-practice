/// Async op guard - Redis Lock.
/// Make sure that only one process here on concurrent env now.
use rslock::{LockError, LockManager};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

pub type AsyncOpGuardGlobal = Arc<AsyncOpGuard>;
pub type AsyncOpGuardConfig = &'static str;

pub struct AsyncOpGuard {
    inner: LockManager,
}

impl AsyncOpGuard {
    fn new(config: AsyncOpGuardConfig) -> Self {
        AsyncOpGuard { inner: LockManager::new(vec![config]) }
    }

    /// Acquire a lock.
    /// May be stuck eternally.
    pub async fn spawn_acquire<F>(&self, resource: &[u8], ttl: usize, async_task: F) -> F::Output
    where
        F: Future,
        F::Output: Send + Sync,
    {
        // May be stuck.
        let lock = self.acquire(resource, ttl).await;
        let result = async_task.await;
        drop(lock);

        result
    }

    pub async fn spawn<F>(&self, resource: &[u8], ttl: usize, async_task: F) -> Result<F::Output, LockError>
    where
        F: Future,
        F::Output: Send + Sync,
    {
        let lock = self.lock(resource, ttl).await?;
        let result = async_task.await;
        self.unlock(&lock).await;

        Ok(result)
    }
}

impl Deref for AsyncOpGuard {
    type Target = LockManager;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub fn generate_async_op_guard(uri: AsyncOpGuardConfig) -> AsyncOpGuardGlobal {
    Arc::new(AsyncOpGuard::new(uri))
}
