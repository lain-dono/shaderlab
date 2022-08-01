use super::{
    convert, EguiClipboard, EguiContext, EguiOutput, EguiRawInput, EguiRenderOutput, EguiSettings,
    WindowSize,
};
use bevy::{
    ecs::{
        event::{EventReader, EventWriter},
        system::{Local, Res, ResMut, SystemParam},
    },
    input::{
        keyboard::{KeyCode, KeyboardInput},
        mouse::{MouseButton, MouseButtonInput, MouseScrollUnit, MouseWheel},
        ButtonState, Input,
    },
    prelude::Time,
    window::{
        CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, RequestRedraw, WindowCreated,
        WindowFocused, WindowId, Windows,
    },
    {log, utils::HashMap},
};

#[derive(SystemParam)]
pub struct InputEvents<'w, 's> {
    cursor_entered: EventReader<'w, 's, CursorEntered>,
    cursor_left: EventReader<'w, 's, CursorLeft>,
    cursor: EventReader<'w, 's, CursorMoved>,
    mouse_button_input: EventReader<'w, 's, MouseButtonInput>,
    mouse_wheel: EventReader<'w, 's, MouseWheel>,
    received_character: EventReader<'w, 's, ReceivedCharacter>,
    keyboard_input: EventReader<'w, 's, KeyboardInput>,
    window_focused: EventReader<'w, 's, WindowFocused>,
    window_created: EventReader<'w, 's, WindowCreated>,
}

#[derive(SystemParam)]
pub struct InputResources<'w, 's> {
    context: ResMut<'w, EguiContext>,
    raw_input: ResMut<'w, HashMap<WindowId, EguiRawInput>>,
    window_sizes: ResMut<'w, HashMap<WindowId, WindowSize>>,
    focused_window: Local<'s, Option<WindowId>>,
    windows: ResMut<'w, Windows>,
    settings: Res<'w, EguiSettings>,
}

impl<'w, 's> InputResources<'w, 's> {
    fn update(&mut self) {
        for window in self.windows.iter() {
            let input = self.raw_input.entry(window.id()).or_default();

            let size = WindowSize::new(
                window.physical_width(),
                window.physical_height(),
                window.scale_factor() as f32,
            );

            let width = size.scaled_width() / self.settings.scale_factor as f32;
            let height = size.scaled_height() / self.settings.scale_factor as f32;

            if width < 1.0 || height < 1.0 {
                continue;
            }

            input.raw_input.screen_rect = Some(egui::Rect::from_min_max(
                egui::pos2(0.0, 0.0),
                egui::pos2(width, height),
            ));

            input.raw_input.pixels_per_point =
                Some(size.scale_factor * self.settings.scale_factor as f32);

            self.window_sizes.insert(window.id(), size);
            self.context.ctx.entry(window.id()).or_default();
        }
    }

    fn focused_input(&mut self) -> Option<&mut EguiRawInput> {
        self.focused_window
            .as_ref()
            .and_then(|window_id| self.raw_input.get_mut(window_id))
    }
}

pub fn init_contexts_on_startup(mut input: InputResources) {
    input.update();
}

