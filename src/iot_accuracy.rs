//! IoT sensor accuracy verification metrics.
//!
//! Types for representing how accurate an IoT sensor's readings are compared to
//! reference values (official weather stations or consensus of nearby sensors).
//! Used in the WRFCoin reward pipeline where more accurate sensors earn higher rewards.

use serde::{Deserialize, Serialize};

use crate::quality::DataQualityBand;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors returned by IoT accuracy metric validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoTAccuracyError {
    /// The MAE value is not finite or is negative.
    InvalidMae,
    /// The RMSE value is not finite or is negative.
    InvalidRmse,
    /// The bias value is not finite.
    InvalidBias,
    /// The correlation value is not finite or not in \[-1.0, 1.0\].
    InvalidCorrelation,
    /// The accuracy score is not finite or not in \[0.0, 1.0\].
    InvalidAccuracyScore,
    /// The sample count is zero.
    ZeroSampleCount,
}

impl core::fmt::Display for IoTAccuracyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidMae => write!(f, "mae must be finite and non-negative"),
            Self::InvalidRmse => write!(f, "rmse must be finite and non-negative"),
            Self::InvalidBias => write!(f, "bias must be finite"),
            Self::InvalidCorrelation => write!(f, "correlation must be finite and in [-1, 1]"),
            Self::InvalidAccuracyScore => {
                write!(f, "accuracy_score must be finite and in [0, 1]")
            }
            Self::ZeroSampleCount => write!(f, "sample count must be greater than zero"),
        }
    }
}

impl std::error::Error for IoTAccuracyError {}

// ---------------------------------------------------------------------------
// SensorReferencePair
// ---------------------------------------------------------------------------

/// A single sensor reading paired with a reference (ground truth) value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorReferencePair {
    /// Value reported by the IoT sensor.
    pub sensor_value: f64,
    /// Reference value from an official station or consensus.
    pub reference_value: f64,
}

impl SensorReferencePair {
    /// Create a new sensor/reference pair.
    pub fn new(sensor_value: f64, reference_value: f64) -> Self {
        Self {
            sensor_value,
            reference_value,
        }
    }

    /// Signed error (sensor minus reference). Positive means sensor reads high.
    pub fn error(&self) -> f64 {
        self.sensor_value - self.reference_value
    }

    /// Absolute error.
    pub fn abs_error(&self) -> f64 {
        (self.sensor_value - self.reference_value).abs()
    }
}

// ---------------------------------------------------------------------------
// SensorAccuracyMetrics
// ---------------------------------------------------------------------------

/// Accuracy metrics for an IoT sensor compared against reference observations.
///
/// All error metrics use the same units as the underlying measurement (e.g.,
/// degrees C for temperature sensors, hPa for pressure sensors).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorAccuracyMetrics {
    /// Mean Absolute Error. Non-negative.
    pub mae: f64,
    /// Root Mean Squared Error. Non-negative.
    pub rmse: f64,
    /// Mean signed bias (sensor - reference). Positive means sensor reads high.
    pub bias: f64,
    /// Pearson correlation coefficient. In \[-1.0, 1.0\].
    pub correlation: f64,
    /// Number of comparison pairs used to compute these metrics.
    pub sample_count: u32,
    /// Composite accuracy score in \[0.0, 1.0\] where 1.0 is perfect.
    pub accuracy_score: f64,
}

