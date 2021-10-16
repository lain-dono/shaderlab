use naga::{
    BinaryOperator, Binding, Constant, ConstantInner, EntryPoint, Expression, Function,
    FunctionArgument, FunctionResult, GlobalVariable, Handle, LocalVariable, Module, ScalarKind,
    ScalarValue, Span, Statement, Type, TypeInner, VectorSize,
};

pub mod expr;
pub mod extract;
pub mod merge;
pub mod node;

pub const U32: Type = Type {
    name: None,
    inner: TypeInner::Scalar {
        kind: ScalarKind::Uint,
        width: 4,
    },
};

pub const I32: Type = Type {
    name: None,
    inner: TypeInner::Scalar {
        kind: ScalarKind::Sint,
        width: 4,
    },
};

pub const F32: Type = Type {
    name: None,
    inner: TypeInner::Scalar {
        kind: ScalarKind::Float,
        width: 4,
    },
};

pub const F32_2: Type = Type {
    name: None,
    inner: TypeInner::Vector {
        size: VectorSize::Bi,
        kind: ScalarKind::Float,
        width: 4,
    },
};

pub const F32_3: Type = Type {
    name: None,
    inner: TypeInner::Vector {
        size: VectorSize::Tri,
        kind: ScalarKind::Float,
        width: 4,
    },
};

pub const F32_4: Type = Type {
    name: None,
    inner: TypeInner::Vector {
        size: VectorSize::Quad,
        kind: ScalarKind::Float,
        width: 4,
    },
};

#[derive(Default)]
pub struct ModuleBuilder {
    module: Module,
}

impl ModuleBuilder {
    pub fn from_wgsl(source: &str) -> Result<Self, naga::front::wgsl::ParseError> {
        let module = naga::front::wgsl::parse_str(source)?;
        Ok(Self { module })
    }

    pub fn build(&self) -> String {
        use naga::valid::{Capabilities, Validator};
        let mut validator = Validator::new(Default::default(), Capabilities::PRIMITIVE_INDEX);
        let info = validator.validate(&self.module).unwrap();
        naga::back::wgsl::write_string(&self.module, &info).unwrap()
    }

    pub fn insert_type(&mut self, ty: Type) -> Handle<Type> {
        self.module.types.insert(ty, Span::default())
    }

    pub fn append_constant(&mut self, constant: Constant) -> Handle<Constant> {
        self.module.constants.append(constant, Span::default())
    }

    pub fn append_global_variable(&mut self, var: GlobalVariable) -> Handle<GlobalVariable> {
        self.module.global_variables.append(var, Span::default())
    }

    pub fn append_function(&mut self, function: Function) -> Handle<Function> {
        self.module.functions.append(function, Span::default())
    }

    pub fn push_entry_point(&mut self, entry: EntryPoint) {
        self.module.entry_points.push(entry)
    }

    pub fn function(&mut self) -> FunctionBuilder {
        FunctionBuilder {
            module: self,
            function: Function::default(),
        }
    }

    pub fn merge_str(&mut self, source: &str) -> Result<(), naga::front::wgsl::ParseError> {
        let src = naga::front::wgsl::parse_str(source)?;
        self::merge::merge_module(&mut self.module, &src);
        Ok(())
    }
}

pub struct FunctionBuilder<'a> {
    module: &'a mut ModuleBuilder,
    function: Function,
}

impl<'a> FunctionBuilder<'a> {
    pub fn build(self) -> Function {
        self.function
    }

    pub fn module(&mut self) -> &mut ModuleBuilder {
        self.module
    }

    pub fn inner(&mut self) -> &mut Function {
        &mut self.function
    }

    pub fn insert_type(&mut self, ty: Type) -> Handle<Type> {
        self.module.insert_type(ty)
    }

    pub fn push_argument(
        &mut self,
        name: impl Into<Option<String>>,
        ty: Type,
        binding: impl Into<Option<Binding>>,
    ) -> usize {
        let ty = self.insert_type(ty);
        let index = self.function.arguments.len();
        self.function.arguments.push(FunctionArgument {
            name: name.into(),
            ty,
            binding: binding.into(),
        });
        index
    }

