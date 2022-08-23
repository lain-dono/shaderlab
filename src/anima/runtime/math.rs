use egui::{Pos2, Vec2};

#[derive(Clone, Copy)]
pub struct Transform {
    pub rotate: f32,
    pub translate: Pos2,
    pub scale: Vec2,
    pub shear: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            rotate: 0.0,
            translate: Pos2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            shear: Vec2::new(0.0, 0.0),
        }
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix {
        let (sx, cx) = (self.rotate - self.shear.x).sin_cos();
        let (sy, cy) = (self.rotate + self.shear.y).sin_cos();
        let sx = -sx;

        Matrix {
            a: cy * self.scale.x,
            b: sy * self.scale.x,
            c: sx * self.scale.y,
            d: cx * self.scale.y,
            tx: self.translate.x,
            ty: self.translate.y,
        }
    }

    pub fn inverse(&self) -> Self {
        Self {
            translate: (-self.translate.to_vec2()).to_pos2(),
            rotate: -self.rotate,
            scale: egui::vec2(self.scale.x.recip(), self.scale.y.recip()),
            shear: -self.shear,
        }
    }

    pub fn mul_transform(&self, transform: Self) -> Self {
        let (x, y) = transform.translate.into();
        Self {
            translate: self.to_matrix().apply(x, y).into(),
            rotate: self.rotate + transform.rotate,
            scale: self.scale * transform.scale,
            shear: self.shear + transform.shear,
        }
    }

    pub fn mul_vec2(&self, egui::Vec2 { x, y }: egui::Vec2) -> egui::Pos2 {
        self.to_matrix().apply(x, y).into()
    }
}

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

    pub fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        ]
    }

    pub fn apply_points(&self, points: &mut [Pos2]) {
        for p in points {
            *p = self.apply(p.x, p.y).into();
        }
    }

    pub fn apply_offset(&self, points: &mut [Pos2]) {
        for p in points {
            p.x += self.tx;
            p.y += self.ty;
        }
    }

    pub fn apply_inv(&self, x: f32, y: f32) -> [f32; 2] {
        let id = (self.a * self.d + self.c * -self.b).recip();
        [
            self.d * id * x - self.c * id * y + (self.ty * self.c - self.tx * self.d) * id,
            self.a * id * y - self.b * id * x + (self.tx * self.b - self.ty * self.a) * id,
        ]
    }

    pub fn invert(&self) -> Self {
        let n = self.a * self.d - self.b * self.c;
        Self {
            a: self.d / n,
            b: -self.b / n,
            c: -self.c / n,
            d: self.a / n,
            tx: (self.c * self.ty - self.d * self.tx) / n,
            ty: (self.b * self.tx - self.a * self.ty) / n,
        }
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
    fn concat(lhs: Self, rhs: Self) -> Self {
        Self {
            a: lhs.a.mul_add(rhs.a, lhs.b * rhs.c),
            b: lhs.a.mul_add(rhs.b, lhs.b * rhs.d),
            c: lhs.c.mul_add(rhs.a, lhs.d * rhs.c),
            d: lhs.c.mul_add(rhs.b, lhs.d * rhs.d),
            tx: lhs.tx.mul_add(rhs.a, lhs.ty.mul_add(rhs.c, rhs.tx)),
            ty: lhs.tx.mul_add(rhs.b, lhs.ty.mul_add(rhs.d, rhs.ty)),
        }
    }
}
