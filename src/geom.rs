use core::fmt::{Display, Formatter};
use core::ops::{Add, Sub};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

impl Size {
    fn empty() -> Size {
        Size { w: -99, h: -99 }
    }
}

impl Size {
    pub fn new(w: i32, h: i32) -> Size {
        Size { w, h }
    }
    pub fn sub(&self, b: &Insets) -> Size {
        Size {
            w: self.w - b.left - b.right,
            h: self.h - b.top - b.bottom,
        }
    }
    pub fn add(&self, b: &Insets) -> Size {
        Size {
            w: self.w + b.left + b.right,
            h: self.h + b.top + b.bottom,
        }
    }
}
impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}x{}", self.w, self.h)
    }
}
impl Add<Size> for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            w: self.w + rhs.w,
            h: self.h + rhs.h,
        }
    }
}
impl Add<Insets> for Size {
    type Output = Size;

    fn add(self, rhs: Insets) -> Self::Output {
        Self {
            w: self.w + rhs.left + rhs.right,
            h: self.h + rhs.top + rhs.bottom,
        }
    }
}
impl Add<&Insets> for Size {
    type Output = Size;

    fn add(self, rhs: &Insets) -> Self::Output {
        Self {
            w: self.w + rhs.left + rhs.right,
            h: self.h + rhs.top + rhs.bottom,
        }
    }
}
impl Sub<Insets> for Size {
    type Output = Self;

