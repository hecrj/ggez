use nalgebra;

/// A point
pub type Point2 = nalgebra::Point2<f32>;

/// A 2 dimensional vector representing an offset of a location
pub type Vector2 = nalgebra::Vector2<f32>;

/// A 4 dimensional matrix representing an arbitrary 3d transformation
pub type Matrix4 = nalgebra::Matrix4<f32>;

/// A simple 2D rectangle.
///
/// The origin of the rectangle is at the top-left,
/// with x increasing to the right and y increasing down.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Rect {
    /// X coordinate of the left edge of the rect.
    pub x: f32,
    /// Y coordinate of the top edge of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new `Rect`.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    /// Creates a new `Rect` a la Love2D's `love.graphics.newQuad`,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

    /// Create a new rect from `i32` coordinates.
    pub fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Create a new `Rect` with all values zero.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new `Rect` at `0,0` with width and height 1.
    pub fn one() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Gets the `Rect`'s x and y coordinates as a `Point2`.
    pub fn point(&self) -> mint::Point2<f32> {
        mint::Point2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<mint::Point2<f32>>,
    {
        let point = point.into();
        point.x >= self.left()
            && point.x <= self.right()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Translates the `Rect` by an offset of (x, y)
    pub fn translate<V>(&mut self, offset: V)
    where
        V: Into<mint::Vector2<f32>>,
    {
        let offset = offset.into();
        self.x += offset.x;
        self.y += offset.y;
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to<P>(&mut self, destination: P)
    where
        P: Into<mint::Point2<f32>>,
    {
        let destination = destination.into();
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.w *= sx;
        self.h *= sy;
    }

    /// Calculated the new Rect around the rotated one.
    pub fn rotate(&mut self, rotation: f32) {
        let rotation = nalgebra::Rotation2::new(rotation);
        let x0 = self.x;
        let y0 = self.y;
        let x1 = self.right();
        let y1 = self.bottom();
        let points = [
            rotation * nalgebra::Point2::new(x0, y0),
            rotation * nalgebra::Point2::new(x0, y1),
            rotation * nalgebra::Point2::new(x1, y0),
            rotation * nalgebra::Point2::new(x1, y1),
        ];
        let p0 = points[0];
        let mut x_max = p0.x;
        let mut x_min = p0.x;
        let mut y_max = p0.y;
        let mut y_min = p0.y;
        for p in &points {
            x_max = f32::max(x_max, p.x);
            x_min = f32::min(x_min, p.x);
            y_max = f32::max(y_max, p.y);
            y_min = f32::min(y_min, p.y);
        }
        *self = Rect {
            w: x_max - x_min,
            h: y_max - y_min,
            x: x_min,
            y: y_min,
        }
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect { x, y, w, h }
    }
}

impl approx::AbsDiffEq for Rect {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.x, &other.x, epsilon)
            && f32::abs_diff_eq(&self.y, &other.y, epsilon)
            && f32::abs_diff_eq(&self.w, &other.w, epsilon)
            && f32::abs_diff_eq(&self.h, &other.h, epsilon)
    }
}

impl approx::RelativeEq for Rect {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f32::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && f32::relative_eq(&self.y, &other.y, epsilon, max_relative)
            && f32::relative_eq(&self.w, &other.w, epsilon, max_relative)
            && f32::relative_eq(&self.h, &other.h, epsilon, max_relative)
    }
}

impl From<[f32; 4]> for Rect {
    fn from(val: [f32; 4]) -> Self {
        Rect::new(val[0], val[1], val[2], val[3])
    }
}

impl From<Rect> for [f32; 4] {
    fn from(val: Rect) -> Self {
        [val.x, val.y, val.w, val.h]
    }
}
