use crate::builder::FunctionBuilder;
use naga::{
    front::Typifier, BinaryOperator, Constant, ConstantInner, Expression, Handle, MathFunction,
    ScalarKind, ScalarValue, Type, TypeInner, UnaryOperator, VectorSize,
};

pub trait Emit: 'static {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression>;
}

pub struct FunctionArgument(pub u32);

impl Emit for FunctionArgument {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        function.append_expression(Expression::FunctionArgument(self.0))
    }
}

impl Emit for i64 {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        ScalarValue::Sint(*self).emit(function)
    }
}

impl Emit for u64 {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        ScalarValue::Uint(*self).emit(function)
    }
}

impl Emit for f64 {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        ScalarValue::Float(*self).emit(function)
    }
}

pub struct Let {
    name: String,
    expr: Box<dyn Emit>,
}

impl Let {
    pub fn new(name: impl Into<String>, expr: impl Emit) -> Self {
        Self {
            name: name.into(),
            expr: Box::new(expr),
        }
    }
}

impl Emit for Let {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let expr = self.expr.emit(function);
        function.insert_expression_name(expr, &self.name);
        expr
    }
}

impl Emit for Handle<Expression> {
    fn emit(&self, _function: &mut FunctionBuilder) -> Handle<Expression> {
        *self
    }
}

impl Emit for ScalarValue {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let value = *self;
        let expr = Expression::Constant(function.module.append_constant(Constant {
            name: None,
            specialization: None,
            inner: ConstantInner::Scalar { width: 4, value },
        }));
        function.append_expression(expr)
    }
}

impl<T: Emit> Emit for [T; 2] {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let components = vec![self[0].emit(function), self[1].emit(function)];
        let mut ifier = Typifier::new();
        let ty = super::extract::extract_type(
            &mut ifier,
            &function.module.module,
            &function.function,
            components[0],
        )
        .unwrap();
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Bi,
                kind,
                width: 4,
            },
        });
        function.emit(Expression::Compose { ty, components })
    }
}

impl<T: Emit> Emit for [T; 3] {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let components = vec![
            self[0].emit(function),
            self[1].emit(function),
            self[2].emit(function),
        ];
        let mut ifier = Typifier::new();
        let ty = super::extract::extract_type(
            &mut ifier,
            &function.module.module,
            &function.function,
            components[0],
        )
        .unwrap();
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                kind,
                width: 4,
            },
        });
        function.emit(Expression::Compose { ty, components })
    }
}

impl<T: Emit> Emit for [T; 4] {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let components = vec![
            self[0].emit(function),
            self[1].emit(function),
            self[2].emit(function),
            self[3].emit(function),
        ];
        let mut ifier = Typifier::new();
        let ty = super::extract::extract_type(
            &mut ifier,
            &function.module.module,
            &function.function,
            components[0],
        )
        .unwrap();
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Quad,
                kind,
                width: 4,
            },
        });
        function.emit(Expression::Compose { ty, components })
    }
}

pub struct Math(MathFunction, arrayvec::ArrayVec<Box<dyn Emit>, 4>);

impl Emit for Math {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let fun = &self.0;
        let args = &self.1;
        let expr = Expression::Math {
            fun: *fun,
            arg: args[0].emit(function),
            arg1: (args.len() > 1).then(|| args[1].emit(function)),
            arg2: (args.len() > 2).then(|| args[2].emit(function)),
            arg3: (args.len() > 3).then(|| args[3].emit(function)),
        };
        function.emit(expr)
    }
}

pub struct Cast(ScalarKind, Box<dyn Emit>);

impl Emit for Cast {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let expr = Expression::As {
            expr: self.1.emit(function),
            kind: self.0,
            convert: Some(4),
        };
        function.emit(expr)
    }
}

pub struct Binary(Box<dyn Emit>, BinaryOperator, Box<dyn Emit>);

impl Emit for Binary {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let expr = Expression::Binary {
            left: self.0.emit(function),
            op: self.1,
            right: self.2.emit(function),
        };
        function.emit(expr)
    }
}

pub struct Unary(UnaryOperator, Box<dyn Emit>);

impl Emit for Unary {
    fn emit(&self, function: &mut FunctionBuilder) -> Handle<Expression> {
        let expr = Expression::Unary {
            op: self.0,
            expr: self.1.emit(function),
        };
        function.emit(expr)
    }
}

pub fn add(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::Add, Box::new(right))
}

pub fn sub(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::Subtract, Box::new(right))
}

pub fn mul(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::Multiply, Box::new(right))
}

pub fn div(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::Divide, Box::new(right))
}

pub fn rem(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::Modulo, Box::new(right))
}

pub fn and(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::And, Box::new(right))
}

pub fn ior(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::InclusiveOr, Box::new(right))
}

pub fn eor(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::ExclusiveOr, Box::new(right))
}

pub fn shift_left(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::ShiftLeft, Box::new(right))
}

pub fn shift_right(left: impl Emit, right: impl Emit) -> Binary {
    Binary(Box::new(left), BinaryOperator::ShiftLeft, Box::new(right))
}

pub fn sint(expr: impl Emit) -> Cast {
    Cast(ScalarKind::Sint, Box::new(expr))
}

pub fn uint(expr: impl Emit) -> Cast {
    Cast(ScalarKind::Uint, Box::new(expr))
}

pub fn float(expr: impl Emit) -> Cast {
    Cast(ScalarKind::Float, Box::new(expr))
}

pub fn negate(expr: impl Emit) -> Unary {
    Unary(UnaryOperator::Negate, Box::new(expr))
}

pub fn not(expr: impl Emit) -> Unary {
    Unary(UnaryOperator::Not, Box::new(expr))
}
