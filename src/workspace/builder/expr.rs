use super::FnBuilder;
use naga::{
    BinaryOperator, Constant, ConstantInner, Expression, Handle, MathFunction, ScalarKind,
    ScalarValue, Type, TypeInner, UnaryOperator, VectorSize,
};

#[derive(Debug)]
pub enum EmitError {
    PortNotFound,
    MaybeDefault,
    FailType,
    Resolve(naga::proc::ResolveError),
    Validation(naga::WithSpan<naga::valid::ValidationError>),
    Wgsl(naga::back::wgsl::Error),
}

pub type EmitResult<T = Handle<Expression>> = Result<T, EmitError>;

impl From<naga::proc::ResolveError> for EmitError {
    fn from(err: naga::proc::ResolveError) -> Self {
        Self::Resolve(err)
    }
}

impl From<naga::WithSpan<naga::valid::ValidationError>> for EmitError {
    fn from(err: naga::WithSpan<naga::valid::ValidationError>) -> Self {
        Self::Validation(err)
    }
}

impl From<naga::back::wgsl::Error> for EmitError {
    fn from(err: naga::back::wgsl::Error) -> Self {
        Self::Wgsl(err)
    }
}

pub trait Emit: 'static {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult;

    fn sint(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Sint, Box::new(self))
    }

    fn uint(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Uint, Box::new(self))
    }

    fn float(self) -> Cast
    where
        Self: Sized,
    {
        Cast(ScalarKind::Float, Box::new(self))
    }

    fn negate(self) -> Unary
    where
        Self: Sized,
    {
        Unary(UnaryOperator::Negate, Box::new(self))
    }

    fn not(self) -> Unary
    where
        Self: Sized,
    {
        Unary(UnaryOperator::Not, Box::new(self))
    }
}

