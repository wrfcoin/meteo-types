//! Data quality scoring and classification.

use serde::{Deserialize, Serialize};

/// Quality score for a weather observation or data submission.
///
/// Each component is in [0.0, 1.0] where 1.0 is perfect quality.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DataQualityScore {
    /// Overall quality score (typically a weighted combination of components).
    pub overall: f64,
    /// Score from range validation (are values within physical limits?).
    pub range_score: f64,
    /// Score from temporal/spatial consistency checks.
    pub consistency_score: f64,
}

impl DataQualityScore {
    fn is_component_valid(value: f64) -> bool {
        value.is_finite() && (0.0..=1.0).contains(&value)
    }

    /// Create a new quality score.
    ///
    /// This constructor preserves backward compatibility and asserts preconditions
    /// in debug builds. For explicit error handling, use [`Self::try_new`].
    pub fn new(overall: f64, range_score: f64, consistency_score: f64) -> Self {
        debug_assert!(
            Self::is_component_valid(overall),
            "overall must be finite and in [0, 1]"
        );
        debug_assert!(
            Self::is_component_valid(range_score),
            "range_score must be finite and in [0, 1]"
        );
        debug_assert!(
            Self::is_component_valid(consistency_score),
            "consistency_score must be finite and in [0, 1]"
        );
        Self {
            overall,
            range_score,
            consistency_score,
        }
    }

    /// Create a new quality score with runtime validation.
    pub fn try_new(
        overall: f64,
        range_score: f64,
        consistency_score: f64,
    ) -> Result<Self, &'static str> {
        let score = Self {
            overall,
            range_score,
            consistency_score,
        };
        score.validate()?;
        Ok(score)
    }

    /// Return whether all score components are finite and in [0, 1].
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Validate score components.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !Self::is_component_valid(self.overall) {
            return Err("overall must be finite and in [0, 1]");
        }
        if !Self::is_component_valid(self.range_score) {
            return Err("range_score must be finite and in [0, 1]");
        }
        if !Self::is_component_valid(self.consistency_score) {
            return Err("consistency_score must be finite and in [0, 1]");
        }
        Ok(())
    }

    /// Classify this score into a quality band.
    pub fn band(&self) -> DataQualityBand {
        DataQualityBand::from_score(self.overall)
    }
}

/// Quality classification bands for weather data.
///
/// Thresholds (configurable in production via `DataQualityPolicy`):
/// - High: >= 0.85
/// - Borderline: >= 0.70
/// - WarningOnly: >= 0.50
/// - Low: < 0.50
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DataQualityBand {
    /// High quality — data is reliable and reward-eligible.
    High,
    /// Borderline — data is usable but may receive reduced rewards.
    Borderline,
    /// Warning only — data is questionable, flagged for review.
    WarningOnly,
    /// Low quality — data is unreliable, may be rejected.
    Low,
}

impl DataQualityBand {
    /// Classify an overall quality score into a band using default thresholds.
    pub fn from_score(score: f64) -> Self {
        if score >= 0.85 {
            Self::High
        } else if score >= 0.70 {
            Self::Borderline
        } else if score >= 0.50 {
            Self::WarningOnly
        } else {
            Self::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_classification() {
        assert_eq!(DataQualityBand::from_score(0.95), DataQualityBand::High);
        assert_eq!(
            DataQualityBand::from_score(0.75),
            DataQualityBand::Borderline
        );
        assert_eq!(
            DataQualityBand::from_score(0.60),
            DataQualityBand::WarningOnly
        );
        assert_eq!(DataQualityBand::from_score(0.30), DataQualityBand::Low);
    }

    #[test]
    fn score_to_band() {
        let score = DataQualityScore::new(0.90, 0.95, 0.85);
        assert_eq!(score.band(), DataQualityBand::High);
    }

    #[test]
    fn try_new_rejects_invalid_values() {
        assert_eq!(
            DataQualityScore::try_new(-0.1, 0.8, 0.9),
            Err("overall must be finite and in [0, 1]")
        );
        assert_eq!(
            DataQualityScore::try_new(0.7, 2.0, 0.9),
            Err("range_score must be finite and in [0, 1]")
        );
        assert_eq!(
            DataQualityScore::try_new(0.7, 0.8, f64::NAN),
            Err("consistency_score must be finite and in [0, 1]")
        );
    }

    #[test]
    fn validate_checks_score_components() {
        let good = DataQualityScore::new(0.8, 0.8, 0.8);
        assert!(good.is_valid());

        let bad = DataQualityScore {
            overall: 0.8,
            range_score: 0.8,
            consistency_score: 1.5,
        };
        assert!(!bad.is_valid());
        assert_eq!(
            bad.validate(),
            Err("consistency_score must be finite and in [0, 1]")
        );
    }

    #[test]
    fn boundary_values() {
        assert_eq!(DataQualityBand::from_score(0.85), DataQualityBand::High);
        assert_eq!(
            DataQualityBand::from_score(0.70),
            DataQualityBand::Borderline
        );
        assert_eq!(
            DataQualityBand::from_score(0.50),
            DataQualityBand::WarningOnly
        );
        assert_eq!(DataQualityBand::from_score(0.0), DataQualityBand::Low);
    }
}
