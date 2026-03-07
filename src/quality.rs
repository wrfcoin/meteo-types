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
    /// Create a new quality score.
    pub fn new(overall: f64, range_score: f64, consistency_score: f64) -> Self {
        Self {
            overall,
            range_score,
            consistency_score,
        }
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
