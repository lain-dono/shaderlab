use super::*;
use bevy::{render::settings::WgpuSettings, winit::WinitPlugin, DefaultPlugins};

#[test]
fn headless_mode() {
    App::new()
        .insert_resource(WgpuSettings {
            backends: None,
            ..Default::default()
        })
        .add_plugins_with(DefaultPlugins, |group| group.disable::<WinitPlugin>())
        .add_plugin(EguiPlugin)
        .update();
}

#[test]
fn ctx_for_windows_mut_unique_check_passes() {
    let mut context = EguiContext::new();
    let primary_window = WindowId::primary();
    let second_window = WindowId::new();
    context.ctx.insert(primary_window, Default::default());
    context.ctx.insert(second_window, Default::default());
    let [primary_ctx, second_ctx] = context.ctx_mut([primary_window, second_window]);
    assert!(primary_ctx != second_ctx);
}

#[test]
#[should_panic(expected = "Window ids passed to `EguiContext::ctx_for_windows_mut` must be unique")]
fn ctx_for_windows_mut_unique_check_panics() {
    let mut context = EguiContext::new();
    let primary_window = WindowId::primary();
    context.ctx.insert(primary_window, Default::default());
    let _ = context.ctx_mut([primary_window, primary_window]);
}

#[test]
fn try_ctx_for_windows_mut_unique_check_passes() {
    let mut context = EguiContext::new();
    let primary_window = WindowId::primary();
    let second_window = WindowId::new();
    context.ctx.insert(primary_window, Default::default());
    context.ctx.insert(second_window, Default::default());
    let [primary_ctx, second_ctx] = context.try_ctx_mut([primary_window, second_window]);
    assert!(primary_ctx.is_some());
    assert!(second_ctx.is_some());
    assert!(primary_ctx != second_ctx);
}

#[test]
#[should_panic(expected = "Window ids passed to `EguiContext::ctx_for_windows_mut` must be unique")]
fn try_ctx_for_windows_mut_unique_check_panics() {
    let mut context = EguiContext::new();
    let primary_window = WindowId::primary();
    context.ctx.insert(primary_window, Default::default());
    let _ = context.try_ctx_mut([primary_window, primary_window]);
}
