pub mod renderer;
pub mod style;
pub mod widget;

pub use self::style::{Handle, HandleShape, Style, StyleSheet};
pub use self::widget::State;

pub type Slider<'a, T, M> = self::renderer::Slider<'a, T, M, iced_wgpu::Backend>;
