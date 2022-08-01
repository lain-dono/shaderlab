use crate::workspace::{Node, Port, Storage};
use ahash::AHashMap;
use naga::front::Typifier;
use naga::valid::{Capabilities, Validator};
use naga::{
    Binding, BuiltIn, Constant, EntryPoint, Expression, Function, FunctionArgument, FunctionResult,
    GlobalVariable, Handle, Interpolation, LocalVariable, Module, Sampling, Span, Statement,
    StructMember, Type, TypeInner,
};

pub use self::expr::{Emit, EmitError, EmitResult};
pub use self::types::*;

pub mod expr;
pub mod types;

pub trait OutputBuilder: downcast_rs::Downcast {
    fn expr(&self, function: &mut FnBuilder, output: Port) -> EmitResult;
}

downcast_rs::impl_downcast!(OutputBuilder);

pub struct ModuleBuilder<'storage> {
    pub storage: &'storage Storage,
    pub module: Module,
}

impl<'storage> ModuleBuilder<'storage> {
    pub const fn new(storage: &'storage Storage, module: Module) -> Self {
        Self { storage, module }
    }

    pub fn from_wgsl(
        storage: &'storage Storage,
        source: &str,
    ) -> Result<Self, naga::front::wgsl::ParseError> {
        Ok(Self::new(storage, naga::front::wgsl::parse_str(source)?))
    }

    /*
    pub fn merge_str(&mut self, source: &str) -> Result<(), naga::front::wgsl::ParseError> {
        let src = naga::front::wgsl::parse_str(source)?;
        self::merge::merge_module(&mut self.module, &src);
        Ok(())
    }
    */

    pub fn build(&self) -> EmitResult<String> {
        let mut validator = Validator::new(Default::default(), Capabilities::PRIMITIVE_INDEX);
        let info = validator.validate(&self.module);

        if let Err(err) = info.as_ref() {
            //println!("validate: {}\n{:#?}\n{:#?}", err, err, self.module);
            println!("validate: {}\n{:#?}", err, err);
        }

        let flags = naga::back::wgsl::WriterFlags::EXPLICIT_TYPES;
        Ok(naga::back::wgsl::write_string(&self.module, &info?, flags)?)
    }

    pub fn insert_type(&mut self, ty: Type) -> Handle<Type> {
        self.module.types.insert(ty, Span::default())
    }

    pub fn constant(&mut self, constant: Constant) -> Handle<Constant> {
        self.module.constants.append(constant, Span::default())
    }

    pub fn global_variable(&mut self, var: GlobalVariable) -> Handle<GlobalVariable> {
        self.module.global_variables.append(var, Span::default())
    }

    pub fn append_function(&mut self, function: Function) -> Handle<Function> {
        self.module.functions.append(function, Span::default())
    }

    pub fn push_entry_point(&mut self, entry: EntryPoint) {
        self.module.entry_points.push(entry)
    }

    pub fn entry(&mut self, f: impl FnOnce(&mut Self) -> EmitResult<EntryPoint>) -> EmitResult<()> {
        let entry = f(self)?;
        self.push_entry_point(entry);
        Ok(())
    }

    pub fn function(&mut self) -> FnBuilder<'_, 'storage> {
        FnBuilder {
            module: self,
            function: Function::default(),
            typifier: Typifier::default(),
            cache: AHashMap::default(),
        }
    }
}

pub struct FnBuilder<'a, 'storage> {
    pub module: &'a mut ModuleBuilder<'storage>,
    pub function: Function,
    pub typifier: Typifier,
    cache: AHashMap<Port, Handle<Expression>>,
}

impl<'a, 'storage> FnBuilder<'a, 'storage> {
    pub fn extract_type(&mut self, expr: Handle<Expression>) -> EmitResult<&TypeInner> {
        self.typifier.reset();
        Ok(self::types::extract_type(
            &mut self.typifier,
            &self.module.module,
            &self.function,
            expr,
        )?)
    }

    pub fn resolve_to(&mut self, expr: Handle<Expression>, dst: VectorKind) -> EmitResult {
        let ty = self.extract_type(expr)?;
        if let Some(src) = VectorKind::parse(ty) {
            self.resolve_vector(expr, src, dst)
        } else {
            Err(EmitError::FailType)
        }
    }

    pub fn insert_type(&mut self, ty: Type) -> Handle<Type> {
        self.module.insert_type(ty)
    }

    pub fn argument(
        &mut self,
        name: impl Into<String>,
        ty: Handle<Type>,
        binding: impl Into<Option<Binding>>,
    ) {
        self.function.arguments.push(FunctionArgument {
            name: Some(name.into()),
            ty,
            binding: binding.into(),
        });
    }

    pub fn set_result(&mut self, ty: Handle<Type>, binding: impl Into<Option<Binding>>) {
        let binding = binding.into();
        self.function.result = Some(FunctionResult { ty, binding });
    }

    pub fn local_variable(&mut self, var: LocalVariable) -> Handle<LocalVariable> {
        self.function.local_variables.append(var, Span::default())
    }

