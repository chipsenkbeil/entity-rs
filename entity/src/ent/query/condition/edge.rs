use super::Condition;

/// Represents a condition on an ent's edge
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
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

    /// Shorthand to produce an edge condition that checks if
    /// any ent in an edge passes the condition
    #[inline]
    pub fn any<C: Into<Condition>>(c: C) -> Self {
        Self::Any(Box::from(c.into()))
    }

    /// Shorthand to produce an edge condition that checks if
    /// exactly N ents in an edge pass the condition
    #[inline]
    pub fn exactly<C: Into<Condition>>(c: C, n: usize) -> Self {
        Self::Exactly(Box::from(c.into()), n)
    }

    /// Shorthand to produce an edge condition that checks if
    /// all ents in an edge pass the condition
    #[inline]
    pub fn all<C: Into<Condition>>(c: C) -> Self {
        Self::All(Box::from(c.into()))
    }
}
