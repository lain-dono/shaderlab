use super::super::builder::{expr::*, *};
use crate::workspace::Port;
use naga::Expression;

pub fn resolve_pair_min(
    function: &mut FnBuilder,
    a: Port,
    b: Port,
) -> EmitResult<(
    VectorKind,
    naga::Handle<Expression>,
    naga::Handle<Expression>,
)> {
    resolve_pair(function, a, b, VectorKind::min)
}

pub fn resolve_pair(
    function: &mut FnBuilder,
    a: Port,
    b: Port,
    merge: impl FnOnce(VectorKind, VectorKind) -> VectorKind,
) -> EmitResult<(
    VectorKind,
    naga::Handle<Expression>,
    naga::Handle<Expression>,
)> {
    let a = function.for_input(a)?;
    let b = function.for_input(b)?;

    let a_src = VectorKind::parse(function.extract_type(a)?).unwrap();
    let b_src = VectorKind::parse(function.extract_type(b)?).unwrap();

    let dst = merge(a_src, b_src);

    let a = function.resolve_vector(a, a_src, dst).unwrap();
    let b = function.resolve_vector(b, b_src, dst).unwrap();

    Ok((dst, a, b))
}
