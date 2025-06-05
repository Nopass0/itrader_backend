pub mod client;
pub mod p2p;
pub mod auth;
pub mod models;
pub mod python_bridge;
pub mod rate_fetcher;
pub mod python_rate_fetcher;

// Include both implementations
pub mod p2p_python;

// Use Python-based implementation by default with feature flag
#[cfg(feature = "python-sdk")]
pub use p2p_python::BybitP2PClient;

// Use native implementation if python-sdk feature is not enabled
#[cfg(not(feature = "python-sdk"))]
pub use p2p::BybitP2PClient;

pub use client::BybitClient;
pub use models::{AccountInfo, Advertisement, P2POrder, ChatMessage, PaymentMethod, 
                 ServerTimeResponse, BybitResponse, AdParams, CreateOrderRequest, 
                 SendMessageRequest, ReleaseOrderRequest, AdvertisementsResponse, 
                 OrdersResponse, OrderChat};
pub use python_bridge::PyBybitClient;
pub use rate_fetcher::{BybitRateFetcher, RateScenario, P2PTradeItem, P2PTradeResponse, P2PTradeResult};
pub use python_rate_fetcher::PythonRateFetcher;