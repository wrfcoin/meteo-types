//! Physics validation types for meteorological data quality checking.
//!
//! These types mirror the validation logic in the WRFCoin blockchain's
//! `shared-protocol/src/weather/validation.rs` but omit blockchain-specific
//! fields (e.g., `activation_height`).

use serde::{Deserialize, Serialize};

/// Configurable bounds for physics-plausibility checks on weather readings.
///
/// Default bounds are based on observed Earth records (WMO extremes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bindings/"))]
pub struct PhysicsRangePolicy {
    /// Minimum plausible temperature in degrees Celsius (world record low: −89.2 °C).
    pub temp_min_c: f64,
    /// Maximum plausible temperature in degrees Celsius (world record high: 70.7 °C).
    pub temp_max_c: f64,
    /// Minimum plausible barometric pressure in hPa (Typhoon Tip: 870 hPa).
    pub pressure_min_hpa: f64,
    /// Maximum plausible barometric pressure in hPa (Siberian High: 1085 hPa).
    pub pressure_max_hpa: f64,
    /// Minimum relative humidity in percent.
    pub humidity_min_pct: f64,
    /// Maximum relative humidity in percent.
    pub humidity_max_pct: f64,
    /// Maximum plausible wind speed in m/s (world record gust: 113.3 m/s).
    pub wind_speed_max_ms: f64,
    /// Maximum physically plausible temperature rate of change in °C per minute.
    pub temp_rate_max_c_per_min: f64,
}

impl Default for PhysicsRangePolicy {
    fn default() -> Self {
        Self {
            temp_min_c: -89.2,
            temp_max_c: 70.7,
            pressure_min_hpa: 870.0,
            pressure_max_hpa: 1085.0,
            humidity_min_pct: 0.0,
            humidity_max_pct: 100.0,
            wind_speed_max_ms: 113.3,
            temp_rate_max_c_per_min: 5.0,
        }
    }
}

/// A physics violation detected during weather reading validation.
///
/// Serialized as an internally tagged union with `"type"` as the discriminator,
/// matching the TypeScript discriminated union pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bindings/"))]
pub enum PhysicsViolation {
    #[serde(rename_all = "camelCase")]
    TemperatureOutOfRange { value_c: f64, min: f64, max: f64 },
    #[serde(rename_all = "camelCase")]
    PressureOutOfRange { value_hpa: f64, min: f64, max: f64 },
    #[serde(rename_all = "camelCase")]
    HumidityOutOfRange { value_pct: f64, min: f64, max: f64 },
    #[serde(rename_all = "camelCase")]
    WindSpeedExceedsMaximum { value_ms: f64, max: f64 },
    #[serde(rename_all = "camelCase")]
    TemperatureRateExceeded { delta_c: f64, limit: f64 },
    #[serde(rename_all = "camelCase")]
    CrossParameterInconsistency { detail: String },
}

/// A minimal weather reading used as input for physics validation.
///
/// Separate from [`crate::WeatherObservation`] — this carries only the subset of
/// fields needed for physics range-checking plus a timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export, export_to = "bindings/"))]
pub struct ValidationReading {
    /// Air temperature in degrees Celsius.
    pub temperature_c: Option<f64>,
    /// Atmospheric pressure in hectopascals (hPa).
    pub pressure_hpa: Option<f64>,
    /// Relative humidity as a percentage [0, 100].
    pub humidity_pct: Option<f64>,
    /// Wind speed in meters per second.
    pub wind_speed_ms: Option<f64>,
    /// Unix timestamp of the reading (seconds since epoch).
    pub timestamp_unix: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_matches_wmo_extremes() {
        let policy = PhysicsRangePolicy::default();
        assert_eq!(policy.temp_min_c, -89.2);
        assert_eq!(policy.temp_max_c, 70.7);
        assert_eq!(policy.pressure_min_hpa, 870.0);
        assert_eq!(policy.pressure_max_hpa, 1085.0);
        assert_eq!(policy.humidity_min_pct, 0.0);
        assert_eq!(policy.humidity_max_pct, 100.0);
        assert_eq!(policy.wind_speed_max_ms, 113.3);
        assert_eq!(policy.temp_rate_max_c_per_min, 5.0);
    }

    #[test]
    fn policy_serde_roundtrip() {
        let policy = PhysicsRangePolicy::default();
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: PhysicsRangePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy, deserialized);
    }

    #[test]
    fn policy_serializes_camel_case() {
        let policy = PhysicsRangePolicy::default();
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("tempMinC"));
        assert!(json.contains("pressureMinHpa"));
        assert!(json.contains("windSpeedMaxMs"));
        assert!(json.contains("tempRateMaxCPerMin"));
        // Should NOT contain snake_case
        assert!(!json.contains("temp_min_c"));
    }

    #[test]
    fn violation_serde_roundtrip() {
        let violations = vec![
            PhysicsViolation::TemperatureOutOfRange {
                value_c: 100.0,
                min: -89.2,
                max: 70.7,
            },
            PhysicsViolation::PressureOutOfRange {
                value_hpa: 500.0,
                min: 870.0,
                max: 1085.0,
            },
            PhysicsViolation::HumidityOutOfRange {
                value_pct: 120.0,
                min: 0.0,
                max: 100.0,
            },
            PhysicsViolation::WindSpeedExceedsMaximum {
                value_ms: 200.0,
                max: 113.3,
            },
            PhysicsViolation::TemperatureRateExceeded {
                delta_c: 15.0,
                limit: 5.0,
            },
            PhysicsViolation::CrossParameterInconsistency {
                detail: "Dew point exceeds temperature".into(),
            },
        ];
        for v in &violations {
            let json = serde_json::to_string(v).unwrap();
            let deserialized: PhysicsViolation = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, deserialized);
        }
    }

    #[test]
    fn violation_serializes_tagged_camel_case() {
        let v = PhysicsViolation::TemperatureOutOfRange {
            value_c: 100.0,
            min: -89.2,
            max: 70.7,
        };
        let json = serde_json::to_string(&v).unwrap();
        // Tagged with "type"
        assert!(json.contains(r#""type":"TemperatureOutOfRange""#));
        // Fields are camelCase
        assert!(json.contains("valueC"));
        assert!(!json.contains("value_c"));
    }

    #[test]
    fn reading_serde_roundtrip() {
        let reading = ValidationReading {
            temperature_c: Some(22.5),
            pressure_hpa: Some(1013.25),
            humidity_pct: Some(65.0),
            wind_speed_ms: Some(5.0),
            timestamp_unix: 1700000000.0,
        };
        let json = serde_json::to_string(&reading).unwrap();
        let deserialized: ValidationReading = serde_json::from_str(&json).unwrap();
        assert_eq!(reading, deserialized);
    }

    #[test]
    fn reading_serializes_camel_case() {
        let reading = ValidationReading {
            temperature_c: Some(22.5),
            pressure_hpa: None,
            humidity_pct: None,
            wind_speed_ms: None,
            timestamp_unix: 1000.0,
        };
        let json = serde_json::to_string(&reading).unwrap();
        assert!(json.contains("temperatureC"));
        assert!(json.contains("timestampUnix"));
        assert!(!json.contains("temperature_c"));
    }

    #[test]
    fn default_reading_is_empty() {
        let reading = ValidationReading::default();
        assert_eq!(reading.temperature_c, None);
        assert_eq!(reading.pressure_hpa, None);
        assert_eq!(reading.humidity_pct, None);
        assert_eq!(reading.wind_speed_ms, None);
        assert_eq!(reading.timestamp_unix, 0.0);
    }
}