    pub fn set_result(&mut self, ty: Type, binding: impl Into<Option<Binding>>) {
        self.function.result = Some(FunctionResult {
            ty: self.insert_type(ty),
            binding: binding.into(),
        });
    }

    pub fn append_local_variable(&mut self, var: LocalVariable) -> Handle<LocalVariable> {
        self.function.local_variables.append(var, Span::default())
    }

    pub fn append_expression(&mut self, expr: Expression) -> Handle<Expression> {
        self.function.expressions.append(expr, Span::default())
    }

    pub fn push_statement(&mut self, stmt: Statement) {
        self.function.body.push(stmt, Span::default())
    }

    pub fn insert_expression_name(&mut self, expr: Handle<Expression>, name: impl Into<String>) {
        self.function.named_expressions.insert(expr, name.into());
    }
}

impl<'a> FunctionBuilder<'a> {
    pub fn emit(&mut self, expr: Expression) -> Handle<Expression> {
        let old_len = self.function.expressions.len();
        let expr = self.append_expression(expr);
        let end = Statement::Emit(self.function.expressions.range_from(old_len));
        self.function.body.push(end, Span::default());
        expr
    }

    pub fn argument(&mut self, index: u32) -> Handle<Expression> {
        self.append_expression(Expression::FunctionArgument(index))
    }

    pub fn uint(&mut self, value: u64) -> Handle<Expression> {
        self.scalar(ScalarValue::Uint(value))
    }

    pub fn sint(&mut self, value: i64) -> Handle<Expression> {
        self.scalar(ScalarValue::Sint(value))
    }

    pub fn float(&mut self, value: f64) -> Handle<Expression> {
        self.scalar(ScalarValue::Float(value))
    }

    fn scalar(&mut self, value: ScalarValue) -> Handle<Expression> {
        let expr = Expression::Constant(self.module.append_constant(Constant {
            name: None,
            specialization: None,
            inner: ConstantInner::Scalar { width: 4, value },
        }));
        self.append_expression(expr)
    }

    pub fn as_float(&mut self, expr: Handle<Expression>) -> Handle<Expression> {
        self.cast(expr, ScalarKind::Float)
    }

    pub fn as_sint(&mut self, expr: Handle<Expression>) -> Handle<Expression> {
        self.cast(expr, ScalarKind::Sint)
    }

    pub fn as_uint(&mut self, expr: Handle<Expression>) -> Handle<Expression> {
        self.cast(expr, ScalarKind::Uint)
    }

    fn cast(&mut self, expr: Handle<Expression>, kind: ScalarKind) -> Handle<Expression> {
        let convert = Some(4);
        self.emit(Expression::As {
            expr,
            kind,
            convert,
        })
    }

    pub fn bin(
        &mut self,
        left: Handle<Expression>,
        op: BinaryOperator,
        right: Handle<Expression>,
    ) -> Handle<Expression> {
        self.emit(Expression::Binary { op, left, right })
    }
}

pub fn example_naga() {
    let mut nodes = node::NodeArena::default();
    let triangle = nodes.push(node::input::Triangle);
    let color = nodes.push(node::input::Vec4::new([0.1, 0.2, 0.3, 0.4]));

    let _master = nodes.push(node::master::Master {
        position: Some((triangle, 0)),
        color: Some((color, 0)),
    });

    for (_, node) in nodes.inner.iter() {
        let master: Option<&node::master::Master> = node.downcast_ref();
        if let Some(master) = master {
            let mut builder = ModuleBuilder::default();
            master.entry(&nodes, &mut builder);

            println!("{:#?}", builder.module);
            let source = builder.build();
            println!("{}", source);
        }
    }

    {
        let source = r#"
        fn lol() {
            let a = vec4<f32>(1.0);
            //let b = a.ww;
            let c = a.w;
        }
        "#;
        let module = ModuleBuilder::from_wgsl(source).unwrap();
        println!("{:#?}", module.module);
        println!("{}", module.build());
    }
}