pub fn process_input(
    mut events: InputEvents,
    mut input: InputResources,
    clipboard: Res<EguiClipboard>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    // This is a workaround for Windows.
    // For some reason, `WindowFocused` event isn't fired when a window is created.
    if let Some(event) = events.window_created.iter().next_back() {
        *input.focused_window = Some(event.id);
    }

    for event in events.window_focused.iter() {
        *input.focused_window = if event.focused { Some(event.id) } else { None };
    }

    input.update();

    let shift = keyboard.pressed(KeyCode::LShift) || keyboard.pressed(KeyCode::RShift);
    let ctrl = keyboard.pressed(KeyCode::LControl) || keyboard.pressed(KeyCode::RControl);
    let alt = keyboard.pressed(KeyCode::LAlt) || keyboard.pressed(KeyCode::RAlt);
    let win = keyboard.pressed(KeyCode::LWin) || keyboard.pressed(KeyCode::RWin);

    let mac_cmd = if cfg!(target_os = "macos") {
        win
    } else {
        false
    };
    let command = if cfg!(target_os = "macos") { win } else { ctrl };

    let modifiers = egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd,
        command,
    };

    let mut cursor_left_window = None;
    if let Some(cursor_left) = events.cursor_left.iter().next_back() {
        input
            .raw_input
            .get_mut(&cursor_left.id)
            .unwrap()
            .raw_input
            .events
            .push(egui::Event::PointerGone);
        cursor_left_window = Some(cursor_left.id);
    }
    let cursor_entered_window = events
        .cursor_entered
        .iter()
        .next_back()
        .map(|event| event.id);

    // When a user releases a mouse button, Safari emits both `CursorLeft` and `CursorEntered`
    // events during the same frame. We don't want to reset mouse position in such a case, otherwise
    // we won't be able to process the mouse button event.
    if cursor_left_window.is_some() && cursor_left_window != cursor_entered_window {
        input.context.mouse_position = None;
    }

    if let Some(cursor_moved) = events.cursor.iter().next_back() {
        // If we've left the window,
        // it's unlikely that we've moved the cursor back to the same window this exact frame.
        if cursor_left_window != Some(cursor_moved.id) {
            let scale_factor = input.settings.scale_factor as f32;
            let mut mouse_position: (f32, f32) = (cursor_moved.position / scale_factor).into();
            mouse_position.1 = input.window_sizes[&cursor_moved.id].scaled_height() / scale_factor
                - mouse_position.1;
            input.context.mouse_position = Some((cursor_moved.id, mouse_position.into()));
            input
                .raw_input
                .get_mut(&cursor_moved.id)
                .unwrap()
                .raw_input
                .events
                .push(egui::Event::PointerMoved(egui::Pos2::from(mouse_position)));
        }
    }

    // Marks the events as read if we are going to ignore them (i.e. there's no window hovered).
    let mouse_button_event_iter = events.mouse_button_input.iter();
    let mouse_wheel_event_iter = events.mouse_wheel.iter();
    if let Some((window_id, position)) = input.context.mouse_position.as_ref() {
        if let Some(input) = input.raw_input.get_mut(window_id) {
            let events = &mut input.raw_input.events;

            for mouse_button_event in mouse_button_event_iter {
                let button = match mouse_button_event.button {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    _ => None,
                };
                let pressed = matches!(mouse_button_event.state, ButtonState::Pressed);
                if let Some(button) = button {
                    events.push(egui::Event::PointerButton {
                        pos: position.to_pos2(),
                        button,
                        pressed,
                        modifiers,
                    });
                }
            }

            for event in mouse_wheel_event_iter {
                let mut delta = egui::vec2(event.x, event.y);
                if let MouseScrollUnit::Line = event.unit {
                    // https://github.com/emilk/egui/blob/a689b623a669d54ea85708a8c748eb07e23754b0/egui-winit/src/lib.rs#L449
                    delta *= 50.0;
                }

                // Winit has inverted hscroll.
                // TODO: remove this line when Bevy updates winit after https://github.com/rust-windowing/winit/pull/2105 is merged and released.
                delta.x *= -1.0;

                if ctrl || mac_cmd {
                    // Treat as zoom instead.
                    let factor = (delta.y / 200.0).exp();
                    events.push(egui::Event::Zoom(factor));
                } else if shift {
                    // Treat as horizontal scrolling.
                    // Note: Mac already fires horizontal scroll events when shift is down.
                    events.push(egui::Event::Scroll(egui::vec2(delta.x + delta.y, 0.0)));
                } else {
                    events.push(egui::Event::Scroll(delta));
                }
            }
        }
    }

    if !ctrl && !win {
        for event in events.received_character.iter() {
            if !event.char.is_control() {
                input
                    .raw_input
                    .get_mut(&event.id)
                    .unwrap()
                    .raw_input
                    .events
                    .push(egui::Event::Text(event.char.to_string()));
            }
        }
    }

    if let Some(focused_input) = input.focused_input() {
        for ev in events.keyboard_input.iter() {
            if let Some(key) = ev.key_code.and_then(convert::key) {
                let pressed = match ev.state {
                    ButtonState::Pressed => true,
                    ButtonState::Released => false,
                };
                focused_input.raw_input.events.push(egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                });

                // We also check that it's an `ElementState::Pressed` event,
                // as we don't want to copy, cut or paste on the key release.
                if command && pressed {
                    match key {
                        egui::Key::C => focused_input.raw_input.events.push(egui::Event::Copy),
                        egui::Key::X => focused_input.raw_input.events.push(egui::Event::Cut),
                        egui::Key::V => {
                            if let Some(contents) = clipboard.contents() {
                                focused_input
                                    .raw_input
                                    .events
                                    .push(egui::Event::Text(contents))
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        focused_input.raw_input.modifiers = modifiers;
    }

    for input in input.raw_input.values_mut() {
        input.raw_input.predicted_dt = time.delta_seconds();
    }
}

pub fn begin_frame(
    mut context: ResMut<EguiContext>,
    mut input: ResMut<HashMap<WindowId, EguiRawInput>>,
) {
    for (id, ctx) in context.ctx.iter_mut() {
        let new_input = input.get_mut(id).unwrap().raw_input.take();
        ctx.begin_frame(new_input);
    }
}

pub fn process_output(
    mut windows: Option<ResMut<Windows>>,
    mut context: ResMut<EguiContext>,
    mut output: ResMut<HashMap<WindowId, EguiOutput>>,
    mut render_output: ResMut<HashMap<WindowId, EguiRenderOutput>>,
    mut clipboard: ResMut<EguiClipboard>,
    mut event: EventWriter<RequestRedraw>,
) {
    for (&window_id, ctx) in context.ctx.iter_mut() {
        let full_output = ctx.end_frame();
        let egui::FullOutput {
            platform_output,
            shapes,
            textures_delta,
            needs_repaint,
        } = full_output;

        let render_output = render_output.entry(window_id).or_default();
        render_output.shapes = shapes;
        render_output.textures_delta.append(textures_delta);

        output.entry(window_id).or_default().platform_output = platform_output.clone();

        if !platform_output.copied_text.is_empty() {
            clipboard.set_contents(&platform_output.copied_text);
        }

        if let Some(ref mut windows) = windows {
            if let Some(window) = windows.get_mut(window_id) {
                if let Some(cursor) = convert::cursor_icon(platform_output.cursor_icon) {
                    window.set_cursor_icon(cursor);
                    if !window.cursor_visible() {
                        window.set_cursor_visibility(true);
                    }
                } else if window.cursor_visible() {
                    window.set_cursor_visibility(false);
                }
            }
        }

        if needs_repaint {
            event.send(RequestRedraw)
        }

        // TODO: see if we can support `new_tab`.
        if let Some(egui::output::OpenUrl { url, new_tab: _ }) = platform_output.open_url {
            if let Err(err) = webbrowser::open(&url) {
                log::error!("Failed to open '{}': {:?}", url, err);
            }
        }
    }
}
