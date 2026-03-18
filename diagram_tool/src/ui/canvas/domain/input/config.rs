use crate::ui::canvas::domain::input::types::Error;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NonNegativeF64(f64);

impl NonNegativeF64 {
    pub fn new(val: f64) -> Result<Self, Error> {
        if val < 0.0 || val.is_nan() {
            Err(Error::NegativeHitPadding)
        } else {
            Ok(Self(val))
        }
    }
    #[must_use]
    pub const fn get(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputConfig {
    pub double_tap_timeout_ms: NonZeroU64,
    pub touch_padding: NonNegativeF64,
    pub base_radius: f64,
}

impl InputConfig {
    pub fn new(
        double_tap_timeout_ms: u64,
        touch_padding: f64,
        base_radius: f64,
    ) -> Result<Self, Error> {
        let timeout =
            NonZeroU64::new(double_tap_timeout_ms).ok_or(Error::InvalidTimingThreshold)?;
        let padding = NonNegativeF64::new(touch_padding)?;
        Ok(Self {
            double_tap_timeout_ms: timeout,
            touch_padding: padding,
            base_radius,
        })
    }
}
