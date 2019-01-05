use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign, Neg};
use std::fmt::{Debug, Display};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Precision {
    Single,
    Double,
}

pub trait Scalar: Debug + Display + Copy + Add + Sub + Mul + Div + AddAssign + SubAssign + MulAssign + DivAssign + Neg<Output=Self> + PartialEq + PartialOrd {
    fn precision() -> Precision;
    fn zero() -> Self;
    fn one() -> Self;
    fn recip(self) -> Self;
    fn from_f32(v: f32) -> Self;
    fn from_f64(v: f64) -> Self;
    fn as_f32(self) -> f32;
    fn as_f64(self) -> f64;
}

impl Scalar for f32 {
    fn precision() -> Precision { Precision::Single }
    fn zero() -> f32 { 0.0f32 }
    fn one() -> f32 { 1.0f32 }
    fn recip(self) -> f32 { self.recip() }
    fn from_f32(v: f32) -> f32 { v }
    fn from_f64(v: f64) -> f32 { v as f32 }
    fn as_f32(self) -> f32 { self }
    fn as_f64(self) -> f64 { self as f64 }
}

impl Scalar for f64 {
    fn precision() -> Precision { Precision::Double }
    fn zero() -> f64 { 0.0f64 }
    fn one() -> f64 { 1.0f64 }
    fn recip(self) -> f64 { self.recip() }
    fn from_f32(v: f32) -> f64 { v as f64 }
    fn from_f64(v: f64) -> f64 { v }
    fn as_f32(self) -> f32 { self as f32 }
    fn as_f64(self) -> f64 { self }
}
