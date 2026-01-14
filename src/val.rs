use core::fmt;
use ordered_float::{self, OrderedFloat};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Val {
    Int(i32),
    Float(OrderedFloat<f32>),
}

impl Neg for Val {
    type Output = Val;

    fn neg(self) -> Self {
        match self {
            Self::Float(f) => Val::Float(-f),
            Self::Int(n) => Val::Int(-n),
        }
    }
}

impl Add for Val {
    type Output = Val;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => Val::Int(a + b),
            (Val::Float(a), Val::Float(b)) => Val::Float(a + b),
            (Val::Float(a), Val::Int(b)) => Val::Float(a + (b as f32)),
            (Val::Int(a), Val::Float(b)) => Val::Float(OrderedFloat(a as f32) + b),
        }
    }
}

impl Sub for Val {
    type Output = Val;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => Val::Int(a - b),
            (Val::Float(a), Val::Float(b)) => Val::Float(a - b),
            (Val::Float(a), Val::Int(b)) => Val::Float(a - (b as f32)),
            (Val::Int(a), Val::Float(b)) => Val::Float(OrderedFloat(a as f32) - b),
        }
    }
}

impl Mul for Val {
    type Output = Val;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => Val::Int(a * b),
            (Val::Float(a), Val::Float(b)) => Val::Float(a * b),
            (Val::Float(a), Val::Int(b)) => Val::Float(a * (b as f32)),
            (Val::Int(a), Val::Float(b)) => Val::Float(OrderedFloat(a as f32) * b),
        }
    }
}

impl Div for Val {
    type Output = Val;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => Val::Int(a / b),
            (Val::Float(a), Val::Float(b)) => Val::Float(a / b),
            (Val::Float(a), Val::Int(b)) => Val::Float(a / (b as f32)),
            (Val::Int(a), Val::Float(b)) => Val::Float(OrderedFloat(a as f32) / b),
        }
    }
}

impl AddAssign for Val {
    fn add_assign(&mut self, rhs: Val) {
        *self = *self + rhs;
    }
}

impl SubAssign for Val {
    fn sub_assign(&mut self, rhs: Val) {
        *self = self.clone() - rhs;
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
        }
    }
}
