//! Test utilities for RustyRay tests
//!
//! This module provides common test utilities and helper functions to make
//! writing tests easier and more consistent.
//!
//! # Test Execution and Concurrency
//!
//! RustyRay's runtime FULLY SUPPORTS concurrent execution of actors and tasks - this is
//! fundamental to Ray's design and is thoroughly tested. However, our test suite requires
//! careful handling of the global runtime to ensure test isolation.
//!
//! ## Important Points:
//!
//! 1. **The runtime is concurrent** - Multiple actors and tasks can and do run concurrently
//!    within a single runtime instance. This is verified by our concurrent operation tests.
//!
//! 2. **Tests need isolation** - Since we use a global runtime (like Ray), tests that
//!    initialize/shutdown the runtime need to be isolated from each other.
//!
//! 3. **Test fixtures serialize runtime lifecycle** - The `with_test_runtime` fixture uses
//!    a mutex to ensure only one test at a time is initializing or shutting down the runtime.
//!    This prevents test interference.
//!
//! 4. **Within each test, everything is concurrent** - Once a test has the runtime, it can
//!    spawn as many concurrent actors and tasks as needed.
//!
//! ## Running Tests
//!
//! For best results, run tests with:
//! ```bash
//! cargo test -- --test-threads=1
//! ```
//!
//! This ensures tests don't interfere with each other's runtime initialization/shutdown.
//! This is NOT because the runtime doesn't support concurrency - it's purely for test isolation.

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Notify;

// Track if a test is currently using the runtime
static TEST_RUNTIME_IN_USE: AtomicBool = AtomicBool::new(false);
static CONCURRENT_TEST_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

/// A test synchronization helper that provides notification and event tracking.
///
/// This is useful for coordinating between test code and async operations,
/// especially when testing actor lifecycle events like `on_stop`.
pub struct TestSync {
    notification: Arc<Notify>,
}

impl TestSync {
    /// Create a new test synchronization helper
    pub fn new() -> Self {
        Self {
            notification: Arc::new(Notify::new()),
        }
    }

    /// Get a notification handle that can be shared with actors or tasks
    pub fn notification() -> Arc<Notify> {
        Arc::new(Notify::new())
    }

    /// Notify any waiters
    pub fn notify(&self) {
        self.notification.notify_one();
    }

    /// Wait for notification
    pub async fn wait(&self) {
        self.notification.notified().await;
    }
}

impl Default for TestSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the test runtime. This is now a no-op as tests use with_test_runtime.
#[deprecated(note = "Use with_test_runtime instead")]
pub fn init_test_runtime() {
    // No-op for backward compatibility
}

/// Clear runtime state. This is now a no-op as tests use with_test_runtime.
#[deprecated(note = "Use with_test_runtime instead")]
pub async fn clear_runtime_state() {
    // No-op for backward compatibility
}

