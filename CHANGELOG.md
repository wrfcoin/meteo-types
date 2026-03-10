# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-09

### Added
- Core weather observation type (`WeatherObservation`) with physical plausibility checks (scaffold)
- Geographic location type (`GeoLocation`) with WGS 84 validation (scaffold)
- Data quality scoring (`DataQualityScore`, `DataQualityBand`) with band classification (scaffold)
- Environmental domain payloads for 7 domains: Weather, AirQuality, WaterQuality, Wildfire, Soil, Ocean, Hydrology (scaffold)
- `EnvironmentalReport` with domain/payload consistency validation (scaffold)
- `#[non_exhaustive]` on public enums for forward compatibility (#7)
- `DataQualityScore::new()` returning `Result<Self, DataQualityError>` for typed validation (#7)
- `DataQualityScore::new_unchecked()` for trusted internal paths (#7)
- `WeatherPayload` consolidated as type alias for `WeatherObservation` (#8)
- `ProvenanceChain` and `ProvenanceEntry` types on `EnvironmentalReport` (#8)
- `EnvironmentalReport::new()` constructor with auto-derived domain (#9)
- IoT sensor accuracy verification metrics (`SensorAccuracyMetrics`, `AccuracyWindow`) (#10)
- `ts-rs` TypeScript binding generation behind `ts` feature flag (#11)
- `repository` field in Cargo.toml
- `#[must_use]` annotations on validation methods
- Doc comments on `ProvenanceChain::is_empty()` and `ProvenanceChain::len()`

### Breaking Changes
- `#[non_exhaustive]` added to `DataQualityError` — downstream `match` arms require a wildcard
- `#[non_exhaustive]` on `EnvironmentalDomain`, `ReportPayload`, `DataQualityBand` (added in #7)
- `DataQualityScore::new()` now returns `Result` instead of constructing directly (added in #7)
