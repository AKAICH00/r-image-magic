//! Printify Provider Module
//!
//! Provides integration with the Printify API for product catalog,
//! variants, print areas, and mockup generation.
//!
//! API Documentation: https://developers.printify.com/docs/api

mod client;
mod mapper;
mod models;

pub use client::PrintifyProvider;
