#![deny(warnings, clippy::pedantic)]
#![warn(rust_2018_idioms)]

use cocoa::appkit::{NSColor, NSWindow};
use winit::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    monitor::MonitorHandle,
    platform::macos::WindowExtMacOS,
    window::{Fullscreen, Window, WindowBuilder, WindowId},
};

fn set_background_color(window: &Window, color: &Color) {
    // ALLOWED: cocoa crate exposes `*mut objc::runtime::Object`, therefore using cast would create
    // a pointer to a pointer. Better to just allow it
    #[allow(clippy::ptr_as_ptr)]
    let ns_window = window.ns_window() as cocoa::base::id;
    unsafe { ns_window.setBackgroundColor_(color.id()) };
}

fn list_monitors(event_loop: &EventLoopWindowTarget<()>) -> Vec<MonitorHandle> {
    let mut monitors = event_loop.available_monitors().collect::<Vec<_>>();
    if let Some(primary_monitor) = event_loop.primary_monitor() {
        if let Some(index) = monitors
            .iter()
            .position(|monitor| *monitor == primary_monitor)
        {
            let last = monitors.len() - 1;
            monitors.swap(index, last);
        }
    }
    monitors
}

fn build_window(
    event_loop: &EventLoopWindowTarget<()>,
    color: &Color,
    monitor: MonitorHandle,
) -> Window {
    let window = WindowBuilder::new()
        .with_title("Blank")
        .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
        .build(event_loop)
        .unwrap();
    set_background_color(&window, color);
    window
}

fn choose_windows(
    event_loop: &EventLoopWindowTarget<()>,
    color: &Color,
    dark: bool,
) -> Vec<Window> {
    let available_monitors = list_monitors(event_loop);

    let count = if dark || available_monitors.len() < 2 {
        available_monitors.len()
    } else {
        available_monitors.len() - 1
    };

    available_monitors
        .into_iter()
        .take(count)
        .map(|monitor| build_window(event_loop, color, monitor))
        .collect()
}

fn add_window(
    event_loop: &EventLoopWindowTarget<()>,
    color: &Color,
    current_windows: &[Window],
) -> Option<Window> {
    list_monitors(event_loop)
        .into_iter()
        .find(|monitor| {
            current_windows
                .iter()
                .filter_map(Window::current_monitor)
                .all(|open| *monitor != open)
        })
        .map(|monitor| build_window(event_loop, color, monitor))
}

struct Color {
    color_id: cocoa::base::id,
    temperature: u32,
    dark: bool,
}

impl Color {
    fn new(dark: bool) -> Self {
        let mut color = Self {
            color_id: cocoa::base::nil,
            temperature: 5500,
            dark,
        };
        color.update();
        color
    }

    fn update(&mut self) {
        if self.dark {
            self.color_id =
                unsafe { NSColor::colorWithRed_green_blue_alpha_(self.color_id, 0., 0., 0., 1.) };
        } else {
            let (r, g, b) = tempergb::rgb_from_temperature(self.temperature).into();
            self.color_id = unsafe {
                NSColor::colorWithRed_green_blue_alpha_(
                    self.color_id,
                    f64::from(r) / 255.0,
                    f64::from(g) / 255.0,
                    f64::from(b) / 255.0,
                    1.0,
                )
            };
        }
    }

    fn increase(&mut self) -> bool {
        if self.temperature < 6600 {
            self.temperature += 100;
            self.update();
            true
        } else {
            false
        }
    }

    fn toggle(&mut self) {
        self.dark = !self.dark;
        self.update();
    }

    fn decrease(&mut self) -> bool {
        if self.temperature > 1500 {
            self.temperature -= 100;
            self.update();
            true
        } else {
            false
        }
    }

    fn id(&self) -> cocoa::base::id {
        self.color_id
    }
}

fn remove(windows: &mut Vec<Window>, id: WindowId) -> bool {
    if let Some(index) = windows.iter().position(|window| window.id() == id) {
        windows.swap_remove(index);
        true
    } else {
        false
    }
}

#[allow(clippy::too_many_lines)]
fn main() {
    let dark = match std::env::args().nth(1).as_deref() {
        Some("b" | "bright") => false,
        Some("d" | "dark") | None => true,
        Some(s) => {
            eprintln!("Unrecognized parameter `{s}`. Expected `b`, `bright`, `d`, `dark`, or none");
            std::process::exit(1);
        }
    };
    let mut current_modifiers = ModifiersState::default();
    let mut released_a = true;
    let mut released_w = true;
    let mut released_q = true;
    let mut graceful = false;
    let mut color = Color::new(dark);

    let event_loop = EventLoop::new();

    let mut windows = choose_windows(&event_loop, &color, dark);

    nosleep::NoSleep::new()
        .unwrap()
        .start(nosleep::NoSleepType::PreventUserIdleDisplaySleep)
        .unwrap();

    event_loop.run(move |event, event_loop, control_flow| {
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
                    if remove(&mut windows, window_id) && windows.is_empty() {
                        graceful = true;
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    current_modifiers = modifiers;
                }
                WindowEvent::ReceivedCharacter('=') => {
                    if color.increase() {
                        for window in &windows {
                            set_background_color(window, &color);
                        }
                    }
                }
                WindowEvent::ReceivedCharacter('-') => {
                    if color.decrease() {
                        for window in &windows {
                            set_background_color(window, &color);
                        }
                    }
                }
                WindowEvent::ReceivedCharacter('b') => {
                    color.toggle();
                    for window in &windows {
                        set_background_color(window, &color);
                    }
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
                        if remove(&mut windows, window_id) && windows.is_empty() {
                            graceful = true;
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    (VirtualKeyCode::A, ElementState::Released) => {
                        released_a = true;
                    }
                    (VirtualKeyCode::A, ElementState::Pressed)
                        if released_a && current_modifiers == ModifiersState::LOGO =>
                    {
                        released_a = false;
                        if let Some(window) = add_window(event_loop, &color, &windows) {
                            windows.push(window);
                        }
                    }
                    (VirtualKeyCode::W, ElementState::Released) => {
                        released_w = true;
                    }
                    (VirtualKeyCode::W, ElementState::Pressed)
                        if released_w && current_modifiers == ModifiersState::LOGO =>
                    {
                        released_w = false;
                        if remove(&mut windows, window_id) && windows.is_empty() {
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
                        if let Some(window) = windows.iter().find(|window| window.id() == window_id)
                        {
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