impl SensorAccuracyMetrics {
    /// Compute accuracy metrics from a slice of sensor/reference pairs.
    ///
    /// Returns [`IoTAccuracyError::ZeroSampleCount`] if `pairs` is empty.
    pub fn from_pairs(pairs: &[SensorReferencePair]) -> Result<Self, IoTAccuracyError> {
        if pairs.is_empty() {
            return Err(IoTAccuracyError::ZeroSampleCount);
        }

        let n = pairs.len() as f64;

        // Means
        let sum_abs_err: f64 = pairs.iter().map(|p| p.abs_error()).sum();
        let sum_sq_err: f64 = pairs.iter().map(|p| p.error() * p.error()).sum();
        let sum_err: f64 = pairs.iter().map(|p| p.error()).sum();

        let mae = sum_abs_err / n;
        let rmse = (sum_sq_err / n).sqrt();
        let bias = sum_err / n;

        // Reference statistics for correlation and accuracy_score
        let mean_ref: f64 = pairs.iter().map(|p| p.reference_value).sum::<f64>() / n;
        let mean_sen: f64 = pairs.iter().map(|p| p.sensor_value).sum::<f64>() / n;

        let var_ref: f64 =
            pairs.iter().map(|p| (p.reference_value - mean_ref).powi(2)).sum::<f64>() / n;
        let var_sen: f64 =
            pairs.iter().map(|p| (p.sensor_value - mean_sen).powi(2)).sum::<f64>() / n;
        let cov: f64 = pairs
            .iter()
            .map(|p| (p.sensor_value - mean_sen) * (p.reference_value - mean_ref))
            .sum::<f64>()
            / n;

        let std_ref = var_ref.sqrt();
        let std_sen = var_sen.sqrt();

        // Pearson correlation — 0.0 for degenerate (zero variance) cases
        let correlation = if std_ref > 0.0 && std_sen > 0.0 {
            (cov / (std_ref * std_sen)).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Accuracy score: skill-score formula (same as forecast-score crate)
        let accuracy_score = if std_ref > 0.0 {
            (1.0 - rmse / std_ref).clamp(0.0, 1.0)
        } else if rmse == 0.0 {
            1.0
        } else {
            0.0
        };

        Ok(Self {
            mae,
            rmse,
            bias,
            correlation,
            sample_count: pairs.len() as u32,
            accuracy_score,
        })
    }

    /// Validate that all fields are finite and within expected ranges.
    #[must_use = "validation errors must be handled"]
    pub fn validate(&self) -> Result<(), IoTAccuracyError> {
        if !self.mae.is_finite() || self.mae < 0.0 {
            return Err(IoTAccuracyError::InvalidMae);
        }
        if !self.rmse.is_finite() || self.rmse < 0.0 {
            return Err(IoTAccuracyError::InvalidRmse);
        }
        if !self.bias.is_finite() {
            return Err(IoTAccuracyError::InvalidBias);
        }
        if !self.correlation.is_finite() || !(-1.0..=1.0).contains(&self.correlation) {
            return Err(IoTAccuracyError::InvalidCorrelation);
        }
        if !self.accuracy_score.is_finite() || !(0.0..=1.0).contains(&self.accuracy_score) {
            return Err(IoTAccuracyError::InvalidAccuracyScore);
        }
        if self.sample_count == 0 {
            return Err(IoTAccuracyError::ZeroSampleCount);
        }
        Ok(())
    }

    /// Classify this accuracy into a [`SensorAccuracyBand`].
    pub fn band(&self) -> SensorAccuracyBand {
        SensorAccuracyBand::from_score(self.accuracy_score)
    }

    /// Map this accuracy to the equivalent [`DataQualityBand`].
    pub fn quality_band(&self) -> DataQualityBand {
        self.band().to_quality_band()
    }
}

// ---------------------------------------------------------------------------
// SensorAccuracyBand
// ---------------------------------------------------------------------------

/// Classification of sensor accuracy for reward calculation.
///
/// Thresholds: Excellent >= 0.90, Good >= 0.75, Marginal >= 0.50, Poor < 0.50.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SensorAccuracyBand {
    /// Excellent accuracy — full reward eligibility.
    Excellent,
    /// Good accuracy — standard reward eligibility.
    Good,
    /// Marginal accuracy — reduced rewards, flagged for calibration.
    Marginal,
    /// Poor accuracy — no rewards, may trigger slashing.
    Poor,
}

