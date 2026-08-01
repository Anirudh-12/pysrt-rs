use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use crate::error::{Result, SrtError};

pub const SECONDS_RATIO: i64 = 1000;
pub const MINUTES_RATIO: i64 = SECONDS_RATIO * 60;
pub const HOURS_RATIO: i64 = MINUTES_RATIO * 60;

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubRipTime {
    pub ordinal: i64,
}

impl SubRipTime {
    pub fn new(hours: i64, minutes: i64, seconds: i64, milliseconds: i64) -> Self {
        let ordinal = hours * HOURS_RATIO
            + minutes * MINUTES_RATIO
            + seconds * SECONDS_RATIO
            + milliseconds;
        Self { ordinal }
    }

    pub fn from_ordinal(ordinal: i64) -> Self {
        Self { ordinal }
    }

    pub fn from_string(source: &str) -> Result<Self> {
        // Split by ':', '.', ','
        let parts: Vec<&str> = source
            .split(|c| c == ':' || c == '.' || c == ',')
            .collect();
        if parts.len() != 4 {
            return Err(SrtError::InvalidTimeString(source.to_string()));
        }
        let h = Self::parse_int(parts[0]);
        let m = Self::parse_int(parts[1]);
        let s = Self::parse_int(parts[2]);
        let ms = Self::parse_int(parts[3]);
        Ok(Self::new(h, m, s, ms))
    }

    pub fn parse_int(digits: &str) -> i64 {
        let digits = digits.trim();
        if let Ok(val) = digits.parse::<i64>() {
            return val;
        }
        // Match leading ASCII digits
        let leading: String = digits.chars().take_while(|c| c.is_ascii_digit()).collect();
        leading.parse::<i64>().unwrap_or(0)
    }

    pub fn hours(&self) -> i64 {
        self.ordinal / HOURS_RATIO
    }

    pub fn set_hours(&mut self, val: i64) {
        let current_h = self.hours();
        self.ordinal += (val - current_h) * HOURS_RATIO;
    }

    pub fn minutes(&self) -> i64 {
        (self.ordinal % HOURS_RATIO) / MINUTES_RATIO
    }

    pub fn set_minutes(&mut self, val: i64) {
        let current_m = self.minutes();
        self.ordinal += (val - current_m) * MINUTES_RATIO;
    }

    pub fn seconds(&self) -> i64 {
        (self.ordinal % MINUTES_RATIO) / SECONDS_RATIO
    }

    pub fn set_seconds(&mut self, val: i64) {
        let current_s = self.seconds();
        self.ordinal += (val - current_s) * SECONDS_RATIO;
    }

    pub fn milliseconds(&self) -> i64 {
        self.ordinal % SECONDS_RATIO
    }

    pub fn set_milliseconds(&mut self, val: i64) {
        let current_ms = self.milliseconds();
        self.ordinal += val - current_ms;
    }

    pub fn shift(
        &mut self,
        hours: i64,
        minutes: i64,
        seconds: i64,
        milliseconds: i64,
        ratio: Option<f64>,
    ) {
        if let Some(r) = ratio {
            *self *= r;
        }
        *self += Self::new(hours, minutes, seconds, milliseconds);
    }
}

impl fmt::Display for SubRipTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ordinal < 0 {
            return write!(f, "00:00:00,000");
        }
        write!(
            f,
            "{:02}:{:02}:{:02},{:03}",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds()
        )
    }
}

impl fmt::Debug for SubRipTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubRipTime({}, {}, {}, {})",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds()
        )
    }
}

impl Add for SubRipTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_ordinal(self.ordinal + rhs.ordinal)
    }
}

impl AddAssign for SubRipTime {
    fn add_assign(&mut self, rhs: Self) {
        self.ordinal += rhs.ordinal;
    }
}

impl Sub for SubRipTime {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_ordinal(self.ordinal - rhs.ordinal)
    }
}

impl SubAssign for SubRipTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.ordinal -= rhs.ordinal;
    }
}

impl Mul<f64> for SubRipTime {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let ord = (self.ordinal as f64 * rhs).round() as i64;
        Self::from_ordinal(ord)
    }
}

impl MulAssign<f64> for SubRipTime {
    fn mul_assign(&mut self, rhs: f64) {
        self.ordinal = (self.ordinal as f64 * rhs).round() as i64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_value() {
        let t = SubRipTime::default();
        assert_eq!(t.ordinal, 0);
    }

    #[test]
    fn test_milliseconds() {
        let mut t = SubRipTime::default();
        t.set_milliseconds(1);
        assert_eq!(t.milliseconds(), 1);
        t.set_hours(t.hours() + 42);
        assert_eq!(t.milliseconds(), 1);
        t.set_milliseconds(t.milliseconds() + 1000);
        assert_eq!(t.seconds(), 1);
    }

    #[test]
    fn test_parse_string() {
        let t = SubRipTime::from_string("01:02:03,456").unwrap();
        assert_eq!(t.hours(), 1);
        assert_eq!(t.minutes(), 2);
        assert_eq!(t.seconds(), 3);
        assert_eq!(t.milliseconds(), 456);
        assert_eq!(t.to_string(), "01:02:03,456");
    }

    #[test]
    fn test_parse_int_recovery() {
        assert_eq!(SubRipTime::parse_int("45foo"), 45);
        assert_eq!(SubRipTime::parse_int("foo"), 0);
    }
}
