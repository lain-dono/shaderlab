use crate::builder::{
    types::{F32_4, U32},
    FunctionBuilder, ModuleBuilder, NodeBuilder,
};
use crate::node::{Event, Node, NodeDescriptor, Output, PortId, Type};
use naga::{
    Binding, BuiltIn, EntryPoint, Expression, Handle, Interpolation, Sampling, ShaderStage,
    Statement,
};

#[derive(Debug, Default)]
pub struct Master {
    pub position: Option<Output>,
    pub color: Option<Output>,
}

impl NodeBuilder for Master {
    fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>> {
        None
    }
}

impl Master {
    pub fn entry<'nodes>(&self, nodes: &'nodes dyn NodeBuilder) -> Option<ModuleBuilder<'nodes>> {
        let mut module = ModuleBuilder::new(nodes);

        let vs_main = self.vertex_entry(nodes, &mut module)?;
        let fs_main = self.fragement_entry(nodes, &mut module)?;

        module.push_entry_point(vs_main);
        module.push_entry_point(fs_main);

        Some(module)
    }

    pub fn vertex_entry(
        &self,
        nodes: &dyn NodeBuilder,
        module: &mut ModuleBuilder,
    ) -> Option<EntryPoint> {
        let mut function = module.function();
        function.set_result(F32_4, Binding::BuiltIn(BuiltIn::Position));
        function.push_argument(
            String::from("vertex_index"),
            U32,
            Binding::BuiltIn(BuiltIn::VertexIndex),
        );

        let value = Some(nodes.expr(&mut function, self.position?)?);
        function.push_statement(Statement::Return { value });

        Some(EntryPoint {
            name: String::from("vs_main"),
            stage: ShaderStage::Vertex,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: function.build(),
        })
    }

    pub fn fragement_entry(
        &self,
        nodes: &dyn NodeBuilder,
        module: &mut ModuleBuilder,
    ) -> Option<EntryPoint> {
        let mut function = module.function();
        function.set_result(
            F32_4,
            Binding::Location {
                location: 0,
                interpolation: Some(Interpolation::Perspective),
                sampling: Some(Sampling::Center),
            },
        );

        let value = Some(nodes.expr(&mut function, self.color?)?);
        function.push_statement(Statement::Return { value });

        Some(EntryPoint {
            name: String::from("fs_main"),
            stage: ShaderStage::Fragment,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: function.build(),
        })
    }
}

impl Node for Master {
    fn desc(&self) -> NodeDescriptor {
        NodeDescriptor {
            label: "master",
            width: 80,
            inputs: &[("position", Type::V4F), ("color", Type::V4F)],
            outputs: &[],
        }
    }

    fn update(&mut self, event: Event) {
        match event {
            Event::Input(PortId(0), remote) => self.position = remote,
            Event::Input(PortId(1), remote) => self.color = remote,
            _ => (),
        }
    }
}
