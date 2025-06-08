use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    /// Creates a new vector with the given x and y coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Vector { x, y }
    }

    /// Returns the string representation of the vector.
    pub fn to_string(&self) -> String {
        format!("({:.4}, {:.4})", self.x, self.y)
    }

    /// Returns the length of the vector.
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector({}, {})", self.x, self.y)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, other: f64) -> Vector {
        Vector::new(self.x * other, self.y * other)
    }
}

impl Mul<i8> for Vector {
    type Output = Vector;

    fn mul(self, other: i8) -> Vector {
        Vector::new(self.x * (other as f64), self.y * (other as f64))
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, other: f64) -> Vector {
        if other == 0.0 {
            panic!("Division by zero is not allowed");
        }
        Vector::new(self.x / other, self.y / other)
    }
}
