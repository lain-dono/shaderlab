use egui::Vec2;

#[derive(Clone, Copy)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Matrix {
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);

    pub const fn new(a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32) -> Self {
        Self { a, b, c, d, tx, ty }
    }

    /*
    pub fn from_transform(tr: Transform) -> Self {
        let a =  (tr.rotation + tr.skew.y).cos() * tr.scale.x;
        let b =  (tr.rotation + tr.skew.y).sin() * tr.scale.x;
        let c = -(tr.rotation - tr.skew.x).sin() * tr.scale.y;
        let d =  (tr.rotation - tr.skew.x).cos() * tr.scale.y;

        let tx = tr.position.x - (tr.pivot.x * a + tr.pivot.y * c);
        let ty = tr.position.y - (tr.pivot.x * b + tr.pivot.y * d);

        Self::new(a, b, c, d, tx, ty)
    }
    */

    pub const fn from_array([a, b, c, d, tx, ty]: [f32; 6]) -> Self {
        Self { a, b, c, d, tx, ty }
    }

    pub const fn to_array(self) -> [f32; 6] {
        let Self { a, b, c, d, tx, ty } = self;
        [a, b, c, d, tx, ty]
    }

    pub const fn to_mat3(self) -> [[f32; 3]; 3] {
        [
            [self.a, self.c, self.tx],
            [self.b, self.d, self.ty],
            [0.0, 0.0, 1.0],
        ]
    }

    pub const fn to_mat3_transposed(self) -> [[f32; 3]; 3] {
        [
            [self.a, self.b, 0.0],
            [self.c, self.d, 0.0],
            [self.tx, self.ty, 1.0],
        ]
    }

    pub fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }

    pub fn apply_inv(&self, x: f32, y: f32) -> [f32; 2] {
        let id = (self.a * self.d + self.c * -self.b).recip();
        [
            self.d * id * x - self.c * id * y + (self.ty * self.c - self.tx * self.d) * id,
            self.a * id * y - self.b * id * x + (self.tx * self.b - self.ty * self.a) * id,
        ]
    }

    pub fn translate(self, x: f32, y: f32) -> Self {
        Self {
            tx: self.tx + x,
            ty: self.ty + y,
            ..self
        }
    }

    pub fn scale(self, x: f32, y: f32) -> Self {
        Self {
            a: self.a * x,
            b: self.b * y,
            c: self.c * x,
            d: self.d * y,
            tx: self.tx * x,
            ty: self.ty * y,
        }
    }

    pub fn rotate(self, angle: f32) -> Self {
        let (sn, cs) = angle.sin_cos();
        Self {
            a: self.a * cs - self.b * sn,
            b: self.a * sn + self.b * cs,
            c: self.c * cs - self.d * sn,
            d: self.c * sn + self.d * cs,
            tx: self.tx * cs - self.ty * sn,
            ty: self.tx * sn + self.ty * cs,
        }
    }

    pub fn invert(&self) -> Self {
        let n = self.a * self.d - self.b * self.c;
        Self::new(
            self.d / n,
            -self.b / n,
            -self.c / n,
            self.a / n,
            (self.c * self.ty - self.d * self.tx) / n,
            (self.b * self.tx - self.a * self.ty) / n,
        )
    }

    /// Appends the given Matrix to this Matrix.
    pub fn append(self, rhs: Self) -> Self {
        Self::concat(self, rhs)
    }

    /// Prepends the given Matrix to this Matrix.
    pub fn prepend(self, lhs: Self) -> Self {
        Self::concat(lhs, self)
    }

    #[inline(always)]
    fn concat(rhs: Self, lhs: Self) -> Self {
        Self {
            a: rhs.a.mul_add(lhs.a, rhs.b * lhs.c),
            b: rhs.a.mul_add(lhs.b, rhs.b * lhs.d),
            c: rhs.c.mul_add(lhs.a, rhs.d * lhs.c),
            d: rhs.c.mul_add(lhs.b, rhs.d * lhs.d),
            tx: rhs.tx.mul_add(lhs.a, rhs.ty.mul_add(lhs.c, lhs.tx)),
            ty: rhs.tx.mul_add(lhs.b, rhs.ty.mul_add(lhs.d, lhs.ty)),
        }
    }

    /// Sets the matrix based on all the available properties
    pub fn compose(position: Vec2, pivot: Vec2, scale: Vec2, rotation: f32, skew: Vec2) -> Self {
        let (sx, cx) = (rotation + skew.y).sin_cos();
        let (sy, cy) = (rotation - skew.x).sin_cos();
        let cy = -cy;

        let a = cx * scale.x;
        let b = sx * scale.x;
        let c = cy * scale.y;
        let d = sy * scale.y;

        let tx = position.x - (pivot.x * a + pivot.y * c);
        let ty = position.y - (pivot.x * b + pivot.y * d);

        Self { a, b, c, d, tx, ty }
    }
}

pub struct Decomposed {
    /// The coordinate of the object relative to the local coordinates of the parent.
    pub position: Vec2,
    /// The scale factor of the object.
    pub scale: Vec2,
    /// The pivot point of the object that it rotates around.
    pub pivot: Vec2,
    /// The skew amount, on the x and y axis.
    pub skew: Vec2,
    /// The rotation amount.
    pub rotation: f32,
}

impl Decomposed {
    pub const IDENTITY: Self = Self {
        // on_change
        position: Vec2::new(0.0, 0.0),
        scale: Vec2::new(1.0, 1.0),
        pivot: Vec2::new(0.0, 0.0),
        // update_skew
        skew: Vec2::new(0.0, 0.0),
        rotation: 0.0,
    };

    pub fn compose(&self) -> Matrix {
        Matrix::compose(
            self.position,
            self.pivot,
            self.scale,
            self.rotation,
            self.skew,
        )
    }

    // Decomposes the matrix (x, y, scaleX, scaleY, and rotation) and sets the properties on to a transform.
    pub fn from_matrix(pivot: Vec2, matrix: Matrix) -> Self {
        use std::f32::consts::TAU;
        let Matrix { a, b, c, d, tx, ty } = matrix;

        // sort out rotation / skew..
        let (rotation, skew) = {
            let skew_x = -f32::atan2(-c, d);
            let skew_y = f32::atan2(b, a);

            let delta = (skew_x + skew_y).abs();
            let eps = 0.00001;

            if delta < eps || (TAU - delta).abs() < eps {
                (skew_y, Vec2::new(0.0, 0.0))
            } else {
                (0.0, Vec2::new(skew_x, skew_y))
            }
        };

        // next set scale
        let scale = Vec2 {
            x: (a * a + b * b).sqrt(),
            y: (c * c + d * d).sqrt(),
        };

        // next set position
        let position = Vec2 {
            x: tx + pivot.x * a + pivot.y * c,
            y: ty + pivot.x * b + pivot.y * d,
        };

        Self {
            position,
            scale,
            pivot,
            skew,
            rotation,
        }
    }
}
