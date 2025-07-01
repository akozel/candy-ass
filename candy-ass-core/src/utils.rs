use time::OffsetDateTime;
use time::error::ComponentRange;

pub trait OffsetDateTimeExt {
    fn from_unix_timestamp_millis(millis: i64) -> Result<OffsetDateTime, ComponentRange>;
    fn unix_timestamp_millis(self) -> i64;
}

impl OffsetDateTimeExt for OffsetDateTime {
    fn from_unix_timestamp_millis(millis: i64) -> Result<Self, ComponentRange> {
        OffsetDateTime::from_unix_timestamp_nanos(millis as i128 * 1_000_000)
    }

    fn unix_timestamp_millis(self) -> i64 {
        self.unix_timestamp() * 1000 + self.millisecond() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn test_from_unix_timestamp_millis_and_back() {
        let millis = 1_682_544_000_000_i64; // 2023-05-01T00:00:00.000Z
        let dt = OffsetDateTime::from_unix_timestamp_millis(millis).unwrap();
        let roundtrip = dt.unix_timestamp_millis();
        assert_eq!(millis, roundtrip);

        let millis = 1_682_544_000_123_i64; // 2023-05-01T00:00:00.123Z
        let dt = OffsetDateTime::from_unix_timestamp_millis(millis).unwrap();
        let roundtrip = dt.unix_timestamp_millis();
        assert_eq!(millis, roundtrip);
    }

    #[test]
    fn test_from_unix_timestamp_millis_epoch() {
        let millis = 0_i64;
        let dt = OffsetDateTime::from_unix_timestamp_millis(millis).unwrap();
        assert_eq!(dt.unix_timestamp(), 0);
        assert_eq!(dt.millisecond(), 0);
        assert_eq!(dt.unix_timestamp_millis(), 0);
    }

    #[test]
    fn test_invalid_timestamp() {
        let millis = i64::MAX;
        let result = OffsetDateTime::from_unix_timestamp_millis(millis);
        assert!(result.is_err());
    }
}
