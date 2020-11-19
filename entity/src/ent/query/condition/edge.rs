use super::Condition;

/// Represents a condition on an ent's edge
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeCondition {
    /// For all ents on a connected edge, check if at least one passes the condition
    Any(Box<Condition>),

    /// For all ents on a connected edge, check if exactly N passes the condition
    Exactly(Box<Condition>, usize),

    /// For all ents on a connected edge, check ll pass the condition
    All(Box<Condition>),
}

impl EdgeCondition {
    /// Returns the underlying condition
    pub fn condition(&self) -> &Condition {
        match self {
            Self::Any(cond) => cond.as_ref(),
            Self::Exactly(cond, _) => cond.as_ref(),
            Self::All(cond) => cond.as_ref(),
        }
    }
}