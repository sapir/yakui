mod keys;

use winit::event::{
    DeviceEvent, ElementState, Event as WinitEvent, MouseButton as WinitMouseButton, WindowEvent,
};
use winit::window::Window;
use yakui_core::event::Event;
use yakui_core::geometry::{Rect, Vec2};
use yakui_core::input::MouseButton;

pub use self::keys::from_winit_key;

#[non_exhaustive]
pub struct State {
    auto_scale: bool,
    unapplied_scale: Option<f32>,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new(window: &Window) -> Self {
        let unapplied_scale = Some(window.scale_factor() as f32);

        Self {
            auto_scale: true,
            unapplied_scale,
        }
    }

    /// Configure whether scale factor (ie DPI) should be automatically applied
    /// from the window to scale the yakui UI.
    ///
    /// Defaults to `true`.
    pub fn set_automatic_scale_factor(&mut self, enabled: bool) {
        self.auto_scale = enabled;
    }

    // TODO: How do we determine if an input event should be sunk by the UI?
    pub fn handle_event<T>(
        &mut self,
        state: &mut yakui_core::State,
        event: &WinitEvent<T>,
    ) -> bool {
        if let Some(scale) = self.unapplied_scale {
            if self.auto_scale {
                state.set_scale_factor(scale);
            }
        }

        #[allow(clippy::single_match)]
        match event {
            WinitEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let rect = Rect::from_pos_size(
                    Vec2::ZERO,
                    Vec2::new(size.width as f32, size.height as f32),
                );

                state.handle_event(Event::ViewportChanged(rect))
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                ..
            } => {
                if self.auto_scale {
                    state.set_scale_factor(*scale_factor as f32)
                }

                false
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let pos = Vec2::new(position.x as f32, position.y as f32);
                state.handle_event(Event::CursorMoved(Some(pos)))
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                ..
            } => state.handle_event(Event::CursorMoved(None)),
            WinitEvent::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button,
                        state: button_state,
                        ..
                    },
                ..
            } => {
                let button = match button {
                    WinitMouseButton::Left => MouseButton::One,
                    WinitMouseButton::Right => MouseButton::Two,
                    WinitMouseButton::Middle => MouseButton::Three,
                    WinitMouseButton::Other(_) => return false,
                };

                let down = match button_state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                state.handle_event(Event::MouseButtonChanged(button, down))
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => state.handle_event(Event::TextInput(*c)),
            WinitEvent::DeviceEvent {
                event: DeviceEvent::Key(input),
                ..
            } => {
                if let Some(key) = input.virtual_keycode.and_then(from_winit_key) {
                    let pressed = match input.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    state.handle_event(Event::KeyChanged(key, pressed))
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
