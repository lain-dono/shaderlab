use crate::builder::{FunctionBuilder, NodeBuilder};
use crate::node::{Event, Node, NodeDescriptor, Output, PortId};
use arrayvec::ArrayVec;
use naga::{
    BinaryOperator, Expression, Handle, MathFunction, SwizzleComponent, UnaryOperator, VectorSize,
};

pub struct Math {
    fun: MathFunction,
    args: ArrayVec<Option<Output>, 4>,
}

impl Math {
    pub fn new(fun: MathFunction) -> Self {
        let mut args = ArrayVec::new();
        for _ in 0..fun.argument_count() {
            args.push(None);
        }
        Self { fun, args }
    }

    pub fn set(&mut self, index: usize, value: impl Into<Option<Output>>) {
        self.args[index] = value.into();
    }
}

impl NodeBuilder for Math {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port() != PortId(0) {
            return None;
        }

        let arg = function.node(self.args[0]?)?;

        let arg1 = if let Some(port) = self.args.get(1) {
            Some(function.node((*port)?)?)
        } else {
            None
        };

        let arg2 = if let Some(port) = self.args.get(2) {
            Some(function.node((*port)?)?)
        } else {
            None
        };

        let arg3 = if let Some(port) = self.args.get(3) {
            Some(function.node((*port)?)?)
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

macro_rules! emit_binary {
    ($name:ident :: $fn:ident => $op:ident) => { emit_binary!($name::$fn => $op:V4F); };
    ($name:ident :: $fn:ident => $op:ident : $ty:ident) => {
        #[derive(Default, Debug)]
        pub struct $name([Option<Output>; 2]);

        impl Node for $name {
            fn desc(&self) -> NodeDescriptor<'_> {
                NodeDescriptor {
                    label: stringify!($fn),
                    width: 75,
                    inputs: &[("a", super::Type::V4F), ("b", super::Type::V4F)],
                    outputs: &[("out", super::Type::V4F)],
                }
            }

            fn update(&mut self, event: Event) {
                match event {
                    Event::AttachInput(PortId(port), remote) => self.0[port] = remote,
                    _ => (),
                }
            }
        }

        impl NodeBuilder for $name {
            fn expr(
                &self,
                function: &mut FunctionBuilder,
                output: Output,
            ) -> Option<Handle<Expression>> {
                if output.port() == PortId(0) {
                    let left = function.node(self.0[0]?)?;
                    let right = function.node(self.0[1]?)?;
                    let op = BinaryOperator::$op;
                    Some(function.emit(Expression::Binary { op, left, right }))
                } else {
                    None
                }
            }
        }
    };
}

emit_binary!(Add::add => Add);
emit_binary!(Sub::sub => Subtract);
emit_binary!(Mul::mul => Multiply);
emit_binary!(Div::div => Divide);
emit_binary!(Rem::rem => Modulo);

macro_rules! emit_unary {
    ($name:ident :: $fn:ident => $op:ident) => { emit_unary!($name::$fn => $op:V4F); };
    ($name:ident :: $fn:ident => $op:ident : $ty:ident) => {
        #[derive(Default, Debug)]
        pub struct $name(Option<Output>);

        impl Node for $name {
            fn desc(&self) -> NodeDescriptor<'_> {
                NodeDescriptor {
                    label: stringify!($fn),
                    width: 75,
                    inputs: &[("x", super::Type::V4F)],
                    outputs: &[("out", super::Type::V4F)],
                }
            }

            fn update(&mut self, event: Event) {
                match event {
                    Event::AttachInput(PortId(0), remote) => self.0 = remote,
                    _ => (),
                }
            }
        }

        impl NodeBuilder for $name {
            fn expr(
                &self,
                function: &mut FunctionBuilder,
                output: Output,
            ) -> Option<Handle<Expression>> {
                assert_eq!(output.port(), PortId(0));
                let expr = function.node(self.0?)?;
                let op = UnaryOperator::$op;
                Some(function.emit(Expression::Unary { op, expr }))
            }
        }
    };
}

emit_unary!(Negate::negate => Negate);
emit_unary!(Not::not => Not);

pub struct Unary {
    op: UnaryOperator,
    expr: Option<Output>,
}

impl NodeBuilder for Unary {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port() == PortId(0) {
            let expr = function.node(self.expr?)?;
            let op = self.op;
            Some(function.emit(Expression::Unary { op, expr }))
        } else {
            None
        }
    }
}

pub struct Swizzle {
    pub size: VectorSize,
    pub vector: Option<Output>,
    pub pattern: [SwizzleComponent; 4],
}

impl NodeBuilder for Swizzle {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port() == PortId(0) {
            let vector = function.node(self.vector?)?;
            Some(function.emit(Expression::Swizzle {
                size: self.size,
                vector,
                pattern: self.pattern,
            }))
        } else {
            None
        }
    }
}

macro_rules! custom {
    ($name:ident :: $fn:ident ($($arg:ident),+) -> $ret:ident ($format:literal, $($arg_expr:expr),+)) => {
        #[derive(Clone, Debug, Default)]
        pub struct $name;

        impl NodeBuilder for $name {
            fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>> {
                None
            }
        }

        impl Node for $name {
            fn desc(&self) -> NodeDescriptor<'_> {
                NodeDescriptor {
                    label: stringify!($fn),
                    width: 75,
                    inputs: &[$((stringify!($arg), super::Type::V4F)),+],
                    outputs: &[(stringify!($ret), super::Type::V4F)]
                }
            }
        }
    };
}

macro_rules! math_fn {
    ($name:ident :: $fn:ident($a:ident))
    => { custom!($name::$fn($a) -> out ("let {} = {}({});", out, stringify!($fn), $a)); };
    ($name:ident :: $fn:ident($a:ident, $b:ident))
    => { custom!($name::$fn($a, $b) -> out ("let {} = {}({}, {});", out, stringify!($fn), $a, $b)); };
}

// Advanced

math_fn!(Abs::abs(x));
math_fn!(Exp::exp(x));
math_fn!(Exp2::exp2(x));
math_fn!(Length::length(x));
math_fn!(Log::log(x));
// negate?
math_fn!(Normalize::normalize(x));
// posterize
// recip
// recip_sqrt/invsqrt

// Basic
//custom!(Add::add(a, b) -> out ("let {} = {} {} {};", out, a, "+", b));
math_fn!(Pow::pow(x, y));
math_fn!(Sqrt::sqrt(x));

// Trigonometry

math_fn!(Acos::acos(x));
math_fn!(Asin::asin(x));
math_fn!(Atan::atan(x));
math_fn!(Atan2::atan2(y, x));
math_fn!(Cos::cos(x));
math_fn!(Sin::sin(x));
math_fn!(Tan::tan(x));
math_fn!(Cosh::cosh(x));
math_fn!(Sinh::sinh(x));
math_fn!(Tanh::tanh(x));

custom!(RadiansToDegres::to_degres(a) -> out ("let {} = {} * 57.29578;", out, a));
custom!(DegresToRadians::to_radians(a) -> out ("let {} = {} * 0.017453292;", out, a));

// Rounding
math_fn!(Ceil::ceil(x));
math_fn!(Floor::floor(x));
math_fn!(Round::round(x));
math_fn!(Sign::sign(x));
math_fn!(Trunc::trunc(x));
math_fn!(Step::step(x, y));

// Derivative
math_fn!(DpDx::dpdx(e));
math_fn!(DpDy::dpdy(e));
//math_fn!(DPDXY::dpdxy(e));
