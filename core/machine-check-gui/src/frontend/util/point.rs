use std::ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign};

/// A type for 2D points on the canvas represented in whole pixels.
///
/// Whole pixels are intentionally used so that there are no problems
/// with e.g. subpixel translation which would make the canvas look ugly.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelPoint {
    pub x: i64,
    pub y: i64,
}

impl Add for PixelPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        PixelPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for PixelPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for PixelPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        PixelPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for PixelPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for PixelPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        PixelPoint {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<i64> for PixelPoint {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        PixelPoint {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<i64> for PixelPoint {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        PixelPoint {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Rem<i64> for PixelPoint {
    type Output = Self;

    fn rem(self, rhs: i64) -> Self::Output {
        PixelPoint {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}
