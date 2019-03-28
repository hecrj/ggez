use crate::graphics::{Color, Matrix4, Rect};

use mint;

type Vec3 = nalgebra::Vector3<f32>;

/// A struct containing all the necessary info for drawing a [`Drawable`](trait.Drawable.html).
///
/// This struct implements the `Default` trait, so to set only some parameter
/// you can just do:
///
/// ```rust
/// # use ggez::*;
/// # use ggez::graphics::*;
/// # fn t<P>(ctx: &mut Context, drawable: &P) where P: Drawable {
/// let my_dest = nalgebra::Point2::new(13.0, 37.0);
/// graphics::draw(ctx, drawable, DrawParam::default().dest(my_dest) );
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// A portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image `(0,0 to 1,1)` if omitted.
    pub src: Rect,
    /// The position to draw the graphic expressed as a `Point2`.
    pub dest: nalgebra::Point2<f32>,
    /// The orientation of the graphic in radians.
    pub rotation: f32,
    /// The x/y scale factors expressed as a `Vector2`.
    pub scale: nalgebra::Vector2<f32>,
    /// An offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point2::new(0.5, 0.5)` here.
    pub offset: nalgebra::Point2<f32>,
    /// A color to draw the target with.
    /// Default: white.
    pub color: Color,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: nalgebra::Point2::new(0.0, 0.0),
            rotation: 0.0,
            scale: nalgebra::Vector2::new(1.0, 1.0),
            offset: nalgebra::Point2::new(0.0, 0.0),
            color: Color::WHITE,
        }
    }
}

impl DrawParam {
    /// Create a new DrawParam with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the source rect
    pub fn src(mut self, src: Rect) -> Self {
        self.src = src;
        self
    }

    /// Set the dest point
    pub fn dest<P>(mut self, dest: P) -> Self
    where
        P: Into<nalgebra::Point2<f32>>,
    {
        let p: nalgebra::Point2<f32> = dest.into();
        self.dest = p;
        self
    }

    /// Set the drawable color.  This will be blended with whatever
    /// color the drawn object already is.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the rotation of the drawable.
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scaling factors of the drawable.
    pub fn scale<V>(mut self, scale: V) -> Self
    where
        V: Into<nalgebra::Vector2<f32>>,
    {
        let p: nalgebra::Vector2<f32> = scale.into();
        self.scale = p;
        self
    }

    /// Set the transformation offset of the drawable.
    pub fn offset<P>(mut self, offset: P) -> Self
    where
        P: Into<nalgebra::Point2<f32>>,
    {
        let p: nalgebra::Point2<f32> = offset.into();
        self.offset = p;
        self
    }

    /// A [`DrawParam`](struct.DrawParam.html) that has been crunched down to a single matrix.
    pub fn into_matrix(&self) -> Matrix4 {
        let translate = Matrix4::new_translation(&Vec3::new(self.dest.x, self.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(self.offset.x, self.offset.y, 0.0));
        let offset_inverse =
            Matrix4::new_translation(&Vec3::new(-self.offset.x, -self.offset.y, 0.0));
        let axis_angle = Vec3::z() * self.rotation;
        let rotation = Matrix4::new_rotation(axis_angle);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(self.scale.x, self.scale.y, 1.0));
        translate * offset * rotation * scale * offset_inverse
    }
}
