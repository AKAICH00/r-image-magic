//! Printful Provider Module
//!
//! Provides integration with the Printful API for product catalog,
//! variants, print areas, and mockup generation.
//!
//! API Documentation: https://developers.printful.com/docs/

mod client;
mod models;
mod mapper;

pub use client::PrintfulProvider;
