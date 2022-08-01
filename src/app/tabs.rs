use bevy::prelude::Entity;
use egui::Rect;

pub struct Tab {
    pub icon: char,
    pub title: String,
    pub entity: Entity,
}

impl ToString for Tab {
    fn to_string(&self) -> String {
        format!("{}  {}", self.icon, self.title)
    }
}

impl Tab {
    pub fn new(icon: char, title: impl Into<String>, entity: Entity) -> Self {
        Self {
            icon,
            title: title.into(),
            entity,
        }
    }
}

pub enum TreeNode {
    None,
    Leaf {
        rect: Rect,
        viewport: Rect,
        tabs: Vec<Tab>,
        active: usize,
    },
    Vertical {
        rect: Rect,
        fraction: f32,
    },
    Horizontal {
        rect: Rect,
        fraction: f32,
    },
}

impl TreeNode {
    pub fn leaf(tab: Tab) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs: vec![tab],
            active: 0,
        }
    }

    pub fn leaf_with(tabs: Vec<Tab>) -> Self {
        Self::Leaf {
            rect: Rect::NOTHING,
            viewport: Rect::NOTHING,
            tabs,
            active: 0,
        }
    }

    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::None => (),
            Self::Leaf { rect, .. }
            | Self::Vertical { rect, .. }
            | Self::Horizontal { rect, .. } => *rect = new_rect,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    pub fn split(&mut self, split: Split, fraction: f32) -> Self {
        let rect = Rect::NOTHING;
        let src = match split {
            Split::Left | Split::Right => TreeNode::Horizontal { fraction, rect },
            Split::Above | Split::Below => TreeNode::Vertical { fraction, rect },
        };
        std::mem::replace(self, src)
    }

    #[track_caller]
    pub fn append_tab(&mut self, tab: Tab) {
        match self {
            TreeNode::Leaf { tabs, .. } => tabs.push(tab),
            _ => unreachable!(),
        }
    }

    pub fn remove_tab(&mut self, index: usize) -> Option<Tab> {
        match self {
            TreeNode::Leaf { tabs, .. } => Some(tabs.remove(index)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
    pub const fn root() -> Self {
        Self(0)
    }

    pub const fn left(self) -> Self {
        Self(self.0 * 2 + 1)
    }

    pub const fn right(self) -> Self {
        Self(self.0 * 2 + 2)
    }

    pub const fn parent(self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
        }
    }

    pub const fn level(self) -> usize {
        (usize::BITS - (self.0 + 1).leading_zeros()) as usize
    }

    pub const fn is_left(self) -> bool {
        self.0 % 2 != 0
    }

    pub const fn is_right(self) -> bool {
        self.0 % 2 == 0
    }

    const fn children_at(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }

    const fn children_left(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base - 1;
        let e = (self.0 + 1) * base + base / 2 - 1;
        s..e
    }

    const fn children_right(self, level: usize) -> std::ops::Range<usize> {
        let base = 1 << level;
        let s = (self.0 + 1) * base + base / 2 - 1;
        let e = (self.0 + 2) * base - 1;
        s..e
    }
}

#[derive(Clone, Copy)]
pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

pub struct SplitTree {
    tree: Vec<TreeNode>,
}

impl std::ops::Index<NodeIndex> for SplitTree {
    type Output = TreeNode;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl std::ops::IndexMut<NodeIndex> for SplitTree {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.tree[index.0]
    }
}

impl SplitTree {
    pub fn new(root: TreeNode) -> Self {
        Self { tree: vec![root] }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TreeNode> {
        self.tree.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, TreeNode> {
        self.tree.iter_mut()
    }

    pub fn split_tabs(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        tabs: Vec<Tab>,
    ) -> [NodeIndex; 2] {
        self.split(parent, split, fraction, TreeNode::leaf_with(tabs))
    }

    pub fn split(
        &mut self,
        parent: NodeIndex,
        split: Split,
        fraction: f32,
        new: TreeNode,
    ) -> [NodeIndex; 2] {
        let old = self[parent].split(split, fraction);
        assert!(old.is_leaf());

        {
            let index = self.tree.iter().rposition(|n| !n.is_none()).unwrap_or(0);
            let level = NodeIndex(index).level();
            self.tree.resize_with(1 << (level + 1), || TreeNode::None);
        }

        let index = match split {
            Split::Right | Split::Above => [parent.right(), parent.left()],
            Split::Left | Split::Below => [parent.left(), parent.right()],
        };

        self[index[0]] = old;
        self[index[1]] = new;

        index
    }

    pub fn remove_empty_leaf(&mut self) {
        let mut nodes = self.tree.iter().enumerate();
        let node = nodes.find_map(|(index, node)| match node {
            TreeNode::Leaf { tabs, .. } if tabs.is_empty() => Some(index),
            _ => None,
        });

        let node = match node {
            Some(node) => NodeIndex(node),
            None => return,
        };

        let parent = node.parent().unwrap();

        self[parent] = TreeNode::None;
        self[node] = TreeNode::None;

        let mut level = 0;

        if node.is_left() {
            'left_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_right(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'left_end;
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], TreeNode::None);
                }
                level += 1;
            }
        } else {
            'right_end: loop {
                let dst = parent.children_at(level);
                let src = parent.children_left(level + 1);
                for (dst, src) in dst.zip(src) {
                    if src >= self.tree.len() {
                        break 'right_end;
                    }
                    self.tree[dst] = std::mem::replace(&mut self.tree[src], TreeNode::None);
                }
                level += 1;
            }
        }
    }
}
