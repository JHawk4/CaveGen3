//! A small 3D point type with the game's distance function.

use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Sub, SubAssign};

use crate::math;

/// A point or vector in game coordinates (x, y, z).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point(pub [f32; 3]);

impl Point {
    pub const ZERO: Point = Point([0.0; 3]);

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point([x, y, z])
    }

    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn z(&self) -> f32 {
        self.0[2]
    }

    /// Euclidean distance with the standard square root.
    pub fn dist(&self, other: &Point) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..3 {
            let d = self.0[i] - other.0[i];
            sum += d * d;
        }
        sum.sqrt()
    }

    /// Distance from the origin, with the standard square root.
    pub fn length(&self) -> f32 {
        self.dist(&Point::ZERO)
    }

    /// This vector scaled to length 1, or unchanged if it's zero.
    pub fn normalized(self) -> Point {
        let factor = Point::ZERO.dist(&self);
        if factor == 0.0 {
            return self;
        }
        Point([self.0[0] / factor, self.0[1] / factor, self.0[2] / factor])
    }

    pub fn dot(self, other: Point) -> f32 {
        self.0.into_iter().zip(other.0).map(|(s, o)| s * o).sum()
    }

    /// Distance in the XZ plane only, with the standard square root.
    pub fn dist_xz(&self, other: &Point) -> f32 {
        let (dx, dz) = (self.0[0] - other.0[0], self.0[2] - other.0[2]);
        let mut sum = 0.0f32;
        sum += dx * dx;
        sum += dz * dz;
        sum.sqrt()
    }

    /// Distance using the game's approximate square root, summing the axes in order the
    /// way the game does. Generation depends on the exact result.
    #[inline]
    pub fn p2_dist(&self, other: &Point) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..3 {
            let d = self.0[i] - other.0[i];
            sum += d * d;
        }
        math::sqrt(sum)
    }
}

impl Index<usize> for Point {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        &self.0[i]
    }
}

impl IndexMut<usize> for Point {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.0[i]
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, o: Point) -> Point {
        Point([self.0[0] + o.0[0], self.0[1] + o.0[1], self.0[2] + o.0[2]])
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, o: Point) -> Point {
        Point([self.0[0] - o.0[0], self.0[1] - o.0[1], self.0[2] - o.0[2]])
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, k: f32) -> Point {
        Point([self.0[0] * k, self.0[1] * k, self.0[2] * k])
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, o: Point) {
        *self = *self + o;
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, o: Point) {
        *self = *self - o;
    }
}
