pub mod models;
pub mod repository_runtime;
pub mod pool_manager;

// Use runtime repository to avoid compile-time database checks
pub use repository_runtime::Repository;
pub use pool_manager::PoolManager;