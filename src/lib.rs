pub mod api;
pub mod core;
pub mod utils;
pub mod gate;
pub mod bybit;
pub mod ai;
pub mod ocr;
// pub mod email;
pub mod db;
pub mod gmail;

pub use core::config::Config;
pub use utils::error::{AppError, Result};