//! Gelato Provider Module
//!
//! Provides integration with the Gelato API for product catalog,
//! variants, print areas, and mockup generation.
//!
//! API Documentation: https://docs.gelato.com/reference

mod client;
mod mapper;
mod models;

pub use client::GelatoProvider;
