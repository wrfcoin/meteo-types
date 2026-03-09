# meteo-types

Canonical Rust types for meteorological and environmental observation data.

`meteo-types` provides a shared vocabulary of strongly-typed structs for weather and environmental
data. It fills a gap in the Rust ecosystem: there is no standard crate for representing weather
observations, air quality readings, hydrological measurements, and other environmental data.

**Minimal dependencies** -- only `serde` for serialization. No runtime, no async, no allocator
tricks.

## Modules

| Module | Types | Purpose |
|--------|-------|---------|
| `observation` | `WeatherObservation` | Surface weather with 8 standard variables (temperature, humidity, pressure, wind, precipitation, dewpoint, visibility) |
| `domain` | `WeatherPayload` (alias of `WeatherObservation`), `AirQualityPayload`, `WaterQualityPayload`, `WildfirePayload`, `SoilPayload`, `OceanPayload`, `HydrologyPayload`, `ReportPayload`, `EnvironmentalReport` | Typed payloads for 7 environmental domains with tagged serialization |
| `quality` | `DataQualityScore`, `DataQualityBand` | Quality scoring and classification (High / Borderline / WarningOnly / Low) |
| `geo` | `GeoLocation` | WGS 84 coordinates with optional altitude and validation |

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
meteo-types = "0.1"
```

### Create a weather observation

```rust
use meteo_types::WeatherObservation;

let obs = WeatherObservation {
    temperature_c: Some(22.5),
    humidity_percent: Some(65.0),
    pressure_hpa: Some(1013.25),
    wind_speed_ms: Some(5.0),
    ..Default::default()
};

assert!(obs.is_physically_plausible());
assert_eq!(obs.variable_count(), 4);
```

### Build a multi-domain environmental report

```rust
use meteo_types::{EnvironmentalReport, ReportPayload, EnvironmentalDomain, GeoLocation};
use meteo_types::domain::OceanPayload;

let report = EnvironmentalReport::new(
    "rpt-001".into(),
    EnvironmentalDomain::Ocean,
    "buoy-42".into(),
    GeoLocation::with_altitude(25.0, -90.0, 0.0),
    1709856000,
    1709856060,
    ReportPayload::Ocean(OceanPayload {
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
    Some(0.92),
).unwrap();
assert!(report.is_valid());
```

### Classify data quality

```rust
use meteo_types::{DataQualityScore, DataQualityBand};

let score = DataQualityScore::try_new(0.90, 0.95, 0.85).unwrap();
assert_eq!(score.band(), DataQualityBand::High);
assert!(score.is_valid());
```

## Design principles

- **All fields are `Option`** -- sensors may not report every variable. Partial observations are
  first-class citizens.
- **SI / WMO units** -- temperature in Celsius, pressure in hPa, wind in m/s, precipitation in mm.
  Units are encoded in field names to prevent misinterpretation.
- **Physics-based validation** -- `is_physically_plausible()` checks values against observed Earth
  records (e.g., -89.2 C to 56.7 C for temperature).
- **Tagged serialization** -- `ReportPayload` uses `#[serde(tag = "domain", content = "data")]`
  for clean JSON like `{"domain": "AirQuality", "data": {...}}`.
- **Validated constructors** -- `DataQualityScore::try_new()` and `EnvironmentalReport::new()`
  enforce score ranges and domain/payload consistency.
- **No runtime coupling** -- types are pure data. No database, no network, no async. Use them in
  embedded systems, web backends, CLI tools, or blockchain nodes.

## Environmental domains

| Domain | Payload struct | Key variables |
|--------|---------------|---------------|
| Weather | `WeatherPayload` | Temperature, humidity, pressure, wind, precipitation, dewpoint, visibility |
| Air Quality | `AirQualityPayload` | AQI, PM2.5, PM10, O3, NO2, SO2, CO |
| Water Quality | `WaterQualityPayload` | pH, dissolved oxygen, turbidity, conductivity, TDS |
| Wildfire | `WildfirePayload` | Fire radiative power, burn area, smoke AOD, containment |
| Soil | `SoilPayload` | Moisture, temperature, pH, organic carbon, NPK, conductivity |
| Ocean | `OceanPayload` | SST, salinity, wave height/period, currents, chlorophyll-a |
| Hydrology | `HydrologyPayload` | River discharge, water level, flow velocity, flood risk, drought index |

## WRFCoin context

This crate was extracted from the [WRFCoin](https://github.com/wrfcoin) blockchain platform, which
rewards contributors for weather data collection and forecast verification. Within WRFCoin, these
types serve as the canonical representation for weather data flowing through the consensus pipeline.

See the [Rust crate audit table](https://github.com/wrfcoin/core4/blob/main/docs/plans/wrfcoin_rust_crate_audit_table.md)
for the full extraction plan and related crates (`forecast-score`, `wrf-primitives`, `station-registry`, etc.).

Tracking issue: [wrfcoin/core4#543](https://github.com/wrfcoin/core4/issues/543)

## Known issues

See [#1](https://github.com/wrfcoin/meteo-types/issues/1) for overall code review context.
With this change, all major items from that review are addressed.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