impl SensorAccuracyBand {
    /// Classify an accuracy score into a band.
    pub fn from_score(accuracy_score: f64) -> Self {
        if accuracy_score >= 0.90 {
            Self::Excellent
        } else if accuracy_score >= 0.75 {
            Self::Good
        } else if accuracy_score >= 0.50 {
            Self::Marginal
        } else {
            Self::Poor
        }
    }

    /// Convert to the corresponding [`DataQualityBand`].
    pub fn to_quality_band(self) -> DataQualityBand {
        match self {
            Self::Excellent => DataQualityBand::High,
            Self::Good => DataQualityBand::Borderline,
            Self::Marginal => DataQualityBand::WarningOnly,
            Self::Poor => DataQualityBand::Low,
        }
    }
}

impl core::fmt::Display for SensorAccuracyBand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Excellent => write!(f, "Excellent"),
            Self::Good => write!(f, "Good"),
            Self::Marginal => write!(f, "Marginal"),
            Self::Poor => write!(f, "Poor"),
        }
    }
}

// ---------------------------------------------------------------------------
// AccuracyWindow
// ---------------------------------------------------------------------------

/// Rolling accuracy window metadata for a sensor over a time period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccuracyWindow {
    /// Unix timestamp of the earliest pair in the window.
    pub window_start: u64,
    /// Unix timestamp of the latest pair in the window.
    pub window_end: u64,
    /// Computed accuracy metrics for this window.
    pub metrics: SensorAccuracyMetrics,
}

impl AccuracyWindow {
    /// Create a new accuracy window.
    pub fn new(window_start: u64, window_end: u64, metrics: SensorAccuracyMetrics) -> Self {
        Self {
            window_start,
            window_end,
            metrics,
        }
    }

    /// Validate window consistency (metrics valid).
    #[must_use = "validation errors must be handled"]
    pub fn validate(&self) -> Result<(), IoTAccuracyError> {
        self.metrics.validate()
    }

