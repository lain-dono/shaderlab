use crate::builder::{types::F32_4, types::U32, FunctionBuilder, ModuleBuilder, NodeBuilder};
use crate::node::{port, Event, Node, NodeElement, NodeId, NodeWidget, Output, Type};
use naga::{
    Binding, BuiltIn, EntryPoint, Expression, Handle, Interpolation, Sampling, ShaderStage,
    Statement,
};

pub struct Master(NodeWidget);

impl Default for Master {
    fn default() -> Self {
        Self(NodeWidget::new(
            "master",
            80,
            &[("position", Type::Vector3), ("color", Type::Vector4)],
            &[],
        ))
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

        let position = self.0.inputs_remote[0];

        let value = Some(nodes.expr(&mut function, position?)?);
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

        let color = self.0.inputs_remote[1];

        let value = Some(nodes.expr(&mut function, color?)?);
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
    fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>> {
        None
    }

    fn inputs(&self) -> &[port::State] {
        &self.0.ports.inputs
    }
    fn outputs(&self) -> &[port::State] {
        &self.0.ports.outputs
    }

    fn update(&mut self, event: Event) {
        self.0.update(event)
    }

    fn view(&mut self, node: NodeId) -> NodeElement {
        self.0.view(node, None)
    }
}
