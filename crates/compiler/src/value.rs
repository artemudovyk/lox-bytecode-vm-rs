use std::{fmt::Display, ops::Mul};

#[derive(Debug, Clone, Default)]
pub struct Value(pub(crate) f64);

impl Value {
    pub fn new(val: f64) -> Self {
        Self(val)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Mul<i32> for Value {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}
