//! Global runtime management for RustyRay

use crate::object_store::{InMemoryStore, StoreConfig};
use crate::{ActorSystem, Result, RustyRayError, TaskSystem};
use std::sync::{Arc, RwLock};

static RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);

/// The global RustyRay runtime
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
    object_store: Arc<InMemoryStore>,
}

/// Initialize the global runtime
pub fn init() -> Result<()> {
    Runtime::init_internal()
}

/// Get the global runtime
pub fn global() -> Result<Arc<Runtime>> {
    Runtime::global()
}

/// Shutdown the global runtime
pub fn shutdown() -> Result<()> {
    Runtime::shutdown_internal()
}

/// Shutdown the global runtime with explicit subsystem shutdown
/// This is the preferred method for graceful shutdown in production code
pub async fn shutdown_async() -> Result<()> {
    Runtime::shutdown_async_internal().await
}

/// Check if the runtime is initialized
pub fn is_initialized() -> bool {
    Runtime::is_initialized()
}

impl Runtime {
    /// Initialize the global runtime
    fn init_internal() -> Result<()> {
        let mut runtime_guard = RUNTIME
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Check if already initialized
        if runtime_guard.is_some() {
            return Err(RustyRayError::Internal(
                "Runtime already initialized. Call runtime::shutdown() first to reinitialize"
                    .to_string(),
            ));
        }

        // Create shared object store with default config
        let object_store = Arc::new(InMemoryStore::new(StoreConfig::default()));

        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::with_store(
            actor_system.clone(),
            object_store.clone(),
        ));

        // Register all remote functions
        init_remote_functions(&task_system);

        let runtime = Arc::new(Runtime {
            actor_system,
            task_system,
            object_store,
        });

        *runtime_guard = Some(runtime);
        Ok(())
    }

    /// Get the global runtime
    pub fn global() -> Result<Arc<Runtime>> {
        let runtime_guard = RUNTIME
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runtime_guard.as_ref().cloned().ok_or_else(|| {
            RustyRayError::Internal(
                "Runtime not initialized. Call runtime::init() first or use #[rustyray::main]"
                    .to_string(),
            )
        })
    }

    /// Shutdown the global runtime
    fn shutdown_internal() -> Result<()> {
        // Use unwrap_or_else to handle poisoned locks gracefully
        let mut runtime_guard = RUNTIME
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if runtime_guard.is_none() {
            return Err(RustyRayError::Internal(
                "Runtime not initialized. Nothing to shutdown".to_string(),
            ));
        }

        // Note: We can't easily call async shutdown methods from this synchronous context
        // when already inside a tokio runtime (which happens during tests).
        // The subsystems will be properly cleaned up when dropped due to their
        // internal shutdown states and Drop implementations.
        //
        // The explicit shutdown methods are available for users who want to
        // ensure graceful shutdown with proper async handling.

        // Clear the runtime, dropping all resources
        *runtime_guard = None;

        Ok(())
    }

    /// Shutdown the global runtime with explicit async subsystem shutdown
    async fn shutdown_async_internal() -> Result<()> {
        // Use unwrap_or_else to handle poisoned locks gracefully
        let mut runtime_guard = RUNTIME
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if runtime_guard.is_none() {
            return Err(RustyRayError::Internal(
                "Runtime not initialized. Nothing to shutdown".to_string(),
            ));
        }

        // Get the runtime before clearing the guard
        let runtime = runtime_guard.take().unwrap();
        drop(runtime_guard); // Release the lock

        // Perform explicit shutdown of subsystems in the correct order
        // First shutdown the task system (which depends on actor system)
        if let Err(e) = runtime.task_system.shutdown().await {
            eprintln!("Warning: TaskSystem shutdown error: {e}");
        }

        // Then shutdown the actor system
        if let Err(e) = runtime.actor_system.shutdown().await {
            eprintln!("Warning: ActorSystem shutdown error: {e}");
        }

        // Object store doesn't have explicit shutdown, will be cleaned up on drop
        drop(runtime);

        Ok(())
    }

    /// Check if the runtime is initialized
    fn is_initialized() -> bool {
        let runtime_guard = RUNTIME
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runtime_guard.is_some()
    }

    /// Get the task system
    pub fn task_system(&self) -> &Arc<TaskSystem> {
        &self.task_system
    }

    /// Get the actor system
    pub fn actor_system(&self) -> &Arc<ActorSystem> {
        &self.actor_system
    }

    /// Get the object store
    pub fn object_store(&self) -> &Arc<InMemoryStore> {
        &self.object_store
    }
}

/// Registration for remote functions using linkme
pub struct RemoteFunctionRegistration {
    pub name: &'static str,
    pub register: fn(&TaskSystem),
}

#[linkme::distributed_slice]
pub static REMOTE_FUNCTIONS: [RemoteFunctionRegistration];

/// Initialize all registered remote functions
pub fn init_remote_functions(system: &TaskSystem) {
    for registration in REMOTE_FUNCTIONS {
        (registration.register)(system);
    }
}
