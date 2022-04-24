use crate::app::{Global, Style};
use egui::{Rect, Ui};

pub struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub window: &'a winit::window::Window,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub attachment: wgpu::RenderPassColorAttachment<'a>,
    pub viewport: Rect,
}

pub trait TabInner {
    fn ui(&mut self, ui: &mut Ui, style: &Style, global: &mut Global);

    fn render(&mut self, _ctx: RenderContext) {}

    fn on_event(
        &mut self,
        _event: &winit::event::WindowEvent<'static>,
        _viewport: Rect,
        _parent_scale: f32,
    ) {
    }
}

pub struct Tab {
    pub icon: char,
    pub title: String,
    pub inner: Box<dyn TabInner>,
}

impl ToString for Tab {
    fn to_string(&self) -> String {
        format!("{}  {}", self.icon, self.title)
    }
}

impl Tab {
    pub fn new(icon: char, title: impl Into<String>, inner: impl TabInner + 'static) -> Self {
        Self {
            icon,
            title: title.into(),
            inner: Box::new(inner),
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

    pub fn replace(&mut self, value: Self) -> Self {
        std::mem::replace(self, value)
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }

    pub fn tabs_mut(&mut self) -> Option<&mut Vec<Tab>> {
        match self {
            TreeNode::Leaf { tabs, .. } => Some(tabs),
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

pub enum Split {
    Left,
    Right,
    Above,
    Below,
}

pub struct Tree {
    tree: Vec<TreeNode>,
}

impl std::ops::Index<NodeIndex> for Tree {
    type Output = TreeNode;

    #[inline(always)]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.tree[index.0]
    }
}

impl std::ops::IndexMut<NodeIndex> for Tree {
    #[inline(always)]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.tree[index.0]
    }
}

impl Tree {
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

    fn fix_len_parent(&mut self, parent: NodeIndex) {
        let new_len = 1 << (parent.level() + 1);
        self.tree.resize_with(new_len + 1, || TreeNode::None);
    }

    pub fn split(&mut self, parent: NodeIndex, new: TreeNode, split: Split) -> [NodeIndex; 2] {
        let fraction = 0.5;
        let rect = Rect::NOTHING;
        let old = self[parent].replace(match split {
            Split::Left | Split::Right => TreeNode::Horizontal { fraction, rect },
            Split::Above | Split::Below => TreeNode::Vertical { fraction, rect },
        });

        assert!(matches!(old, TreeNode::Leaf { .. }));
        self.fix_len_parent(parent);

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
                    self.tree[dst] = self.tree[src].take();
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
                    self.tree[dst] = self.tree[src].take();
                }
                level += 1;
            }
        }
    }
}
