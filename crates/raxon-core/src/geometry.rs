//! Value-type geometry primitives used throughout layout and rendering.
//!
//! All coordinates are `f32`. Screen geometry never needs `f64` precision, and
//! `f32` matches the native graphics APIs we target (Core Graphics, Android
//! `Canvas`/`View` bounds, wgpu), avoiding conversions at the backend boundary.
//!
//! Every type here is `Copy` and allocation-free.

/// A point in a 2D coordinate space, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// Horizontal offset, increasing rightward.
    pub x: f32,
    /// Vertical offset, increasing downward (matching native UI conventions).
    pub y: f32,
}

impl Point {
    /// The origin, `(0, 0)`.
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    /// Constructs a point.
    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    /// Translates this point by `dx`, `dy`.
    #[must_use]
    pub fn offset(self, dx: f32, dy: f32) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// A 2D size, in logical pixels. Width and height are expected to be
/// non-negative; the layout engine is responsible for clamping.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    /// Extent along the x axis.
    pub width: f32,
    /// Extent along the y axis.
    pub height: f32,
}

impl Size {
    /// A zero-area size.
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    /// Constructs a size.
    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    /// Returns the size shrunk by `insets` on every edge, clamped at zero so the
    /// result never goes negative.
    #[must_use]
    pub fn deflate(self, insets: EdgeInsets) -> Size {
        Size {
            width: (self.width - insets.horizontal()).max(0.0),
            height: (self.height - insets.vertical()).max(0.0),
        }
    }
}

/// An axis-aligned rectangle defined by its top-left `origin` and `size`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left corner.
    pub origin: Point,
    /// Width and height.
    pub size: Size,
}

impl Rect {
    /// A zero rectangle at the origin.
    pub const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    /// Constructs a rect from explicit coordinates.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// The x coordinate of the left edge.
    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    /// The y coordinate of the top edge.
    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    /// The x coordinate of the right edge.
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    /// The y coordinate of the bottom edge.
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    /// Whether `point` lies within this rect, inclusive of the top/left edges
    /// and exclusive of the bottom/right (standard hit-testing convention, so
    /// adjacent rects never both claim a pixel).
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.min_x()
            && point.x < self.max_x()
            && point.y >= self.min_y()
            && point.y < self.max_y()
    }

    /// Returns the rect inset on all sides by `insets`, used to derive a child's
    /// content box from its border box.
    #[must_use]
    pub fn inset(&self, insets: EdgeInsets) -> Rect {
        Rect {
            origin: self.origin.offset(insets.left, insets.top),
            size: self.size.deflate(insets),
        }
    }
}

/// Per-edge spacing (padding, margin, border widths), in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    /// Top edge.
    pub top: f32,
    /// Right edge.
    pub right: f32,
    /// Bottom edge.
    pub bottom: f32,
    /// Left edge.
    pub left: f32,
}

impl EdgeInsets {
    /// Zero on every edge.
    pub const ZERO: EdgeInsets = EdgeInsets {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// The same inset on all four edges.
    pub const fn all(value: f32) -> Self {
        EdgeInsets {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Symmetric insets: `vertical` on top/bottom, `horizontal` on left/right.
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        EdgeInsets {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Total horizontal inset (`left + right`).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical inset (`top + bottom`).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_offset() {
        assert_eq!(Point::new(1.0, 2.0).offset(3.0, -1.0), Point::new(4.0, 1.0));
    }

    #[test]
    fn size_deflate_clamps_at_zero() {
        let s = Size::new(10.0, 4.0);
        assert_eq!(s.deflate(EdgeInsets::all(2.0)), Size::new(6.0, 0.0));
        // Over-deflation never produces negative extents.
        assert_eq!(s.deflate(EdgeInsets::all(100.0)), Size::ZERO);
    }

    #[test]
    fn rect_edges() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.min_x(), 10.0);
        assert_eq!(r.min_y(), 20.0);
        assert_eq!(r.max_x(), 40.0);
        assert_eq!(r.max_y(), 60.0);
    }

    #[test]
    fn rect_contains_is_half_open() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Point::new(0.0, 0.0)), "top-left is inclusive");
        assert!(r.contains(Point::new(9.9, 9.9)));
        assert!(
            !r.contains(Point::new(10.0, 5.0)),
            "right edge is exclusive"
        );
        assert!(
            !r.contains(Point::new(5.0, 10.0)),
            "bottom edge is exclusive"
        );
        assert!(!r.contains(Point::new(-0.1, 5.0)));
    }

    #[test]
    fn rect_inset_moves_origin_and_shrinks() {
        let r = Rect::new(0.0, 0.0, 20.0, 20.0);
        let inner = r.inset(EdgeInsets::symmetric(2.0, 4.0));
        assert_eq!(inner.origin, Point::new(4.0, 2.0));
        assert_eq!(inner.size, Size::new(12.0, 16.0));
    }

    #[test]
    fn edge_insets_totals() {
        let e = EdgeInsets {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };
        assert_eq!(e.horizontal(), 6.0);
        assert_eq!(e.vertical(), 4.0);
    }
}
