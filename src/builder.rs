use crate::controls::edge::{Output, PortId};
use crate::node::NodeId;
use naga::{
    Binding, Constant, EntryPoint, Expression, Function, FunctionArgument, FunctionResult,
    GlobalVariable, Handle, LocalVariable, Module, ScalarKind, Span, Statement, Type, TypeInner,
    VectorSize,
};

pub mod expr;
pub mod extract;
pub mod merge;

pub trait NodeBuilder {
    fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>>;
}

pub trait AnyNode: downcast_rs::Downcast + NodeBuilder {}
downcast_rs::impl_downcast!(AnyNode);

impl<T: downcast_rs::Downcast + NodeBuilder> AnyNode for T {}

pub type Node = Box<dyn AnyNode>;

impl NodeBuilder for slotmap::SlotMap<NodeId, Node> {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        self[output.node].expr(function, output)
    }
}

impl NodeBuilder for slotmap::SlotMap<NodeId, crate::controls::NodeWidget> {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        self[output.node].node.expr(function, output)
    }
}

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

pub struct ModuleBuilder<'nodes> {
    nodes: &'nodes dyn NodeBuilder,
    module: Module,
}

impl<'nodes> ModuleBuilder<'nodes> {
    pub fn new(nodes: &'nodes dyn NodeBuilder) -> Self {
        let module = Module::default();
        Self { nodes, module }
    }

    pub fn from_wgsl(
        nodes: &'nodes dyn NodeBuilder,
        source: &str,
    ) -> Result<Self, naga::front::wgsl::ParseError> {
        let module = naga::front::wgsl::parse_str(source)?;
        Ok(Self { nodes, module })
    }

    pub fn merge_str(&mut self, source: &str) -> Result<(), naga::front::wgsl::ParseError> {
        let src = naga::front::wgsl::parse_str(source)?;
        self::merge::merge_module(&mut self.module, &src);
        Ok(())
    }

    pub fn module(&self) -> &Module {
        &self.module
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

    pub fn function(&mut self) -> FunctionBuilder<'_, 'nodes> {
        FunctionBuilder {
            module: self,
            function: Function::default(),
        }
    }
}

pub struct FunctionBuilder<'a, 'nodes> {
    module: &'a mut ModuleBuilder<'nodes>,
    function: Function,
}

impl<'a, 'nodes> FunctionBuilder<'a, 'nodes> {
    pub fn node(&mut self, port: Output) -> Option<Handle<Expression>> {
        self.module.nodes.expr(self, port)
    }

    pub fn build(self) -> Function {
        self.function
    }

    pub fn module(&mut self) -> &mut ModuleBuilder<'nodes> {
        self.module
    }

    pub fn function(&mut self) -> &mut Function {
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

    pub fn emit(&mut self, expr: Expression) -> Handle<Expression> {
        let old_len = self.function.expressions.len();
        let expr = self.append_expression(expr);
        let end = Statement::Emit(self.function.expressions.range_from(old_len));
        self.function.body.push(end, Span::default());
        expr
    }
}

pub fn example_naga() {
    let mut nodes: slotmap::SlotMap<crate::node::NodeId, Node> = slotmap::SlotMap::default();

    let triangle = nodes.insert(Box::new(crate::node::input::Triangle));
    let color = nodes.insert(Box::new(crate::node::input::Vec4 {
        value: [0.1, 0.2, 0.3, 0.4],
    }));

    let _master = nodes.insert(Box::new(crate::node::master::Master {
        position: Some(Output {
            node: triangle,
            port: PortId(0),
        }),
        color: Some(Output {
            node: color,
            port: PortId(0),
        }),
    }));

    for (_, node) in nodes.iter() {
        let master: Option<&crate::node::master::Master> = node.downcast_ref();
        if let Some(master) = master {
            if let Some(builder) = master.entry(&nodes) {
                println!("{:#?}", builder.module);
                println!("{}", builder.build());
            }
        }
    }

    if false {
        let source = r#"
        fn lol() {
            let a = vec4<f32>(1.0);
            //let b = a.ww;
            let c = a.w;
        }
        "#;
        let module = ModuleBuilder::from_wgsl(&nodes, source).unwrap();
        println!("{:#?}", module.module);
        println!("{}", module.build());
    }
}
