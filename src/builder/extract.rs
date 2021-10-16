use naga::{
    front::Typifier,
    proc::{ResolveContext, ResolveError},
    Expression, Function, Handle, Module, ScalarKind, TypeInner, VectorSize,
};

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

pub fn is_scalar_or_vector_i32(ty: &TypeInner) -> bool {
    matches!(is_scalar_or_vector(ty), Some((ScalarKind::Uint, _)))
}

pub fn is_scalar_or_vector_u32(ty: &TypeInner) -> bool {
    matches!(is_scalar_or_vector(ty), Some((ScalarKind::Uint, _)))
}

pub fn is_scalar_or_vector_f32(ty: &TypeInner) -> bool {
    matches!(is_scalar_or_vector(ty), Some((ScalarKind::Float, _)))
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

pub fn is_scalar_or_vector(ty: &TypeInner) -> Option<(ScalarKind, usize)> {
    match ty {
        TypeInner::Scalar { kind, .. } => Some((*kind, 1)),
        TypeInner::Vector { kind, size, .. } => Some((
            *kind,
            match size {
                VectorSize::Bi => 2,
                VectorSize::Tri => 3,
                VectorSize::Quad => 4,
            },
        )),
        _ => None,
    }
}
