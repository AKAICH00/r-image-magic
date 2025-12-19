//! POD (Print-on-Demand) Provider Integration Module
//!
//! This module provides a unified interface for integrating with multiple
//! print-on-demand providers like Printful, Printify, Gelato, SPOD, and Gooten.
//!
//! # Architecture
//!
//! ```text
//!                    ┌─────────────────────┐
//!                    │   PodProvider Trait │
//!                    └──────────┬──────────┘
//!                               │
//!     ┌─────────────┬──────────┼──────────┬──────────────┐
//!     │             │          │          │              │
//! ┌───┴───┐   ┌─────┴──┐  ┌────┴───┐  ┌───┴──┐   ┌──────┴─┐
//! │Printful│  │Printify│  │ Gelato │  │ SPOD │   │ Gooten │
//! └───────┘   └────────┘  └────────┘  └──────┘   └────────┘
//! ```

pub mod traits;
pub mod http_client;
pub mod printful;

// Re-export commonly used types
pub use traits::{
    PodProvider,
    ProviderError,
    ProviderResult,
    CatalogPage,
    ProviderCredentials,
    ProviderFactory,
};
pub use http_client::RateLimitedClient;
