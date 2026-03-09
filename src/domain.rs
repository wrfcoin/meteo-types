//! Environmental domain payloads covering 7 observational domains.
//!
//! Each domain has a typed payload struct with SI / standard units.
//! The [`ReportPayload`] enum wraps all domains for tagged serialization.

use crate::geo::GeoLocation;
use serde::{Deserialize, Serialize};

/// Environmental observation domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EnvironmentalDomain {
    Weather,
    AirQuality,
    WaterQuality,
    Wildfire,
    Soil,
    Ocean,
    Hydrology,
}

/// Standard weather payload (surface observations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherPayload {
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub wind_speed_ms: Option<f64>,
    pub wind_direction_deg: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub visibility_m: Option<f64>,
}

/// Air quality payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirQualityPayload {
    pub aqi: Option<u16>,
    pub pm25_ugm3: Option<f64>,
    pub pm10_ugm3: Option<f64>,
    pub o3_ppb: Option<f64>,
    pub no2_ppb: Option<f64>,
    pub so2_ppb: Option<f64>,
    pub co_ppm: Option<f64>,
}

/// Water quality payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterQualityPayload {
    pub ph: Option<f64>,
    pub dissolved_oxygen_mgl: Option<f64>,
    pub turbidity_ntu: Option<f64>,
    pub conductivity_usm: Option<f64>,
    pub temperature_c: Option<f64>,
    pub total_dissolved_solids_mgl: Option<f64>,
}

/// Wildfire observation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WildfirePayload {
    pub fire_radiative_power_mw: Option<f64>,
    pub burn_area_km2: Option<f64>,
    pub confidence_pct: Option<f64>,
    pub smoke_aod: Option<f64>,
    pub containment_pct: Option<f64>,
    pub active: Option<bool>,
}

/// Soil observation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilPayload {
    pub moisture_pct: Option<f64>,
    pub temperature_c: Option<f64>,
    pub ph: Option<f64>,
    pub organic_carbon_pct: Option<f64>,
    pub nitrogen_mgl: Option<f64>,
    pub phosphorus_mgl: Option<f64>,
    pub potassium_mgl: Option<f64>,
    pub conductivity_dsm: Option<f64>,
    pub depth_cm: Option<f64>,
}

/// Ocean observation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OceanPayload {
    pub sea_surface_temperature_c: Option<f64>,
    pub salinity_psu: Option<f64>,
    pub wave_height_m: Option<f64>,
    pub wave_period_s: Option<f64>,
    pub current_speed_ms: Option<f64>,
    pub current_direction_deg: Option<f64>,
    pub chlorophyll_a_mgl: Option<f64>,
    pub dissolved_oxygen_mgl: Option<f64>,
    pub depth_m: Option<f64>,
}

/// Hydrology observation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HydrologyPayload {
    pub river_discharge_m3s: Option<f64>,
    pub water_level_m: Option<f64>,
    pub flow_velocity_ms: Option<f64>,
    pub sediment_load_mgl: Option<f64>,
    pub groundwater_level_m: Option<f64>,
    pub reservoir_storage_pct: Option<f64>,
    pub flood_risk_level: Option<u8>,
    pub drought_index: Option<f64>,
}

/// Tagged union of all environmental domain payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data")]
#[non_exhaustive]
pub enum ReportPayload {
    Weather(WeatherPayload),
    AirQuality(AirQualityPayload),
    WaterQuality(WaterQualityPayload),
    Wildfire(WildfirePayload),
    Soil(SoilPayload),
    Ocean(OceanPayload),
    Hydrology(HydrologyPayload),
}

impl ReportPayload {
    /// Return the environmental domain of this payload.
    pub fn domain(&self) -> EnvironmentalDomain {
        match self {
            Self::Weather(_) => EnvironmentalDomain::Weather,
            Self::AirQuality(_) => EnvironmentalDomain::AirQuality,
            Self::WaterQuality(_) => EnvironmentalDomain::WaterQuality,
            Self::Wildfire(_) => EnvironmentalDomain::Wildfire,
            Self::Soil(_) => EnvironmentalDomain::Soil,
            Self::Ocean(_) => EnvironmentalDomain::Ocean,
            Self::Hydrology(_) => EnvironmentalDomain::Hydrology,
        }
    }
}

/// A complete environmental observation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalReport {
    /// Unique report identifier.
    pub report_id: String,
    /// Environmental domain.
    pub domain: EnvironmentalDomain,
    /// Station or sensor identifier.
    pub station_id: String,
    /// Geographic location of the observation.
    pub location: GeoLocation,
    /// Unix timestamp when the observation was taken.
    pub observed_at: u64,
    /// Unix timestamp when the report was submitted.
    pub submitted_at: u64,
    /// Domain-specific observation data.
    pub payload: ReportPayload,
    /// Optional quality score [0.0, 1.0].
    pub quality_score: Option<f64>,
}

