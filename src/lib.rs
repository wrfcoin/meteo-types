//! Canonical Rust types for meteorological and environmental observation data.
//!
//! `meteo-types` provides a shared vocabulary of types for weather and environmental
//! data across applications, filling a gap in the Rust ecosystem. Types are
//! serialization-ready (serde) with no other dependencies.
//!
//! # Modules
//!
//! - [`observation`] — Core weather observation types
//! - [`domain`] — Environmental domain payloads (weather, air quality, water, etc.)
//! - [`quality`] — Data quality scoring and classification
//! - [`geo`] — Geographic location types

pub mod domain;
pub mod geo;
pub mod observation;
pub mod quality;

// Re-export core types at crate root for convenience.
pub use domain::{
    AirQualityPayload, EnvironmentalDomain, EnvironmentalReport, HydrologyPayload, OceanPayload,
    ProvenanceChain, ProvenanceEntry, ReportPayload, SoilPayload, WaterQualityPayload,
    WildfirePayload,
};
pub use geo::GeoLocation;
pub use observation::WeatherObservation;
pub use quality::{DataQualityBand, DataQualityScore};
