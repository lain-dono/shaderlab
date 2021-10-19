use naga::{
    front::Typifier,
    proc::{ResolveContext, ResolveError},
    Expression, Function, Handle, Module, ScalarKind, SwizzleComponent, Type, TypeInner,
    VectorSize,
};

const fn scalar(kind: ScalarKind, width: u8) -> Type {
    Type {
        name: None,
        inner: TypeInner::Scalar { kind, width },
    }
}

const fn vector(kind: ScalarKind, size: VectorSize, width: u8) -> Type {
    Type {
        name: None,
        inner: TypeInner::Vector { size, kind, width },
    }
}

pub const U32: Type = scalar(ScalarKind::Uint, 4);
pub const U32_2: Type = vector(ScalarKind::Uint, VectorSize::Bi, 4);
pub const U32_3: Type = vector(ScalarKind::Uint, VectorSize::Tri, 4);
pub const U32_4: Type = vector(ScalarKind::Uint, VectorSize::Quad, 4);

pub const I32: Type = scalar(ScalarKind::Sint, 4);
pub const I32_2: Type = vector(ScalarKind::Sint, VectorSize::Bi, 4);
pub const I32_3: Type = vector(ScalarKind::Sint, VectorSize::Tri, 4);
pub const I32_4: Type = vector(ScalarKind::Sint, VectorSize::Quad, 4);

pub const F32: Type = scalar(ScalarKind::Float, 4);
pub const F32_2: Type = vector(ScalarKind::Float, VectorSize::Bi, 4);
pub const F32_3: Type = vector(ScalarKind::Float, VectorSize::Tri, 4);
pub const F32_4: Type = vector(ScalarKind::Float, VectorSize::Quad, 4);

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

pub fn is_scalar_or_vector(ty: &TypeInner) -> Option<VectorKind> {
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

pub enum MatrixKind {
    M2,
    M3,
    M4,
}

pub enum VectorKind {
    V1,
    V2,
    V3,
    V4,
}

pub fn resolve_vector(
    function: &mut super::FunctionBuilder,
    expr: Handle<Expression>,
    src: VectorKind,
    dst: VectorKind,
) -> Handle<Expression> {
    use super::expr::Emit;
    use VectorKind::*;

    let (zero, one) = (0f64.emit(function), 1f64.emit(function));
    let pattern = [
        SwizzleComponent::X,
        SwizzleComponent::Y,
        SwizzleComponent::Z,
        SwizzleComponent::W,
    ];

    match (src, dst) {
        // same
        (V1, V1) | (V2, V2) | (V3, V3) | (V4, V4) => expr,

        // promoting
        (V1, V2) => [expr, zero].emit(function),
        (V1, V3) => [expr, zero, zero].emit(function),
        (V1, V4) => [expr, zero, zero, one].emit(function),
        (V2, V3) => [expr, zero].emit(function),
        (V2, V4) => [expr, zero, one].emit(function),
        (V3, V4) => [expr, one].emit(function),

        // truncating
        (V2, V1) | (V3, V1) | (V4, V1) => function.emit(Expression::AccessIndex {
            base: expr,
            index: 0,
        }),
        (V3, V2) | (V4, V2) => function.emit(Expression::Swizzle {
            size: VectorSize::Bi,
            vector: expr,
            pattern,
        }),
        (V4, V3) => function.emit(Expression::Swizzle {
            size: VectorSize::Tri,
            vector: expr,
            pattern,
        }),
    }
}
