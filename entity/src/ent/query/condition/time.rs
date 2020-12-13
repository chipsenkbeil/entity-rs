/// Represents a condition on an ent's created or last updated property
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeCondition {
    /// The time property has a value that comes before the provided value
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The value is exclusive, meaning that if the time value is exactly
    /// the provided value, the condition should NOT pass
    Before(u64),

    /// The time property has a value that comes before the provided value
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The value is inclusive, meaning that if the time value is exactly
    /// the provided value, the condition should still pass
    OnOrBefore(u64),

    /// The time property has a value that comes after the provided value
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The value is exclusive, meaning that if the time value is exactly
    /// the provided value, the condition should NOT pass
    After(u64),

    /// The time property has a value that comes after the provided value
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The value is inclusive, meaning that if the time value is exactly
    /// the provided value, the condition should still pass
    OnOrAfter(u64),

    /// The time property has a value that comes between the provided values
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The values are exclusive, meaning that if the time value is exactly
    /// one of the two provided values, this condition should NOT pass
    Between(u64, u64),

    /// The time property has a value that comes between the provided values
    /// as the milliseconds since epoch (1970-01-01 00:00:00 UTC)
    ///
    /// The values are inclusive, meaning that if the time value is exactly
    /// one of the two provided values, this condition should still pass
    OnOrBetween(u64, u64),
}

impl TimeCondition {
    /// Checks if the given time since epoch passes the condition.
    pub fn check(&self, value: u64) -> bool {
        match *self {
            Self::Before(x) => value < x,
            Self::OnOrBefore(x) => value <= x,
            Self::After(x) => value > x,
            Self::OnOrAfter(x) => value >= x,
            Self::Between(x, y) => value > x && value < y,
            Self::OnOrBetween(x, y) => value >= x && value <= y,
        }
    }
}