impl EnvironmentalReport {
    /// Construct a report with validation on domain/payload consistency.
    pub fn new(
        report_id: String,
        domain: EnvironmentalDomain,
        station_id: String,
        location: GeoLocation,
        observed_at: u64,
        submitted_at: u64,
        payload: ReportPayload,
        quality_score: Option<f64>,
    ) -> Result<Self, &'static str> {
        let report = Self {
            report_id,
            domain,
            station_id,
            location,
            observed_at,
            submitted_at,
            payload,
            quality_score,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validate report consistency and field ranges.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.domain != self.payload.domain() {
            return Err("environmental report domain does not match payload domain");
        }
        if let Some(score) = self.quality_score {
            if !score.is_finite() || !(0.0..=1.0).contains(&score) {
                return Err("quality_score must be finite and in [0, 1]");
            }
        }
        Ok(())
    }

    /// Return whether report consistency checks pass.
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_payload_domain() {
        let payload = ReportPayload::Weather(WeatherPayload {
            temperature_c: Some(22.0),
            humidity_pct: Some(65.0),
            pressure_hpa: Some(1013.25),
            wind_speed_ms: None,
            wind_direction_deg: None,
            precipitation_mm: None,
            visibility_m: None,
        });
        assert_eq!(payload.domain(), EnvironmentalDomain::Weather);
    }

    #[test]
    fn report_payload_serde_roundtrip() {
        let payload = ReportPayload::AirQuality(AirQualityPayload {
            aqi: Some(42),
            pm25_ugm3: Some(12.5),
            pm10_ugm3: None,
            o3_ppb: Some(30.0),
            no2_ppb: None,
            so2_ppb: None,
            co_ppm: None,
        });
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: ReportPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, deserialized);
    }

    #[test]
    fn environmental_report_construction() {
        let report = EnvironmentalReport {
            report_id: "rpt-001".into(),
            domain: EnvironmentalDomain::Ocean,
            station_id: "buoy-42".into(),
            location: GeoLocation::with_altitude(25.0, -90.0, 0.0),
            observed_at: 1709856000,
            submitted_at: 1709856060,
            payload: ReportPayload::Ocean(OceanPayload {
                sea_surface_temperature_c: Some(26.5),
                salinity_psu: Some(35.0),
                wave_height_m: Some(1.2),
                wave_period_s: Some(8.0),
                current_speed_ms: None,
                current_direction_deg: None,
                chlorophyll_a_mgl: None,
                dissolved_oxygen_mgl: None,
                depth_m: Some(0.0),
            }),
            quality_score: Some(0.92),
        };
        assert_eq!(report.domain, EnvironmentalDomain::Ocean);
        assert!(report.location.is_valid());
        assert!(report.is_valid());
    }

    #[test]
    fn environmental_report_new_rejects_domain_mismatch() {
        let result = EnvironmentalReport::new(
            "rpt-002".into(),
            EnvironmentalDomain::Weather,
            "station-1".into(),
            GeoLocation::new(40.0, -105.0),
            1709856000,
            1709856060,
            ReportPayload::Ocean(OceanPayload {
                sea_surface_temperature_c: Some(21.0),
                salinity_psu: Some(33.0),
                wave_height_m: None,
                wave_period_s: None,
                current_speed_ms: None,
                current_direction_deg: None,
                chlorophyll_a_mgl: None,
                dissolved_oxygen_mgl: None,
                depth_m: None,
            }),
            Some(0.7),
        );
        assert_eq!(
            result,
            Err("environmental report domain does not match payload domain")
        );
    }

    #[test]
    fn environmental_report_new_rejects_invalid_quality_score() {
        let result = EnvironmentalReport::new(
            "rpt-003".into(),
            EnvironmentalDomain::Weather,
            "station-1".into(),
            GeoLocation::new(40.0, -105.0),
            1709856000,
            1709856060,
            ReportPayload::Weather(WeatherPayload {
                temperature_c: Some(18.0),
                humidity_pct: Some(50.0),
                pressure_hpa: None,
                wind_speed_ms: None,
                wind_direction_deg: None,
                precipitation_mm: None,
                visibility_m: None,
            }),
            Some(1.2),
        );
        assert_eq!(result, Err("quality_score must be finite and in [0, 1]"));
    }

    #[test]
    fn environmental_report_new_accepts_matching_domain() {
        let report = EnvironmentalReport::new(
            "rpt-004".into(),
            EnvironmentalDomain::Weather,
            "station-1".into(),
            GeoLocation::new(40.0, -105.0),
            1709856000,
            1709856060,
            ReportPayload::Weather(WeatherPayload {
                temperature_c: Some(18.0),
                humidity_pct: Some(50.0),
                pressure_hpa: None,
                wind_speed_ms: None,
                wind_direction_deg: None,
                precipitation_mm: None,
                visibility_m: None,
            }),
            Some(0.95),
        )
        .unwrap();
        assert!(report.is_valid());
    }
}
