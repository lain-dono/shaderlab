use super::{Armature, Transform};

pub struct TransformBone {
    pub bone: usize,
    pub transform: Transform,
}

impl undo::Action for TransformBone {
    type Target = Armature;
    type Output = ();
    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let bone = &mut target.bones[self.bone];
        bone.transform = bone.transform.mul_transform(self.transform);
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let bone = &mut target.bones[self.bone];
        bone.transform = bone.transform.mul_transform(self.transform.inverse());
        Ok(())
    }
}
