#[cfg(not(target_arch = "wasm32"))]
use {
    arboard::Clipboard,
    bevy::log,
    std::cell::{RefCell, RefMut},
    thread_local::ThreadLocal,
};

/// A resource for accessing clipboard.
#[derive(Default)]
pub struct EguiClipboard {
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: ThreadLocal<Option<RefCell<Clipboard>>>,
    #[cfg(target_arch = "wasm32")]
    clipboard: String,
}

#[cfg(target_arch = "wasm32")]
impl EguiClipboard {
    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    #[allow(clippy::unnecessary_wraps)]
    #[must_use]
    pub fn contents(&self) -> Option<String> {
        Some(self.clipboard.clone())
    }

    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        self.clipboard = contents.to_owned();
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl EguiClipboard {
    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    #[must_use]
    pub fn contents(&self) -> Option<String> {
        if let Some(mut clipboard) = self.get() {
            match clipboard.get_text() {
                Ok(contents) => return Some(contents),
                Err(err) => log::info!("Failed to get clipboard contents: {:?}", err),
            }
        };
        None
    }

    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        if let Some(mut clipboard) = self.get() {
            if let Err(err) = clipboard.set_text(contents.to_owned()) {
                log::error!("Failed to set clipboard contents: {:?}", err);
            }
        }
    }

    fn get(&self) -> Option<RefMut<Clipboard>> {
        self.clipboard
            .get_or(|| {
                Clipboard::new()
                    .map(RefCell::new)
                    .map_err(|err| log::info!("Failed to initialize clipboard: {:?}", err))
                    .ok()
            })
            .as_ref()
            .map(|cell| cell.borrow_mut())
    }
}