/// Run a test with a fresh runtime instance
///
/// This fixture ensures that:
/// - Each test gets a fresh runtime instance
/// - Runtime is properly initialized before the test
/// - Runtime is shutdown after the test (even if it panics)
/// - Tests are isolated from each other
///
/// # Example
///
/// ```rust
/// #[tokio::test]
/// async fn test_something() {
///     with_test_runtime(|| async {
///         // Your test code here
///         let obj_ref = ray::put(42).await?;
///         let value = ray::get(obj_ref).await?;
///         assert_eq!(value, 42);
///     }).await;
/// }
/// ```
pub async fn with_test_runtime<F, Fut>(test_body: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    // Check if another test is trying to use the runtime concurrently
    if TEST_RUNTIME_IN_USE.swap(true, Ordering::SeqCst) {
        CONCURRENT_TEST_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
        panic!(
            "\n\n\
            ========================================\n\
            CONCURRENT TEST EXECUTION DETECTED!\n\
            ========================================\n\
            \n\
            Multiple tests are trying to use the global runtime concurrently.\n\
            This is not supported due to test isolation requirements.\n\
            \n\
            Please run tests with: cargo test -- --test-threads=1\n\
            \n\
            Alternatively, set the environment variable: RUST_TEST_THREADS=1\n\
            \n\
            Note: The runtime itself DOES support concurrent actors/tasks.\n\
            This restriction only applies to test execution.\n\
            ========================================\n"
        );
    }

    // Use a mutex as a secondary protection
    static TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _guard = TEST_MUTEX.lock().await;

    // Helper to perform cleanup
    async fn cleanup() {
        if let Ok(runtime) = crate::runtime::global() {
            let task_system = runtime.task_system().clone();
            let actor_system = runtime.actor_system().clone();
            let _ = task_system.shutdown().await;
            let _ = actor_system.shutdown().await;
        }
        let _ = crate::runtime::shutdown();
    }

    // Ensure clean state before test
    if crate::runtime::is_initialized() {
        cleanup().await;
    }

    // Initialize runtime for this test
    crate::runtime::init().expect("Failed to initialize runtime for test");

    // Create a wrapper future that handles panics
    use futures::FutureExt;
    let test_future = std::panic::AssertUnwindSafe(test_body()).catch_unwind();

    // Run the test with panic protection
    let result = test_future.await;

    // Always cleanup, even if test panicked
    cleanup().await;

    // Release the runtime for the next test
    TEST_RUNTIME_IN_USE.store(false, Ordering::SeqCst);

    // Re-panic if test failed
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

/// Run a test with a fresh runtime instance (blocking version)
///
/// This is useful for non-async tests that still need runtime access.
///
/// # Example
///
/// ```rust,no_run
/// #[test]
/// fn test_something_sync() {
///     with_test_runtime_blocking(|| {
///         // Your test code here
///         let runtime = runtime::global().unwrap();
///         // ...
///     });
/// }
/// ```
pub fn with_test_runtime_blocking<F>(test_body: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    // Check if another test is trying to use the runtime concurrently
    if TEST_RUNTIME_IN_USE.swap(true, Ordering::SeqCst) {
        CONCURRENT_TEST_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
        panic!(
            "\n\n\
            ========================================\n\
            CONCURRENT TEST EXECUTION DETECTED!\n\
            ========================================\n\
            \n\
            Multiple tests are trying to use the global runtime concurrently.\n\
            This is not supported due to test isolation requirements.\n\
            \n\
            Please run tests with: cargo test -- --test-threads=1\n\
            \n\
            Alternatively, set the environment variable: RUST_TEST_THREADS=1\n\
            \n\
            Note: The runtime itself DOES support concurrent actors/tasks.\n\
            This restriction only applies to test execution.\n\
            ========================================\n"
        );
    }

    // Use a mutex as a secondary protection
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = TEST_MUTEX.lock().unwrap();

    // Create a tokio runtime for async cleanup
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Ensure clean state before test
    if crate::runtime::is_initialized() {
        // Get the runtime to shut down its systems properly
        if let Ok(runtime) = crate::runtime::global() {
            let task_system = runtime.task_system().clone();
            let actor_system = runtime.actor_system().clone();
            rt.block_on(async move {
                let _ = task_system.shutdown().await;
                let _ = actor_system.shutdown().await;
            });
        }
        let _ = crate::runtime::shutdown();

        // Small delay to ensure cleanup completes
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Initialize runtime for this test
    crate::runtime::init().expect("Failed to initialize runtime for test");

    // Run test with panic protection
    let result = std::panic::catch_unwind(test_body);

    // Always cleanup, even if test panicked
    if let Ok(runtime) = crate::runtime::global() {
        let task_system = runtime.task_system().clone();
        let actor_system = runtime.actor_system().clone();
        rt.block_on(async move {
            let _ = task_system.shutdown().await;
            let _ = actor_system.shutdown().await;
        });
    }
    let _ = crate::runtime::shutdown();

    // Release the runtime for the next test
    TEST_RUNTIME_IN_USE.store(false, Ordering::SeqCst);

    // Re-panic if test failed
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
