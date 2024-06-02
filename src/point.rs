use std::ops::{Add, Sub};

use serde::Deserialize;

#[derive(Default, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn dist_sq(&self, other: &Point) -> usize {
        let dx = self.x as i64 - other.x as i64;
        let dy = self.y as i64 - other.y as i64;
        (dx * dx + dy * dy) as usize
    }

    pub fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }

    pub fn with_resolution(&self, resolution: usize) -> Self {
        Point {
            x: self.x / resolution * resolution,
            y: self.y / resolution * resolution,
        }
    }
}

impl From<&Point> for [usize; 2] {
    fn from(p: &Point) -> [usize; 2] {
        [p.x, p.y]
    }
}

impl From<&[usize; 2]> for Point {
    fn from(p: &[usize; 2]) -> Self {
        Point { x: p[0], y: p[1] }
    }
}

impl Add<usize> for &Point {
    type Output = Point;

    fn add(self, rhs: usize) -> Self::Output {
        Point {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Sub<usize> for &Point {
    type Output = Point;

    fn sub(self, rhs: usize) -> Self::Output {
        Point {
            x: self.x.saturating_sub(rhs),
            y: self.y.saturating_sub(rhs),
        }
    }
}
