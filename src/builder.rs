use crate::node::{NodeId, Output};
use naga::{
    front::Typifier, proc::ResolveError, Binding, Constant, EntryPoint, Expression, Function,
    FunctionArgument, FunctionResult, GlobalVariable, Handle, LocalVariable, Module, Span,
    Statement, Type, TypeInner,
};

pub mod expr;
pub mod merge;
pub mod types;

pub trait NodeBuilder {
    fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>> {
        None
    }
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
        let flags = naga::back::wgsl::WriterFlags::empty();
        naga::back::wgsl::write_string(&self.module, &info, flags).unwrap()
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
            typifier: Typifier::default(),
        }
    }
}

pub struct FunctionBuilder<'a, 'nodes> {
    module: &'a mut ModuleBuilder<'nodes>,
    function: Function,
    typifier: Typifier,
}

impl<'a, 'nodes> FunctionBuilder<'a, 'nodes> {
    pub fn extract_type(&mut self, expr: Handle<Expression>) -> Result<&TypeInner, ResolveError> {
        self::types::extract_type(
            &mut self.typifier,
            &self.module.module,
            &self.function,
            expr,
        )
    }

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
