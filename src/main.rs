#[cfg(feature = "druid")]
mod druid {
    use druid::widget::{Button, Flex, Label};
    use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};

    fn ui_builder() -> impl Widget<()> {
        Flex::column()
    }

    pub fn run() {
        let main_window = WindowDesc::new(ui_builder);
        let _ = AppLauncher::with_window(main_window).launch(());
    }
}
#[cfg(feature = "druid")]
use crate::druid as runner;

#[cfg(feature = "orbtk")]
mod orbtk {
    use orbtk::prelude::*;

    pub fn run() {
        // let theme = Theme::from_config(orbtk::theming::config::ThemeConfig::from(
        //     orbtk::theme::LIGHT_THEME_RON,
        // ));

        Application::new()
            // .theme(theme)
            .window(|ctx| {
                Window::new()
                    .title("Blank Screen - OrbTk")
                    .background(Color::rgb(255, 204, 128))
                    .resizeable(true)
                    .size(1024.0, 768.0)
                    .build(ctx)
            })
            .run();
    }
}
#[cfg(feature = "orbtk")]
use crate::orbtk as runner;

#[cfg(feature = "web-view")]
mod webview {
    use web_view::*;

    pub fn run() {
        web_view::builder()
            .title("Blank Screen - WebView")
            .content(Content::Html("<html><body></body></html>"))
            .size(1024, 768)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .run()
            .unwrap();
    }
}
#[cfg(feature = "web-view")]
use crate::webview as runner;

#[cfg(feature = "win")]
mod winit {
    use winit::{
        event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::macos::WindowExtMacOS,
        window::{Fullscreen, WindowBuilder},
    };

    pub fn run() {
        let event_loop = EventLoop::new();

        let color = unsafe {
            cocoa::appkit::NSColor::colorWithRed_green_blue_alpha_(
                cocoa::base::nil,
                1.0,
                0.8,
                0.5,
                1.0,
            )
        };

        let mut windows = event_loop
            .available_monitors()
            .map(|monitor| {
                let window = WindowBuilder::new()
                    .with_title("Black Screen - Winit")
                    .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
                    .build(&event_loop)
                    .unwrap();
                use cocoa::appkit::NSWindow;
                let ns_window = window.ns_window() as cocoa::base::id;
                unsafe { ns_window.setBackgroundColor_(color) };

                (window.id(), window)
            })
            .collect::<std::collections::HashMap<_, _>>();

        let mut current_modifiers = ModifiersState::default();
        let mut released_w = true;
        let mut released_q = true;
        let mut graceful = false;

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

            match event {
                Event::WindowEvent { event, window_id } => match event {
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
                },
                _ => {}
            }
        });
    }
}
#[cfg(feature = "win")]
use crate::winit as runner;

#[cfg(feature = "glium")]
mod glium;
#[cfg(feature = "glium")]
use crate::glium as runner;

fn main() {
    runner::run();
}
