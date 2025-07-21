//! Global runtime management for RustyRay

use crate::object_store::{InMemoryStore, StoreConfig};
use crate::{ActorSystem, Result, RustyRayError, TaskSystem};
use once_cell::sync::OnceCell;
use std::sync::Arc;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

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
pub fn global() -> Result<&'static Runtime> {
    Runtime::global()
}

/// Shutdown the global runtime
pub fn shutdown() -> Result<()> {
    Runtime::shutdown_internal()
}

impl Runtime {
    /// Initialize the global runtime
    fn init_internal() -> Result<()> {
        // Create shared object store with default config
        let object_store = Arc::new(InMemoryStore::new(StoreConfig::default()));

        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::with_store(
            actor_system.clone(),
            object_store.clone(),
        ));

        // Register all remote functions
        init_remote_functions(&task_system);

        let runtime = Runtime {
            actor_system,
            task_system,
            object_store,
        };

        RUNTIME.set(runtime).map_err(|_| {
            RustyRayError::Internal(
                "Runtime already initialized. Call runtime::init() only once per process"
                    .to_string(),
            )
        })?;

        Ok(())
    }

    /// Get the global runtime
    pub fn global() -> Result<&'static Runtime> {
        RUNTIME.get().ok_or_else(|| {
            RustyRayError::Internal(
                "Runtime not initialized. Call runtime::init() first or use #[rustyray::main]"
                    .to_string(),
            )
        })
    }

    /// Shutdown the global runtime
    fn shutdown_internal() -> Result<()> {
        // In the future, this will properly shutdown all systems
        Ok(())
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
