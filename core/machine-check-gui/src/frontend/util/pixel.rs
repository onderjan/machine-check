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

impl PixelPoint {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn to_tuple(self) -> (i64, i64) {
        (self.x, self.y)
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelRect {
    top_left: PixelPoint,
    bottom_right: PixelPoint,
}

impl PixelRect {
    pub fn new(top_left: PixelPoint, bottom_right: PixelPoint) -> Self {
        assert!(top_left.x <= bottom_right.x);
        assert!(top_left.y <= bottom_right.y);
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn top_left(self) -> PixelPoint {
        self.top_left
    }

    pub fn bottom_right(self) -> PixelPoint {
        self.bottom_right
    }

    pub fn left_x(self) -> i64 {
        self.top_left.x
    }

    pub fn right_x(self) -> i64 {
        self.bottom_right.x
    }

    pub fn middle_x(self) -> i64 {
        // TODO: use midpoint once it is stabilised
        self.left_x() + self.width() as i64 / 2
    }

    pub fn top_y(self) -> i64 {
        self.top_left.y
    }

    #[allow(dead_code)]
    pub fn bottom_y(self) -> i64 {
        self.bottom_right.y
    }

    pub fn middle_y(self) -> i64 {
        self.top_y() + self.height() as i64 / 2
    }

    pub fn width(self) -> u64 {
        (self.bottom_right.x - self.top_left.x) as u64
    }

    pub fn height(self) -> u64 {
        (self.bottom_right.y - self.top_left.y) as u64
    }

    pub fn without_half_margin(self, half_margin_x: u64, half_margin_y: u64) -> Self {
        let half_margin_point = PixelPoint {
            x: half_margin_x as i64,
            y: half_margin_y as i64,
        };

        let top_left = self.top_left + half_margin_point;
        let mut bottom_right = self.bottom_right - half_margin_point;
        if bottom_right.x < top_left.x {
            bottom_right.x = top_left.x;
        }
        if bottom_right.y < top_left.y {
            bottom_right.y = top_left.y;
        }

        Self::new(top_left, bottom_right)
    }
}