    /// Duration of the window in seconds.
    pub fn duration_secs(&self) -> u64 {
        self.window_end.saturating_sub(self.window_start)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    // -- SensorReferencePair -------------------------------------------------

    #[test]
    fn pair_error_positive() {
        let p = SensorReferencePair::new(25.0, 20.0);
        assert!(approx_eq(p.error(), 5.0));
    }

    #[test]
    fn pair_error_negative() {
        let p = SensorReferencePair::new(18.0, 20.0);
        assert!(approx_eq(p.error(), -2.0));
    }

    #[test]
    fn pair_abs_error() {
        let p = SensorReferencePair::new(18.0, 20.0);
        assert!(approx_eq(p.abs_error(), 2.0));
    }

    // -- SensorAccuracyMetrics::from_pairs -----------------------------------

    #[test]
    fn from_pairs_empty() {
        let result = SensorAccuracyMetrics::from_pairs(&[]);
        assert_eq!(result, Err(IoTAccuracyError::ZeroSampleCount));
    }

    #[test]
    fn from_pairs_perfect() {
        let pairs: Vec<SensorReferencePair> = (1..=5)
            .map(|i| SensorReferencePair::new(i as f64, i as f64))
            .collect();
        let m = SensorAccuracyMetrics::from_pairs(&pairs).unwrap();
        assert!(approx_eq(m.mae, 0.0));
        assert!(approx_eq(m.rmse, 0.0));
        assert!(approx_eq(m.bias, 0.0));
        assert!(approx_eq(m.correlation, 1.0));
        assert!(approx_eq(m.accuracy_score, 1.0));
        assert_eq!(m.sample_count, 5);
    }

    #[test]
    fn from_pairs_constant_offset() {
        // sensor = reference + 2, with varying reference values
        let pairs: Vec<SensorReferencePair> = vec![10.0, 20.0, 30.0, 40.0, 50.0]
            .into_iter()
            .map(|r| SensorReferencePair::new(r + 2.0, r))
            .collect();
        let m = SensorAccuracyMetrics::from_pairs(&pairs).unwrap();
        assert!(approx_eq(m.mae, 2.0));
        assert!(approx_eq(m.rmse, 2.0));
        assert!(approx_eq(m.bias, 2.0));
        assert!(approx_eq(m.correlation, 1.0));
        assert_eq!(m.sample_count, 5);
    }

    #[test]
    fn from_pairs_mixed() {
        let pairs = vec![
            SensorReferencePair::new(10.0, 12.0), // err = -2
            SensorReferencePair::new(22.0, 20.0), // err = +2
            SensorReferencePair::new(31.0, 30.0), // err = +1
        ];
        let m = SensorAccuracyMetrics::from_pairs(&pairs).unwrap();
        // MAE = (2 + 2 + 1) / 3 = 5/3
        assert!(approx_eq(m.mae, 5.0 / 3.0));
        // RMSE = sqrt((4 + 4 + 1) / 3) = sqrt(3)
        assert!(approx_eq(m.rmse, (3.0_f64).sqrt()));
        // bias = (-2 + 2 + 1) / 3 = 1/3
        assert!(approx_eq(m.bias, 1.0 / 3.0));
        assert_eq!(m.sample_count, 3);
    }

    #[test]
    fn from_pairs_single() {
        let pairs = vec![SensorReferencePair::new(15.0, 10.0)];
        let m = SensorAccuracyMetrics::from_pairs(&pairs).unwrap();
        // Single pair: zero variance → correlation = 0.0
        assert!(approx_eq(m.correlation, 0.0));
        assert!(approx_eq(m.mae, 5.0));
        assert!(approx_eq(m.rmse, 5.0));
        assert!(approx_eq(m.bias, 5.0));
        assert_eq!(m.sample_count, 1);
    }

    #[test]
    fn from_pairs_anticorrelated() {
        // As reference goes up, sensor goes down
        let pairs = vec![
            SensorReferencePair::new(30.0, 10.0),
            SensorReferencePair::new(20.0, 20.0),
            SensorReferencePair::new(10.0, 30.0),
        ];
        let m = SensorAccuracyMetrics::from_pairs(&pairs).unwrap();
        assert!(m.correlation < 0.0, "expected negative correlation");
        assert!(approx_eq(m.correlation, -1.0));
    }

    // -- SensorAccuracyMetrics::validate -------------------------------------

    #[test]
    fn validate_valid() {
        let m = SensorAccuracyMetrics {
            mae: 1.0,
            rmse: 1.5,
            bias: -0.3,
            correlation: 0.95,
            sample_count: 100,
            accuracy_score: 0.85,
        };
        assert!(m.validate().is_ok());
    }

    #[test]
    fn validate_negative_mae() {
        let m = SensorAccuracyMetrics {
            mae: -1.0,
            rmse: 1.0,
            bias: 0.0,
            correlation: 0.5,
            sample_count: 10,
            accuracy_score: 0.8,
        };
        assert_eq!(m.validate(), Err(IoTAccuracyError::InvalidMae));
    }

    #[test]
    fn validate_nan_rmse() {
        let m = SensorAccuracyMetrics {
            mae: 1.0,
            rmse: f64::NAN,
            bias: 0.0,
            correlation: 0.5,
            sample_count: 10,
            accuracy_score: 0.8,
        };
        assert_eq!(m.validate(), Err(IoTAccuracyError::InvalidRmse));
    }

    #[test]
    fn validate_correlation_out_of_range() {
        let m = SensorAccuracyMetrics {
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            correlation: 1.5,
            sample_count: 10,
            accuracy_score: 0.8,
        };
        assert_eq!(m.validate(), Err(IoTAccuracyError::InvalidCorrelation));
    }

    #[test]
    fn validate_accuracy_score_out_of_range() {
        let m = SensorAccuracyMetrics {
            mae: 1.0,
            rmse: 1.0,
            bias: 0.0,
            correlation: 0.5,
            sample_count: 10,
            accuracy_score: 1.1,
        };
        assert_eq!(m.validate(), Err(IoTAccuracyError::InvalidAccuracyScore));
    }

    // -- SensorAccuracyBand --------------------------------------------------

    #[test]
    fn band_classification() {
        assert_eq!(SensorAccuracyBand::from_score(0.95), SensorAccuracyBand::Excellent);
        assert_eq!(SensorAccuracyBand::from_score(0.90), SensorAccuracyBand::Excellent);
        assert_eq!(SensorAccuracyBand::from_score(0.80), SensorAccuracyBand::Good);
        assert_eq!(SensorAccuracyBand::from_score(0.75), SensorAccuracyBand::Good);
        assert_eq!(SensorAccuracyBand::from_score(0.60), SensorAccuracyBand::Marginal);
        assert_eq!(SensorAccuracyBand::from_score(0.50), SensorAccuracyBand::Marginal);
        assert_eq!(SensorAccuracyBand::from_score(0.30), SensorAccuracyBand::Poor);
    }

    #[test]
    fn quality_band_mapping() {
        assert_eq!(SensorAccuracyBand::Excellent.to_quality_band(), DataQualityBand::High);
        assert_eq!(SensorAccuracyBand::Good.to_quality_band(), DataQualityBand::Borderline);
        assert_eq!(SensorAccuracyBand::Marginal.to_quality_band(), DataQualityBand::WarningOnly);
        assert_eq!(SensorAccuracyBand::Poor.to_quality_band(), DataQualityBand::Low);
    }

    // -- Serde roundtrips ----------------------------------------------------

    #[test]
    fn serde_roundtrip_metrics() {
        let m = SensorAccuracyMetrics {
            mae: 1.5,
            rmse: 2.0,
            bias: -0.3,
            correlation: 0.92,
            sample_count: 50,
            accuracy_score: 0.87,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: SensorAccuracyMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn serde_roundtrip_pair() {
        let p = SensorReferencePair::new(25.3, 24.8);
        let json = serde_json::to_string(&p).unwrap();
        let back: SensorReferencePair = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_roundtrip_window() {
        let w = AccuracyWindow::new(
            1_700_000_000,
            1_700_003_600,
            SensorAccuracyMetrics {
                mae: 1.0,
                rmse: 1.2,
                bias: 0.1,
                correlation: 0.98,
                sample_count: 30,
                accuracy_score: 0.91,
            },
        );
        let json = serde_json::to_string(&w).unwrap();
        let back: AccuracyWindow = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }

    // -- AccuracyWindow ------------------------------------------------------

    #[test]
    fn accuracy_window_duration() {
        let w = AccuracyWindow::new(
            1000,
            2000,
            SensorAccuracyMetrics {
                mae: 0.0,
                rmse: 0.0,
                bias: 0.0,
                correlation: 0.0,
                sample_count: 1,
                accuracy_score: 1.0,
            },
        );
        assert_eq!(w.duration_secs(), 1000);
    }

    // -- Display impls -------------------------------------------------------

    #[test]
    fn error_display() {
        assert!(!IoTAccuracyError::InvalidMae.to_string().is_empty());
        assert!(!IoTAccuracyError::InvalidRmse.to_string().is_empty());
        assert!(!IoTAccuracyError::InvalidBias.to_string().is_empty());
        assert!(!IoTAccuracyError::InvalidCorrelation.to_string().is_empty());
        assert!(!IoTAccuracyError::InvalidAccuracyScore.to_string().is_empty());
        assert!(!IoTAccuracyError::ZeroSampleCount.to_string().is_empty());
    }

    #[test]
    fn sensor_accuracy_band_display() {
        assert_eq!(SensorAccuracyBand::Excellent.to_string(), "Excellent");
        assert_eq!(SensorAccuracyBand::Good.to_string(), "Good");
        assert_eq!(SensorAccuracyBand::Marginal.to_string(), "Marginal");
        assert_eq!(SensorAccuracyBand::Poor.to_string(), "Poor");
    }
}
