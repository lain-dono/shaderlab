use super::{Armature, Bone};

pub struct AddBone {
    pub bone: Bone,
}

impl undo::Action for AddBone {
    type Target = Armature;
    type Output = ();
    type Error = ();

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        //s.push(self.0);
        //target.add_bone(self.bone);
        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        //self.0 = s.pop().ok_or("s is empty")?;
        Ok(())
    }
}
