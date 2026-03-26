//! Shared timestamp parsing utilities.
//!
//! All services should use these functions instead of defining their own
//! `parse_timestamp` copies.

/// Parse a DB timestamp string (RFC 3339 or `YYYY-MM-DD HH:MM:SS.f`) into a
/// prost `Timestamp`.  Returns `None` for unparseable input.
pub fn parse_timestamp(s: &str) -> Option<prost_types::Timestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                .map(|ndt| ndt.and_utc().fixed_offset())
        })
        .ok()
        .map(|dt| prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}
