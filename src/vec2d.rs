use std::ops::{Add, Sub};

#[derive(Clone, PartialEq, Eq, Debug, Copy, Default, Hash)]
pub struct Vec2D<T> {
    pub x: T,
    pub y: T,
}

impl Sub for Vec2D<i32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add for Vec2D<i32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Vec2D<i32> {
    pub fn distance_manhatten(&self, b: &Vec2D<i32>) -> i32 {
        (self.x - b.x).abs() + (self.y - b.y).abs()
    }

    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}
