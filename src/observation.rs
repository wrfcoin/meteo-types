//! Core weather observation types.

use serde::{Deserialize, Serialize};

/// A weather observation with standard meteorological variables.
///
/// All fields are optional — sensors may not report every variable.
/// Units follow SI / WMO conventions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WeatherObservation {
    /// Air temperature in degrees Celsius.
    pub temperature_c: Option<f64>,
    /// Relative humidity as a percentage [0, 100].
    pub humidity_percent: Option<f64>,
    /// Atmospheric pressure in hectopascals (hPa / mbar).
    pub pressure_hpa: Option<f64>,
    /// Wind speed in meters per second.
    pub wind_speed_ms: Option<f64>,
    /// Wind direction in degrees [0, 360). 0 = North, 90 = East.
    pub wind_direction_deg: Option<f64>,
    /// Precipitation accumulation in millimeters.
    pub precipitation_mm: Option<f64>,
    /// Dewpoint temperature in degrees Celsius.
    pub dewpoint_c: Option<f64>,
    /// Horizontal visibility in meters.
    pub visibility_m: Option<f64>,
}

impl WeatherObservation {
    /// Returns `true` if all present values fall within physically plausible ranges.
    ///
    /// Ranges based on observed Earth records:
    /// - Temperature: -89.2°C (Vostok) to 56.7°C (Death Valley)
    /// - Humidity: 0–100%
    /// - Pressure: 870–1084 hPa (observed extremes)
    /// - Wind speed: 0–113 m/s (strongest recorded gust)
    /// - Wind direction: 0–360°
    /// - Precipitation: 0–500 mm (per observation period)
    /// - Dewpoint: -80°C to 35°C
    /// - Visibility: 0–100,000 m
    pub fn is_physically_plausible(&self) -> bool {
        let checks = [
            self.temperature_c.map(|v| (-89.2..=56.7).contains(&v)),
            self.humidity_percent.map(|v| (0.0..=100.0).contains(&v)),
            self.pressure_hpa.map(|v| (870.0..=1084.0).contains(&v)),
            self.wind_speed_ms.map(|v| (0.0..=113.0).contains(&v)),
            self.wind_direction_deg.map(|v| (0.0..360.0).contains(&v)),
            self.precipitation_mm.map(|v| (0.0..=500.0).contains(&v)),
            self.dewpoint_c.map(|v| (-80.0..=35.0).contains(&v)),
            self.visibility_m.map(|v| (0.0..=100_000.0).contains(&v)),
        ];
        checks.iter().all(|c| c.unwrap_or(true))
    }

    /// Count how many variables are present (non-None).
    pub fn variable_count(&self) -> usize {
        [
            self.temperature_c.is_some(),
            self.humidity_percent.is_some(),
            self.pressure_hpa.is_some(),
            self.wind_speed_ms.is_some(),
            self.wind_direction_deg.is_some(),
            self.precipitation_mm.is_some(),
            self.dewpoint_c.is_some(),
            self.visibility_m.is_some(),
        ]
        .iter()
        .filter(|&&v| v)
        .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let obs = WeatherObservation::default();
        assert_eq!(obs.variable_count(), 0);
        assert!(obs.is_physically_plausible());
    }

    #[test]
    fn plausible_observation() {
        let obs = WeatherObservation {
            temperature_c: Some(22.5),
            humidity_percent: Some(65.0),
            pressure_hpa: Some(1013.25),
            wind_speed_ms: Some(5.0),
            wind_direction_deg: Some(180.0),
            precipitation_mm: Some(0.0),
            dewpoint_c: Some(15.0),
            visibility_m: Some(10_000.0),
        };
        assert!(obs.is_physically_plausible());
        assert_eq!(obs.variable_count(), 8);
    }

    #[test]
    fn implausible_temperature() {
        let obs = WeatherObservation {
            temperature_c: Some(100.0), // impossible
            ..Default::default()
        };
        assert!(!obs.is_physically_plausible());
    }

    #[test]
    fn partial_observation() {
        let obs = WeatherObservation {
            temperature_c: Some(20.0),
            pressure_hpa: Some(1013.0),
            ..Default::default()
        };
        assert_eq!(obs.variable_count(), 2);
        assert!(obs.is_physically_plausible());
    }
}
