pub mod filebrowser;
pub mod hierarchy;
pub mod inspector;
pub mod placeholder;
pub mod timeline;

pub use self::{
    filebrowser::FileBrowser, hierarchy::Hierarchy, inspector::Inspector,
    placeholder::PlaceholderTab, timeline::Timeline,
};
