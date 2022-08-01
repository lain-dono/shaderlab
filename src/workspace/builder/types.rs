use super::expr::{Emit, EmitResult};
use super::FnBuilder;
use naga::{
    front::Typifier,
    proc::{ResolveContext, ResolveError},
    Expression, Function, Handle, Module, ScalarKind, SwizzleComponent, Type, TypeInner,
    VectorSize,
};

pub struct BaseTypes {
    pub bool: Handle<Type>,
    pub bool2: Handle<Type>,
    pub bool3: Handle<Type>,
    pub bool4: Handle<Type>,

    pub u32: Handle<Type>,
    pub u32x2: Handle<Type>,
    pub u32x3: Handle<Type>,
    pub u32x4: Handle<Type>,

    pub i32: Handle<Type>,
    pub i32x2: Handle<Type>,
    pub i32x3: Handle<Type>,
    pub i32x4: Handle<Type>,

    pub f32: Handle<Type>,
    pub f32x2: Handle<Type>,
    pub f32x3: Handle<Type>,
    pub f32x4: Handle<Type>,
}

impl BaseTypes {
    pub fn new(module: &mut super::ModuleBuilder) -> Self {
        const fn scalar(kind: ScalarKind, width: u8) -> Type {
            let inner = TypeInner::Scalar { kind, width };
            Type { name: None, inner }
        }

        const fn vector(kind: ScalarKind, size: VectorSize, width: u8) -> Type {
            let inner = TypeInner::Vector { size, kind, width };
            Type { name: None, inner }
        }

        Self {
            bool: module.insert_type(scalar(ScalarKind::Bool, naga::BOOL_WIDTH)),
            bool2: module.insert_type(vector(ScalarKind::Bool, VectorSize::Bi, naga::BOOL_WIDTH)),
            bool3: module.insert_type(vector(ScalarKind::Bool, VectorSize::Tri, naga::BOOL_WIDTH)),
            bool4: module.insert_type(vector(ScalarKind::Bool, VectorSize::Quad, naga::BOOL_WIDTH)),

            u32: module.insert_type(scalar(ScalarKind::Uint, 4)),
            u32x2: module.insert_type(vector(ScalarKind::Uint, VectorSize::Bi, 4)),
            u32x3: module.insert_type(vector(ScalarKind::Uint, VectorSize::Tri, 4)),
            u32x4: module.insert_type(vector(ScalarKind::Uint, VectorSize::Quad, 4)),

            i32: module.insert_type(scalar(ScalarKind::Sint, 4)),
            i32x2: module.insert_type(vector(ScalarKind::Sint, VectorSize::Bi, 4)),
            i32x3: module.insert_type(vector(ScalarKind::Sint, VectorSize::Tri, 4)),
            i32x4: module.insert_type(vector(ScalarKind::Sint, VectorSize::Quad, 4)),

            f32: module.insert_type(scalar(ScalarKind::Float, 4)),
            f32x2: module.insert_type(vector(ScalarKind::Float, VectorSize::Bi, 4)),
            f32x3: module.insert_type(vector(ScalarKind::Float, VectorSize::Tri, 4)),
            f32x4: module.insert_type(vector(ScalarKind::Float, VectorSize::Quad, 4)),
        }
    }
}

pub fn extract_type<'a>(
    ifier: &'a mut Typifier,
    module: &'a Module,
    function: &'a Function,
    expr_handle: Handle<Expression>,
) -> Result<&'a TypeInner, ResolveError> {
    let ctx = ResolveContext {
        constants: &module.constants,
        types: &module.types,
        global_vars: &module.global_variables,
        local_vars: &function.local_variables,
        functions: &module.functions,
        arguments: &function.arguments,
    };
    ifier.grow(expr_handle, &function.expressions, &ctx)?;
    Ok(ifier.get(expr_handle, &module.types))
}

/*
pub fn is_scalar(ty: &TypeInner) -> Option<ScalarKind> {
    match ty {
        TypeInner::Scalar { kind, .. } => Some(*kind),
        _ => None,
    }
}

pub fn is_vector(ty: &TypeInner) -> Option<(ScalarKind, VectorSize)> {
    match ty {
        TypeInner::Vector { kind, size, .. } => Some((*kind, *size)),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub enum MatrixKind {
    M2,
    M3,
    M4,
}
*/

