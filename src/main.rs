#![deny(warnings, clippy::pedantic)]
#![warn(rust_2018_idioms)]

use cocoa::appkit::{NSColor, NSWindow};
use std::collections::HashMap;
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::macos::WindowExtMacOS,
    window::{Fullscreen, Window, WindowBuilder, WindowId},
};

fn ns_color_from_temperature(temperature: u32, old_id: cocoa::base::id) -> cocoa::base::id {
    let (r, g, b) = tempergb::rgb_from_temperature(temperature).into();
    unsafe {
        NSColor::colorWithRed_green_blue_alpha_(
            old_id,
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
            1.0,
        )
    }
}

fn set_background_color(window: &Window, color: cocoa::base::id) {
    // ALLOWED: cocoa crate exposes `*mut objc::runtime::Object`, therefore using cast would create
    // a pointer to a pointer. Better to just allow it
    #[allow(clippy::ptr_as_ptr)]
    let ns_window = window.ns_window() as cocoa::base::id;
    unsafe { ns_window.setBackgroundColor_(color) };
}

fn choose_windows(event_loop: &EventLoop<()>, color: cocoa::base::id) -> HashMap<WindowId, Window> {
    let build_window = |monitor| {
        let window = WindowBuilder::new()
            .with_title("Blank")
            .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
            .build(event_loop)
            .unwrap();
        set_background_color(&window, color);
        (window.id(), window)
    };

    if let Some(primary_monitor) = event_loop.primary_monitor() {
        if event_loop.available_monitors().count() > 1 {
            return event_loop
                .available_monitors()
                .filter_map(|monitor| {
                    if monitor == primary_monitor {
                        None
                    } else {
                        Some(build_window(monitor))
                    }
                })
                .collect();
        }
    }

    event_loop.available_monitors().map(build_window).collect()
}

#[allow(clippy::too_many_lines)]
fn main() {
    let mut current_modifiers = ModifiersState::default();
    let mut released_w = true;
    let mut released_q = true;
    let mut graceful = false;
    let mut temperature = 4300;
    let mut color = ns_color_from_temperature(temperature, cocoa::base::nil);

    let event_loop = EventLoop::new();

    let mut windows = choose_windows(&event_loop, color);

    event_loop.run(move |event, _, control_flow| {
        if *control_flow == ControlFlow::Exit {
            if let Event::NewEvents(winit::event::StartCause::Poll) = event {
                if windows.is_empty() {
                    if graceful {
                        graceful = false;
                    } else {
                        panic!("Force exit");
                    }
                } else {
                    windows.clear();
                }
            }
        }
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent { event, window_id } = event {
            match event {
                WindowEvent::CloseRequested => {
                    windows.remove(&window_id);
                    if windows.is_empty() {
                        graceful = true;
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    current_modifiers = modifiers;
                }
                WindowEvent::ReceivedCharacter('=') if temperature < 6600 => {
                    temperature += 100;
                    color = ns_color_from_temperature(temperature, color);
                    windows
                        .iter()
                        .for_each(|window| set_background_color(window.1, color));
                }
                WindowEvent::ReceivedCharacter('-') if temperature > 1500 => {
                    temperature -= 100;
                    color = ns_color_from_temperature(temperature, color);
                    windows
                        .iter()
                        .for_each(|window| set_background_color(window.1, color));
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => match (virtual_code, state) {
                    (VirtualKeyCode::Escape, ElementState::Released) => {
                        windows.remove(&window_id);
                        if windows.is_empty() {
                            graceful = true;
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    (VirtualKeyCode::W, ElementState::Released) => {
                        released_w = true;
                    }
                    (VirtualKeyCode::W, ElementState::Pressed)
                        if released_w && current_modifiers == ModifiersState::LOGO =>
                    {
                        released_w = false;
                        windows.remove(&window_id);
                        if windows.is_empty() {
                            graceful = true;
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    (VirtualKeyCode::Q, ElementState::Released) => {
                        released_q = true;
                    }
                    (VirtualKeyCode::Q, ElementState::Pressed)
                        if released_q && current_modifiers == ModifiersState::LOGO =>
                    {
                        released_q = false;
                        *control_flow = ControlFlow::Exit;
                    }
                    (VirtualKeyCode::F, ElementState::Released) => {
                        if let Some(window) = windows.get(&window_id) {
                            if window.fullscreen().is_some() {
                                window.set_fullscreen(None);
                            } else {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    });
}
