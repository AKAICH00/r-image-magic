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

pub mod gelato;
pub mod gooten;
pub mod http_client;
pub mod printful;
pub mod printify;
pub mod spod;
pub mod traits;

// Re-export commonly used types
pub use http_client::RateLimitedClient;
pub use traits::{
    CatalogPage, PodProvider, ProviderCredentials, ProviderError, ProviderFactory, ProviderResult,
};
