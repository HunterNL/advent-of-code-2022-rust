use std::{
    fmt::Debug,
    ops::{Add, Sub},
    str::FromStr,
};

#[derive(Clone, PartialEq, Eq, Debug, Copy, Default, Hash)]
pub struct Vec2D<T> {
    pub x: T,
    pub y: T,
}

pub const UP: Vec2D<i32> = Vec2D { x: 0, y: -1 };
pub const DOWN: Vec2D<i32> = Vec2D { x: 0, y: 1 };
pub const LEFT: Vec2D<i32> = Vec2D { x: -1, y: 0 };
pub const RIGHT: Vec2D<i32> = Vec2D { x: 1, y: 0 };

impl<T> FromStr for Vec2D<T>
where
    T: FromStr,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.split_once(',').ok_or("Could not split string")?;
        let a: Self = Self {
            x: left.parse().map_err(|_| "Could not parse left")?,
            y: right.parse().map_err(|_| "Could not parse right")?,
        };
        Ok(a)
    }
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

fn sign(x: i32) -> i32 {
    match x.cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl Vec2D<i32> {
    pub fn distance_manhatten(&self, b: &Self) -> i32 {
        (self.x - b.x).abs() + (self.y - b.y).abs()
    }

    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    /// Returns the normalized version of the vector. With i32s this only takes the sign of each component
    pub fn normalized(&self) -> Self {
        Self {
            x: sign(self.x),
            y: sign(self.y),
        }
    }

    /// Normalizes the vector. With i32s this only takes the sign of each component
    pub fn normalize(&mut self) {
        self.x = sign(self.x);
        self.y = sign(self.y);
    }

    pub fn scale(&self, factor: i32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

trait Bounds {
    fn bounds(&self) -> (Vec2D<i32>, Vec2D<i32>);
}

impl Bounds for [Vec2D<i32>] {
    fn bounds(&self) -> (Vec2D<i32>, Vec2D<i32>) {
        let mut min: Vec2D<i32> = *self.get(0).expect("At least one element");
        let mut max: Vec2D<i32> = min;

        for vec in self {
            if vec.x < min.x {
                min.x = vec.x;
            }
            if vec.y < min.y {
                min.y = vec.y;
            }

            if vec.x > max.x {
                max.x = vec.x;
            }
            if vec.y > max.y {
                max.y = vec.y;
            }
        }

        (max, min)
    }
}

// impl Bounds for dyn Iterator<Item = Vec2D<i32>> {
//     fn bounds(&self) -> (Vec2D<i32>, Vec2D<i32>) {
//         todo!()
//     }
// }

// impl Display for [Vec2D<i32>] {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// fn tryathing() {
//     let v: Vec<MyType> = vec![MyType { a: 1 }, MyType { a: 2 }, MyType { a: 3 }];
//     v.iter().default_implementation();
// }

pub trait Vec2DBounds {
    fn bounds_iter<T>(mut self) -> (Vec2D<T>, Vec2D<T>)
    where
        Self: Iterator<Item = Vec2D<T>> + Sized,
        T: PartialOrd + Clone + Ord + Sized,
    {
        let first = self
            .next()
            .expect("Iterator should contain at least one item");
        let mut min = first.clone();
        let mut max = first;

        for vec in self {
            min.x = min.x.min(vec.x.clone());
            min.y = min.y.min(vec.y.clone());

            max.x = max.x.max(vec.x);
            max.y = max.y.max(vec.y);
        }
        (min, max)
    }
}

impl<I> Vec2DBounds for I where I: Iterator<Item = Vec2D<i32>> {}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::Vec2D;
    use super::Vec2DBounds;

    #[test]
    fn bounds() {
        let vectors: Vec<Vec2D<i32>> = vec![
            Vec2D { x: -5, y: 22 },
            Vec2D { x: -17, y: 55 },
            Vec2D { x: 62, y: -42 },
            Vec2D { x: 30, y: 0 },
        ];

        let (min, max) = vectors.iter().copied().inspect(|_| {}).bounds_iter();

        assert_eq!(min.x, -17);
        assert_eq!(min.y, -42);
        assert_eq!(max.x, 62);
        assert_eq!(max.y, 55);
    }
}