#[derive(Clone, Copy)]
pub enum VectorKind {
    V1,
    V2,
    V3,
    V4,
}

impl VectorKind {
    pub fn parse(ty: &TypeInner) -> Option<Self> {
        match ty {
            TypeInner::Scalar { .. } => Some(VectorKind::V1),
            TypeInner::Vector { size, .. } => Some(match size {
                VectorSize::Bi => VectorKind::V2,
                VectorSize::Tri => VectorKind::V3,
                VectorSize::Quad => VectorKind::V4,
            }),
            _ => None,
        }
    }
}

impl VectorKind {
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (Self::V1, _) | (_, Self::V1) => Self::V1,
            (Self::V2, _) | (_, Self::V2) => Self::V2,
            (Self::V3, _) | (_, Self::V3) => Self::V3,
            (Self::V4, Self::V4) => Self::V4,
        }
    }

    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Self::V4, _) | (_, Self::V4) => Self::V4,
            (Self::V3, _) | (_, Self::V3) => Self::V3,
            (Self::V2, _) | (_, Self::V2) => Self::V2,
            (Self::V1, Self::V1) => Self::V1,
        }
    }

    pub fn splat<T: Emit + Copy>(&self, function: &mut FnBuilder, value: T) -> EmitResult {
        match self {
            VectorKind::V1 => value.emit(function),
            VectorKind::V2 => [value, value].emit(function),
            VectorKind::V3 => [value, value, value].emit(function),
            VectorKind::V4 => [value, value, value, value].emit(function),
        }
    }
}

impl<'a, 'storage> FnBuilder<'a, 'storage> {
    pub fn resolve_vector(
        &mut self,
        expr: Handle<Expression>,
        src: VectorKind,
        dst: VectorKind,
    ) -> EmitResult {
        use super::expr::Float;
        use VectorKind::*;

        let function = self;

        let zero = Float(0.0).emit(function)?;
        let one = Float(1.0).emit(function)?;

        macro_rules! swizzle {
            ($x:ident, $y:ident, $z:ident, $w:ident) => {
                [
                    swizzle!(@$x),
                    swizzle!(@$y),
                    swizzle!(@$z),
                    swizzle!(@$w),
                ]
            };

            (@x) => { SwizzleComponent::X };
            (@y) => { SwizzleComponent::Y };
            (@z) => { SwizzleComponent::Z };
            (@w) => { SwizzleComponent::W };
        }

        macro_rules! expr_swizzle {
            ($vector:expr, $size:ident, [$x:ident, $y:ident, $z:ident, $w:ident]) => {
                Expression::Swizzle {
                    vector: $vector,
                    size: naga::VectorSize::$size,
                    pattern: swizzle!($x, $y, $z, $w),
                }
            };
        }

        Ok(match (src, dst) {
            // same
            (V1, V1) | (V2, V2) | (V3, V3) | (V4, V4) => expr,

            // promoting
            (V1, V2) => [expr, zero].emit(function)?,
            (V1, V3) => [expr, zero, zero].emit(function)?,
            (V1, V4) => [expr, zero, zero, one].emit(function)?,

            (V2, V3) => {
                let x = function.access_index(expr, 0);
                let y = function.access_index(expr, 1);
                [x, y, zero].emit(function)?
            }
            (V2, V4) => {
                let x = function.access_index(expr, 0);
                let y = function.access_index(expr, 1);
                [x, y, zero, one].emit(function)?
            }

            (V3, V4) => {
                let x = function.access_index(expr, 0);
                let y = function.access_index(expr, 1);
                let z = function.access_index(expr, 2);
                [x, y, z, one].emit(function)?
            }

            // truncating
            (V2, V1) | (V3, V1) | (V4, V1) => function.access_index(expr, 0),
            (V3, V2) | (V4, V2) => function.emit(expr_swizzle!(expr, Bi, [x, y, z, w])),
            (V4, V3) => function.emit(expr_swizzle!(expr, Tri, [x, y, z, w])),
        })
    }
}
