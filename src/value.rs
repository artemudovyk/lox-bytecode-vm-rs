use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::vm::RuntimeError;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => {
                if n.trunc() != *n {
                    write!(f, "{n}")
                } else {
                    write!(f, "{n}.0")
                }
            }
            Self::Bool(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Neg for Value {
    type Output = Result<Self, RuntimeError>;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Ok(Self::Number(-n)),
            _ => Err(RuntimeError::OperandNotANumber),
        }
    }
}

impl Add for Value {
    type Output = Result<Self, RuntimeError>;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Ok(Self::Number(l + r)),
            _ => Err(RuntimeError::OperandNotANumber),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, RuntimeError>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Ok(Self::Number(l - r)),
            _ => Err(RuntimeError::OperandNotANumber),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, RuntimeError>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Ok(Self::Number(l * r)),
            _ => Err(RuntimeError::OperandNotANumber),
        }
    }
}

impl Div for Value {
    type Output = Result<Self, RuntimeError>;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Ok(Self::Number(l / r)),
            _ => Err(RuntimeError::OperandNotANumber),
        }
    }
}
