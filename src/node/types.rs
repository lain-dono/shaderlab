use naga::{ScalarKind, TypeInner, VectorSize};

pub const F32: TypeInner = TypeInner::Scalar {
    kind: ScalarKind::Float,
    width: 4,
};

pub const VEC_F32_2: TypeInner = TypeInner::Vector {
    size: VectorSize::Bi,
    kind: ScalarKind::Float,
    width: 4,
};

pub const VEC_F32_3: TypeInner = TypeInner::Vector {
    size: VectorSize::Tri,
    kind: ScalarKind::Float,
    width: 4,
};

pub const VEC_F32_4: TypeInner = TypeInner::Vector {
    size: VectorSize::Quad,
    kind: ScalarKind::Float,
    width: 4,
};
