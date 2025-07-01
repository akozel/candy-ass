use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, EnumString, AsRefStr, EnumIter)]
pub enum Timeframe {
    #[serde(rename = "1m")]
    #[strum(serialize = "1m")]
    OneMinute,
    #[serde(rename = "3m")]
    #[strum(serialize = "3m")]
    ThreeMinutes,
    #[serde(rename = "5m")]
    #[strum(serialize = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    #[strum(serialize = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    #[strum(serialize = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    #[strum(serialize = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    #[strum(serialize = "2h")]
    TwoHours,
    #[serde(rename = "3h")]
    #[strum(serialize = "3h")]
    ThreeHours,
    #[serde(rename = "4h")]
    #[strum(serialize = "4h")]
    FourHours,
    #[serde(rename = "6h")]
    #[strum(serialize = "6h")]
    SixHours,
    #[serde(rename = "8h")]
    #[strum(serialize = "8h")]
    EightHours,
    #[serde(rename = "12h")]
    #[strum(serialize = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    #[strum(serialize = "1d")]
    OneDay,
}

impl Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::timeframe::Timeframe;
    use crate::domain::timeframe::Timeframe::{OneMinute, ThreeMinutes};
    use std::str::FromStr;

    #[test]
    fn to_str() {
        // Given
        let min_1 = OneMinute;
        let min_3 = ThreeMinutes;

        // When
        let min_1_str = min_1.as_ref();
        let min_3_str = min_3.as_ref();

        // Then
        assert_eq!("1m", min_1_str);
        assert_eq!("3m", min_3_str);
    }

    #[test]
    fn from_str() {
        // Given
        let min_1 = OneMinute;

        // When
        let min_1_str = Timeframe::from_str("1m").unwrap();

        // Then
        assert_eq!(min_1_str, min_1);
    }
}
