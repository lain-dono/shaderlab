use super::super::Transform;

pub struct BoneData {
    pub name: String,
    pub parent: String,
    pub transform: Transform,
    pub length: f32,
    pub color: u32,
}

pub struct SlotData {
    pub name: String,
    pub bone: String,
    pub attachment: String,

    pub color: u32,
    // TODO: blend + blend_color
}

pub struct SkinData {
    pub name: String,
}

pub struct ClipData {
    pub name: String,
}

pub struct ArmatureData {
    pub bones: Vec<BoneData>, // parent first order
    pub slots: Vec<SlotData>, // draw order
    pub skins: Vec<SkinData>,
    pub clips: Vec<ClipData>,
}

macro_rules! _finder {
    ($ref_fn:ident, $mut_fn:ident => $field:ident : $ty:ident) => {
        pub fn $ref_fn(&self, name: &str) -> Option<&$ty> {
            self.$field.iter().find(|item| item.name == name)
        }

        pub fn $mut_fn(&mut self, name: &str) -> Option<&mut $ty> {
            self.$field.iter_mut().find(|item| item.name == name)
        }
    };
}

impl ArmatureData {
    _finder!(bone, bone_mut => bones: BoneData);
    _finder!(slot, slot_mut => slots: SlotData);
    _finder!(skin, skin_mut => skins: SkinData);
    _finder!(clip, clip_mut => clips: ClipData);

    pub fn add_bone(&mut self, bone: BoneData) {
        let index = self
            .bones
            .iter()
            .position(|item| item.name == bone.parent)
            .map(|i| i + 1)
            .filter(|&i| i < self.bones.len());

        if let Some(index) = index {
            self.bones.insert(index, bone)
        } else {
            self.bones.push(bone)
        }
    }
}

pub struct Slot {
    pub name: String,
    pub bone: u32,
}

pub struct Bone {
    pub name: String,
    pub transform: Transform,
    pub color: u32,
    pub length: f32,
    pub parent: u32,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            name: String::new(),
            transform: Transform::default(),
            color: 0xFFFFFFFF,
            length: 0.0,
            parent: u32::max_value(),
        }
    }
}

pub struct Armature {
    pub bones: Vec<Bone>,
    pub slots: Vec<Slot>,
}

impl Armature {
    pub fn insert_bone(&mut self, index: usize, bone: Bone) {
        for bone in &mut self.bones {
            if bone.parent >= index as u32 {
                bone.parent = bone.parent.saturating_add(1);
            }
        }
        self.bones.insert(index, bone);
    }

    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
    }
}
