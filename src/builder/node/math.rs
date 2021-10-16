use crate::builder::{
    node::{NodeArena, NodeBuilder, Port},
    FunctionBuilder,
};
use arrayvec::ArrayVec;
use naga::{
    BinaryOperator, Expression, Handle, MathFunction, SwizzleComponent, UnaryOperator, VectorSize,
};

pub struct Math {
    fun: MathFunction,
    args: ArrayVec<Option<Port>, 4>,
}

impl Math {
    pub fn new(fun: MathFunction) -> Self {
        let mut args = ArrayVec::new();
        for _ in 0..fun.argument_count() {
            args.push(None);
        }
        Self { fun, args }
    }

    pub fn set(&mut self, index: usize, value: impl Into<Option<Port>>) {
        self.args[index] = value.into();
    }
}

impl NodeBuilder for Math {
    fn expr(
        &self,
        nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        if output != 0 {
            return None;
        }

        let arg = nodes.expr(function, self.args[0]?)?;

        let arg1 = if let Some(port) = self.args.get(1) {
            Some(nodes.expr(function, (*port)?)?)
        } else {
            None
        };

        let arg2 = if let Some(port) = self.args.get(2) {
            Some(nodes.expr(function, (*port)?)?)
        } else {
            None
        };

        let arg3 = if let Some(port) = self.args.get(3) {
            Some(nodes.expr(function, (*port)?)?)
        } else {
            None
        };

        Some(function.emit(Expression::Math {
            fun: self.fun,
            arg,
            arg1,
            arg2,
            arg3,
        }))
    }
}

pub struct Binary {
    op: BinaryOperator,
    left: Option<Port>,
    right: Option<Port>,
}

impl NodeBuilder for Binary {
    fn expr(
        &self,
        nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        if output != 0 {
            return None;
        }
        let left = nodes.expr(function, self.left?)?;
        let right = nodes.expr(function, self.right?)?;
        let op = self.op;
        Some(function.emit(Expression::Binary { op, left, right }))
    }
}

pub struct Unary {
    op: UnaryOperator,
    expr: Option<Port>,
}

impl NodeBuilder for Unary {
    fn expr(
        &self,
        nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        if output != 0 {
            return None;
        }
        let expr = nodes.expr(function, self.expr?)?;
        let op = self.op;
        Some(function.emit(Expression::Unary { op, expr }))
    }
}

pub struct Swizzle {
    pub size: VectorSize,
    pub vector: Option<Port>,
    pub pattern: [SwizzleComponent; 4],
}

impl NodeBuilder for Swizzle {
    fn expr(
        &self,
        nodes: &NodeArena,
        function: &mut FunctionBuilder<'_>,
        output: usize,
    ) -> Option<Handle<Expression>> {
        if output != 0 {
            return None;
        }
        let vector = nodes.expr(function, self.vector?)?;
        Some(function.emit(Expression::Swizzle {
            size: self.size,
            vector,
            pattern: self.pattern,
        }))
    }
}