    pub fn expression(&mut self, expr: Expression) -> Handle<Expression> {
        self.function.expressions.append(expr, Span::default())
    }

    pub fn statement(&mut self, stmt: Statement) {
        self.function.body.push(stmt, Span::default())
    }

    pub fn insert_expression_name(&mut self, expr: Handle<Expression>, name: impl Into<String>) {
        self.function.named_expressions.insert(expr, name.into());
    }

    pub fn emit(&mut self, expr: Expression) -> Handle<Expression> {
        let old_len = self.function.expressions.len();
        let expr = self.expression(expr);
        let stmt = Statement::Emit(self.function.expressions.range_from(old_len));
        self.statement(stmt);
        expr
    }

    pub fn for_input_resolve(&mut self, input: Port, dst: VectorKind) -> EmitResult {
        let expr = self.for_input(input)?;
        self.resolve_to(expr, dst)
    }

    pub fn for_input_float(&mut self, input: Port) -> EmitResult {
        self.for_input_resolve(input, VectorKind::V1)
    }
    pub fn for_input_vector2(&mut self, input: Port) -> EmitResult {
        self.for_input_resolve(input, VectorKind::V2)
    }
    pub fn for_input_vector3(&mut self, input: Port) -> EmitResult {
        self.for_input_resolve(input, VectorKind::V3)
    }
    pub fn for_input_vector4(&mut self, input: Port) -> EmitResult {
        self.for_input_resolve(input, VectorKind::V4)
    }

    pub fn for_input(&mut self, input: Port) -> EmitResult {
        assert!(self.module.storage.ports[input].direction.is_input());

        let output = self
            .module
            .storage
            .links
            .values()
            .find_map(|link| link.output_for(input));

        let output = match output {
            Some(output) => output,
            None => {
                return self.module.storage.ports[input]
                    .input_default
                    .as_ref()
                    .ok_or(EmitError::MaybeDefault)
                    .and_then(|def| def.emit(self));
            }
        };

        Ok(if let Some(&expr) = self.cache.get(&output) {
            expr
        } else {
            let node = self.module.storage.ports[output].node;
            let expr = self.module.storage.nodes[node]
                .builder
                .output_expr(node, self, output)?;

            self.cache.insert(output, expr);
            expr
        })
    }

    pub fn function_by_name(&self, name: &str) -> Option<Handle<Function>> {
        self.module
            .module
            .functions
            .iter()
            .find_map(|(handle, f)| (f.name.as_deref() == Some(name)).then(|| handle))
    }

    pub fn access_index(&mut self, base: Handle<Expression>, index: u32) -> Handle<Expression> {
        self.emit(Expression::AccessIndex { base, index })
    }

    pub fn call(
        &mut self,
        name: &str,
        arguments: impl Into<Vec<Handle<Expression>>>,
    ) -> EmitResult {
        let function = self.function_by_name(name).unwrap();
        let result = self.expression(Expression::CallResult(function));
        self.statement(Statement::Call {
            function,
            arguments: arguments.into(),
            result: Some(result),
        });
        Ok(result)
    }

    pub fn named_expr(&mut self, node: Node, expr: Handle<Expression>) {
        let prefix = self.module.storage.nodes[node]
            .title
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        let key = <Node as slotmap::Key>::data(&node);
        let name = format!("_e{}_{}{:?}", expr.index(), prefix, key);
        self.function.named_expressions.insert(expr, name);
    }
}

pub struct StructBuilder<'a> {
    pub name: String,
    pub module: &'a mut Module,
    pub members: Vec<StructMember>,
    pub span: u32,
}

impl<'a> StructBuilder<'a> {
    pub fn new(module: &'a mut Module, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            module,
            members: Vec::new(),
            span: 0,
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.members.iter().any(|m| m.name.as_deref() == Some(name))
    }

    pub fn builtin(self, name: impl Into<String>, ty: Handle<Type>, builtin: BuiltIn) -> Self {
        self.append(name, ty, Some(Binding::BuiltIn(builtin)))
    }

    pub fn interpolator(self, name: impl Into<String>, ty: Handle<Type>, loc: u32) -> Self {
        let binding = Binding::Location {
            location: loc,
            interpolation: Some(Interpolation::Perspective),
            sampling: Some(Sampling::Center),
        };
        self.append(name, ty, binding)
    }

    pub fn append(
        mut self,
        name: impl Into<String>,
        ty: Handle<Type>,
        binding: impl Into<Option<Binding>>,
    ) -> Self {
        let name = name.into();
        assert!(!self.has(&name));

        let ty_inner = &self.module.types[ty].inner;
        let span = ty_inner.size(&self.module.constants);

        self.members.push(StructMember {
            name: Some(name),
            ty,
            binding: binding.into(),
            offset: self.span,
        });

        let align_mask = 0b111;
        self.span += (span + align_mask) & !align_mask;
        self
    }

    pub fn build(self) -> Handle<Type> {
        let Self {
            module,
            name,
            members,
            span,
        } = self;

        let value = Type {
            name: Some(name),
            inner: TypeInner::Struct { members, span },
        };
        module.types.insert(value, Span::default())
    }
}