macro_rules! impl_emit {
    ($(
        $name:ident
        ($self:ident, $function:ident) $body:stmt
    )+) => {
        $(
            impl Emit for $name {
                fn emit(&$self, $function: &mut FnBuilder) -> EmitResult {
                    $body
                }
            }

            impl_emit!(@ $name Add add Add);
            impl_emit!(@ $name Sub sub Subtract);
            impl_emit!(@ $name Mul mul Multiply);
            impl_emit!(@ $name Div div Divide);
            impl_emit!(@ $name Rem rem Modulo);
            impl_emit!(@ $name BitAnd bitand And);
            impl_emit!(@ $name BitOr bitor InclusiveOr);
            impl_emit!(@ $name BitXor bitxor ExclusiveOr);
            impl_emit!(@ $name Shl shl ShiftLeft);
            impl_emit!(@ $name Shr shr ShiftRight);
        )+
    };

    (@ $for:ident $rust_op:ident $f:ident $naga_op:ident) => {
        impl<RHS: Emit> std::ops::$rust_op<RHS> for $for {
            type Output = Binary;
            fn $f(self, rhs: RHS) -> Self::Output {
                Binary(Box::new(self), BinaryOperator::$naga_op, Box::new(rhs))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct FunctionArgument(pub u32);

#[derive(Clone, Copy)]
pub struct AccessIndex(pub Handle<Expression>, pub u32);

pub struct Math(MathFunction, arrayvec::ArrayVec<Box<dyn Emit>, 4>);
pub struct Cast(ScalarKind, Box<dyn Emit>);
pub struct Binary(Box<dyn Emit>, BinaryOperator, Box<dyn Emit>);
pub struct Unary(UnaryOperator, Box<dyn Emit>);

#[derive(Clone)]
pub struct Expr(pub naga::Expression);

#[derive(Clone, Copy)]
pub struct Sint(pub i64);
#[derive(Clone, Copy)]
pub struct Uint(pub u64);
#[derive(Clone, Copy)]
pub struct Float(pub f64);
#[derive(Clone, Copy)]
pub struct Bool(pub bool);

#[derive(Clone, Copy)]
pub struct Wrap(pub Handle<Expression>);

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

impl Emit for Handle<Expression> {
    fn emit(&self, _: &mut FnBuilder) -> EmitResult {
        Ok(*self)
    }
}

impl Emit for naga::ScalarValue {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let value = *self;
        let width = if matches!(self, ScalarValue::Bool(_)) {
            naga::BOOL_WIDTH
        } else {
            4
        };
        let expr = Expression::Constant(function.module.constant(Constant {
            name: None,
            specialization: None,
            inner: ConstantInner::Scalar { width, value },
        }));
        Ok(function.expression(expr))
    }
}

impl_emit! {
    Wrap(self, _fn) Ok(self.0)
    Expr(self, function) Ok(function.emit(self.0.clone()))
    Sint(self, function) ScalarValue::Sint(self.0).emit(function)
    Uint(self, function) ScalarValue::Uint(self.0).emit(function)
    Float(self, function) ScalarValue::Float(self.0).emit(function)
    Bool(self, function) ScalarValue::Bool(self.0).emit(function)

    AccessIndex(self, function) Ok(function.expression(Expression::AccessIndex { base: self.0, index: self.1 }))
    FunctionArgument(self, function) Ok(function.expression(Expression::FunctionArgument(self.0)))

    Let(self, function) {
        let expr = self.expr.emit(function)?;
        function.insert_expression_name(expr, &self.name);
        Ok(expr)
    }
    Math(self, function) {
        let arg = &self.1;
        let expr = Expression::Math {
            fun: self.0,
            arg: arg[0].emit(function)?,
            arg1: (arg.len() > 1).then(|| arg[1].emit(function)).transpose()?,
            arg2: (arg.len() > 2).then(|| arg[2].emit(function)).transpose()?,
            arg3: (arg.len() > 3).then(|| arg[3].emit(function)).transpose()?,
        };
        Ok(function.emit(expr))
    }
    Cast(self, function) {
        let expr = Expression::As {
            expr: self.1.emit(function)?,
            kind: self.0,
            convert: Some(4),
        };
        Ok(function.emit(expr))
    }
    Binary(self, function) {
        let expr = Expression::Binary {
            left: self.0.emit(function)?,
            op: self.1,
            right: self.2.emit(function)?,
        };
        Ok(function.emit(expr))
    }
    Unary(self, function) {
        let expr = Expression::Unary {
            op: self.0,
            expr: self.1.emit(function)?,
        };
        Ok(function.emit(expr))
    }
}

impl<T: Emit> Emit for [T; 2] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![self[0].emit(function)?, self[1].emit(function)?];
        let ty = function.extract_type(components[0])?;
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Bi,
                kind,
                width: match kind {
                    ScalarKind::Bool => naga::BOOL_WIDTH,
                    _ => 4,
                },
            },
        });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}
impl<T: Emit> Emit for [T; 3] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![
            self[0].emit(function)?,
            self[1].emit(function)?,
            self[2].emit(function)?,
        ];
        let ty = function.extract_type(components[0])?;
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Tri,
                kind,
                width: match kind {
                    ScalarKind::Bool => naga::BOOL_WIDTH,
                    _ => 4,
                },
            },
        });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}
impl<T: Emit> Emit for [T; 4] {
    fn emit(&self, function: &mut FnBuilder) -> EmitResult {
        let components = vec![
            self[0].emit(function)?,
            self[1].emit(function)?,
            self[2].emit(function)?,
            self[3].emit(function)?,
        ];
        let ty = function.extract_type(components[0])?;
        let kind = ty.scalar_kind().unwrap();
        let ty = function.insert_type(Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Quad,
                kind,
                width: match kind {
                    ScalarKind::Bool => naga::BOOL_WIDTH,
                    _ => 4,
                },
            },
        });
        Ok(function.emit(Expression::Compose { ty, components }))
    }
}