    fn sub(self, rhs: Insets) -> Self::Output {
        Size {
            w: self.w - rhs.left - rhs.right,
            h: self.h - rhs.top - rhs.bottom,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Insets {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl Insets {
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Insets {
        Insets {
            top,
            right,
            bottom,
            left,
        }
    }
    pub fn new_same(size: i32) -> Insets {
        Insets {
            top: size,
            bottom: size,
            left: size,
            right: size,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub(crate) fn negate(&self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Point {
    pub(crate) fn subtract(&self, p0: &Point) -> Point {
        Point {
            x: self.x - p0.x,
            y: self.y - p0.y,
        }
    }
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    pub fn zero() -> Point {
        Point::new(0, 0)
    }
}
impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    pub position: Point,
    pub size: Size,
}

impl Bounds {
    pub(crate) fn intersects(&self, b: Bounds) -> bool {
        if self.x2() < b.x() {
            return false;
        }
        if self.y2() < b.y() {
            return false;
        }
        if self.x() > b.x2() {
            return false;
        }
        if self.y() > b.y2() {
            return false;
        }

        true
    }
}

impl Bounds {
    pub(crate) fn x(&self) -> i32 {
        self.position.x
    }
    pub(crate) fn y(&self) -> i32 {
        self.position.y
    }
    pub fn w(&self) -> i32 {
        self.size.w
    }
    pub fn h(&self) -> i32 {
        self.size.h
    }
}

impl Bounds {
    pub fn contract(&self, i: i32) -> Bounds {
        Bounds {
            position: Point {
                x: self.position.x + i,
                y: self.position.y + i,
            },
            size: Size {
                w: self.size.w - i - i,
                h: self.size.h - i - i,
            },
        }
    }
}

impl Bounds {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Bounds {
        Bounds {
            position: Point::new(x, y),
            size: Size::new(w, h),
        }
    }
    pub fn new_from(position: Point, size: Size) -> Bounds {
        Bounds { position, size }
    }
}
impl Default for Bounds {
    fn default() -> Self {
        Bounds::new(0, 0, 100, 100)
    }
}
impl Add<Point> for Bounds {
    type Output = Bounds;

    fn add(self, rhs: Point) -> Self::Output {
        Bounds {
            position: self.position + rhs,
            size: self.size,
        }
    }
}
impl Sub<Point> for Bounds {
    type Output = Bounds;

    fn sub(self, rhs: Point) -> Self::Output {
        Bounds {
            position: self.position - rhs,
            size: self.size,
        }
    }
}
impl Add<Insets> for Bounds {
    type Output = Bounds;

    fn add(self, rhs: Insets) -> Self::Output {
        Bounds {
            position: Point {
                x: self.position.x - rhs.left,
                y: self.position.y - rhs.top,
            },
            size: Size {
                w: self.size.w + rhs.left + rhs.right,
                h: self.size.h + rhs.top + rhs.bottom,
            },
        }
    }
}
impl Sub<Insets> for Bounds {
    type Output = Bounds;

    fn sub(self, rhs: Insets) -> Self::Output {
        Bounds {
            position: Point {
                x: self.position.x + rhs.left,
                y: self.position.y + rhs.top,
            },
            size: Size {
                w: self.size.w - rhs.left - rhs.right,
                h: self.size.h - rhs.top - rhs.bottom,
            },
        }
    }
}
impl Display for Bounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{},{} {}x{}",
            self.position.x, self.position.y, self.size.w, self.size.h
        )
    }
}
impl Bounds {
    pub fn union(&self, b: Bounds) -> Bounds {
        if self.is_empty() {
            return b;
        }
        if b.is_empty() {
            return *self;
        }
        Bounds::from_xyxy2(
            self.position.x.min(b.position.x),
            self.position.y.min(b.position.y),
            self.x2().max(b.x2()),
            self.y2().max(b.y2()),
        )
    }
    pub(crate) fn center(&self) -> Point {
        Point::new(
            self.position.x + self.size.w / 2,
            self.position.y + self.size.h / 2,
        )
    }
    fn from_xyxy2(x: i32, y: i32, x2: i32, y2: i32) -> Bounds {
        Bounds {
            position: Point::new(x, y),
            size: Size::new(x2 - x, y2 - y),
        }
    }
    pub fn x2(&self) -> i32 {
        self.position.x + self.size.w
    }
    pub fn y2(&self) -> i32 {
        self.position.y + self.size.h
    }
    pub fn center_at(&self, x: i32, y: i32) -> Bounds {
        Bounds {
            position: Point::new(x - self.size.w / 2, y - self.size.h / 2),
            size: self.size,
        }
    }
}

impl Bounds {
    pub fn new_empty() -> Bounds {
        Bounds {
            position: Point::zero(),
            size: Size::empty(),
        }
    }
}

impl Bounds {
    pub fn is_empty(&self) -> bool {
        self.size.w < 1 || self.size.h < 1
    }
    pub fn contains(&self, pt: &Point) -> bool {
        if self.position.x <= pt.x && self.position.y <= pt.y {
            if self.position.x + self.size.w > pt.x && self.position.y + self.size.h > pt.y {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Bounds, Insets, Point, Size};
    use alloc::format;

    #[test]
    fn points() {
        let p1 = Point::new(20, 30);
        let p2 = Point::new(40, 50);
        assert_eq!(p1 + p2, Point::new(60, 80));
        assert_eq!(p1 - p2, Point::new(-20, -20));
        assert_eq!(format!("{}", Point::new(20, 30)), "20,30");
    }
    #[test]
    fn sizes() {
        let a = Size::new(10, 10);
        let b = Size::new(10, 10);
        let c = Size::new(20, 20);
        assert_eq!(a + b, c);
        let d = Insets::new_same(10);
        assert_eq!(a + d, Size::new(30, 30));
        assert_eq!(format!("{}", Size::new(10, 20)), "10x20");
    }
    #[test]
    fn bounds() {
        let a = Bounds::new(10, 20, 30, 40);
        let pt = Point::new(1, 1);
        assert_eq!(format!("{}", a), "10,20 30x40");
        assert_eq!(a + pt, Bounds::new(11, 21, 30, 40));
        assert_eq!(a - pt, Bounds::new(9, 19, 30, 40));
        assert_eq!(
            Bounds::new_from(Point::new(5, 6), Size::new(7, 8)),
            Bounds::new(5, 6, 7, 8)
        );
    }

    #[test]
    fn intersects() {
        let a = Bounds::new(0, 0, 50, 50);
        let b = Bounds::new(100, 0, 50, 50);
        let c = Bounds::new(25, 0, 50, 50);
        assert!(!a.intersects(b));
        assert!(a.intersects(c));
    }

    #[test]
    fn test_geometry() {
        let bounds = Bounds::new(0, 0, 100, 100);
        assert_eq!(bounds.contains(&Point::new(10, 10)), true);
        assert_eq!(bounds.contains(&Point::new(-1, -1)), false);

        let b2 = Bounds::new(140, 180, 80, 30);
        let b3 = Bounds::new(140, 180, 80, 30);
        // INFO - union Bounds { x: 140, y: 180, w: 80, h: 30 } Bounds { x: 140, y: 180, w: 80, h: 30 }
        assert_eq!(b2.union(b3), b2.clone());
    }
    #[test]
    fn test_point() {
        let pt1 = Point::new(8, 9);
        let pt2 = Point::new(10, 11);
        let pt3 = pt1 + pt2;
        assert_eq!(pt3, Point::new(18, 20));
        let bounds = Bounds::new(1, 2, 3, 4);
        assert_eq!(bounds.position, Point::new(1, 2));

        let pt4 = pt1 - pt2;
        assert_eq!(pt4, Point::new(-2, -2));
    }
}
