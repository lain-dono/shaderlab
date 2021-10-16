use crate::builder::{
    node::{NodeArena, NodeBuilder},
    FunctionBuilder, F32_2, F32_3, F32_4,
};
use naga::{Constant, ConstantInner, Expression, Handle, ScalarValue, Type};

pub struct Float {
    value: f64,
}

impl Float {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl NodeBuilder for Float {
    fn expr(
        &self,
        _nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        (output == 0).then(|| append_float(function, self.value))
    }
}

pub struct Vec2 {
    value: [f64; 2],
}

impl Vec2 {
    pub fn new(value: [f64; 2]) -> Self {
        Self { value }
    }
}

impl NodeBuilder for Vec2 {
    fn expr(
        &self,
        _nodes: &NodeArena,
        fun: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        (output == 0).then(|| vector(fun, F32_2, &self.value))
    }
}

pub struct Vec3 {
    value: [f64; 3],
}

impl Vec3 {
    pub fn new(value: [f64; 3]) -> Self {
        Self { value }
    }
}

impl NodeBuilder for Vec3 {
    fn expr(
        &self,
        _nodes: &NodeArena,
        fun: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        (output == 0).then(|| vector(fun, F32_3, &self.value))
    }
}

pub struct Vec4 {
    value: [f64; 4],
}

impl Vec4 {
    pub fn new(value: [f64; 4]) -> Self {
        Self { value }
    }
}

impl NodeBuilder for Vec4 {
    fn expr(
        &self,
        _nodes: &NodeArena,
        fun: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        (output == 0).then(|| vector(fun, F32_4, &self.value))
    }
}

fn vector(fun: &mut FunctionBuilder, ty: Type, value: &[f64]) -> Handle<Expression> {
    let ty = fun.insert_type(ty);
    let components = value
        .iter()
        .copied()
        .map(|value| append_float(fun, value))
        .collect();
    fun.emit(Expression::Compose { ty, components })
}

fn append_float(fun: &mut FunctionBuilder, value: f64) -> Handle<Expression> {
    let value = ScalarValue::Float(value);
    let handle = fun.module().append_constant(Constant {
        name: None,
        specialization: None,
        inner: ConstantInner::Scalar { width: 4, value },
    });
    fun.append_expression(Expression::Constant(handle))
}

pub struct Triangle;

impl NodeBuilder for Triangle {
    fn expr(
        &self,
        _nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        if output != 0 {
            return None;
        }

        use crate::builder::expr::*;

        let vertex_index = function.argument(0);
        let x = float(sub(sint(vertex_index), 1i64));
        let y = float(sub(mul(sint(and(vertex_index, 1u64)), 2i64), 1i64));

        Some(
            [
                Let::new("x", x).emit(function),
                Let::new("y", y).emit(function),
                (0.0f64).emit(function),
                (1.0f64).emit(function),
            ]
            .emit(function),
        )
    }
}
