use super::{types, GenError, Message, Node, NodeId, Type};

#[derive(Clone, Default, Debug)]
pub struct MasterNode;

impl Node for MasterNode {
    fn label(&self) -> &str {
        "master"
    }

    fn width(&self) -> u16 {
        100
    }

    fn inputs(&self) -> &[(&str, Type)] {
        &[("color", super::types::VEC_F32_4)]
    }

    fn outputs(&self) -> &[(&str, Type)] {
        &[]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        if let ([Some(a)], []) = (inputs, outputs) {
            Ok(format!("return {};", a))
        } else {
            Err(GenError)
        }
    }
}
