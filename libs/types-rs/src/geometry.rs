use std::ops::{Add, AddAssign, Sub};

use serde::Serialize;

/// A unit of length in timing mark grid, i.e. 1 `GridUnit` is the logical
/// distance from one timing mark to the next. This does not map directly to
/// pixels.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type GridUnit = u32;

/// An x or y coordinate in pixels.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type PixelPosition = i32;

/// A width or height in pixels.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type PixelUnit = u32;

/// A sub-pixel coordinate or distance of pixels.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type SubPixelUnit = f32;

/// Angle in radians.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type Radians = f32;

/// Fractional number of inches.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type Inch = f32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Point<T: Sub<Output = T>> {
    pub x: T,
    pub y: T,
}

impl<T: Sub<Output = T>> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Sub<Output = T> + Add<Output = T>> Add for Point<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Sub<Output = T> + AddAssign + Copy> AddAssign for Point<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Point<SubPixelUnit> {
    pub fn round(self) -> Point<PixelPosition> {
        Point::new(
            self.x.round() as PixelPosition,
            self.y.round() as PixelPosition,
        )
    }
}

/// A rectangle area of pixels within an image.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Rect {
    left: PixelPosition,
    top: PixelPosition,
    width: PixelUnit,
    height: PixelUnit,
}

impl Rect {
    pub const fn new(
        left: PixelPosition,
        top: PixelPosition,
        width: PixelUnit,
        height: PixelUnit,
    ) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    pub const fn from_points(
        top_left: Point<PixelPosition>,
        bottom_right: Point<PixelPosition>,
    ) -> Self {
        Self::new(
            top_left.x,
            top_left.y,
            (bottom_right.x - top_left.x + 1) as PixelUnit,
            (bottom_right.y - top_left.y + 1) as PixelUnit,
        )
    }

    pub const fn left(&self) -> PixelPosition {
        self.left
    }

    pub const fn top(&self) -> PixelPosition {
        self.top
    }

    pub const fn width(&self) -> PixelUnit {
        self.width
    }

    pub const fn height(&self) -> PixelUnit {
        self.height
    }

    pub const fn right(&self) -> PixelPosition {
        self.left + self.width as PixelPosition - 1
    }

    pub const fn bottom(&self) -> PixelPosition {
        self.top + self.height as PixelPosition - 1
    }

    pub const fn offset(&self, dx: PixelPosition, dy: PixelPosition) -> Self {
        Self::new(self.left + dx, self.top + dy, self.width, self.height)
    }

    pub const fn top_left(&self) -> Point<PixelPosition> {
        Point::new(self.left, self.top)
    }

    pub const fn bottom_right(&self) -> Point<PixelPosition> {
        Point::new(self.right(), self.bottom())
    }

    pub fn center(&self) -> Point<SubPixelUnit> {
        Point::new(
            self.left() as SubPixelUnit
                + (self.right() as SubPixelUnit - self.left() as SubPixelUnit) / 2.0,
            self.top() as SubPixelUnit
                + (self.bottom() as SubPixelUnit - self.top() as SubPixelUnit) / 2.0,
        )
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if left <= right && top <= bottom {
            Some(Self::new(
                left,
                top,
                (right - left + 1) as PixelUnit,
                (bottom - top + 1) as PixelUnit,
            ))
        } else {
            None
        }
    }

    // Returns the smallest rectangle that contains both `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        let left = self.left.min(other.left);
        let top = self.top.min(other.top);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(
            left,
            top,
            (right - left + 1) as PixelUnit,
            (bottom - top + 1) as PixelUnit,
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[cfg(test)]

mod normalize_center_of_rect {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_center_of_rect() {
        let rect = super::Rect::new(0, 0, 10, 10);
        let center = rect.center();
        assert_eq!(center.x, 4.5);
        assert_eq!(center.y, 4.5);
    }

    #[test]
    fn test_center_of_rect_with_odd_dimensions() {
        let rect = super::Rect::new(0, 0, 11, 11);
        let center = rect.center();
        assert_eq!(center.x, 5.0);
        assert_eq!(center.y, 5.0);
    }

    proptest! {
        #[test]
        fn prop_center_of_rect_is_in_rect(x in 0i32..100i32, y in 0i32..100i32, width in 1u32..100u32, height in 1u32..100u32) {
            let rect = super::Rect::new(x, y, width, height);
            let center = rect.center();
            prop_assert!((rect.left() as SubPixelUnit) <= center.x);
            prop_assert!(center.x <= (rect.right() as SubPixelUnit));
            prop_assert!((rect.top() as SubPixelUnit) <= center.y);
            prop_assert!(center.y <= (rect.bottom() as SubPixelUnit));
        }
    }
}
