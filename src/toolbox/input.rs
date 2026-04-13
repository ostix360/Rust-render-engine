//! Per-frame keyboard and mouse state collected from GLFW events.

use glfw::{Action, Key};

pub struct Input {
    input_keyboard: Vec<Key>,
    pressed_this_frame: Vec<Key>,
    pub mouse_pos: (f64, f64),
    pub d_mouse_pos: (f64, f64),
    left_mouse_button: bool,
    right_mouse_button: bool,
}

impl Input {
    /// Creates an empty per-frame input snapshot.
    pub fn new() -> Input {
        Input {
            input_keyboard: Vec::new(),
            pressed_this_frame: Vec::new(),
            mouse_pos: (0.0, 0.0),
            d_mouse_pos: (0.0, 0.0),
            left_mouse_button: false,
            right_mouse_button: false,
        }
    }

    /// Clears the edge-triggered keyboard state for the new frame.
    ///
    /// Keys that remain held stay in `input_keyboard`; only the one-frame press edges are reset.
    pub fn begin_frame(&mut self) {
        self.pressed_this_frame.clear();
    }

    /// Updates the cached keyboard state from one GLFW key event.
    ///
    /// `Action::Repeat` intentionally does not retrigger `pressed_this_frame`, which keeps hotkey
    /// handlers like tangent-mode toggles edge-triggered.
    pub fn key_handler(&mut self, action: Action, key: Key) {
        if action == Action::Press {
            if !self.input_keyboard.contains(&key) {
                self.input_keyboard.push(key);
            }
            if !self.pressed_this_frame.contains(&key) {
                self.pressed_this_frame.push(key);
            }
        } else if action == Action::Release {
            self.input_keyboard.retain(|&x| x != key);
        }
    }

    /// Updates the cached mouse position and stores the per-frame mouse delta.
    pub fn set_mouse_pos(&mut self, x: f64, y: f64) {
        self.d_mouse_pos = (x - self.mouse_pos.0, y - self.mouse_pos.1);
        self.mouse_pos = (x, y);
    }

    /// Updates the cached mouse button state from one GLFW mouse event.
    pub fn mouse_button_handler(&mut self, action: Action, button: glfw::MouseButton) {
        match button {
            glfw::MouseButton::Left => {
                if action == Action::Press {
                    self.left_mouse_button = true;
                } else if action == Action::Release {
                    self.left_mouse_button = false;
                }
            }
            glfw::MouseButton::Right => {
                if action == Action::Press {
                    self.right_mouse_button = true;
                } else if action == Action::Release {
                    self.right_mouse_button = false;
                }
            }
            _ => {}
        }
    }

    /// Returns whether the key is currently held down.
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_keyboard.contains(&key)
    }

    /// Returns whether the key transitioned to pressed during the current frame.
    pub fn is_key_just_pressed(&self, key: Key) -> bool {
        self.pressed_this_frame.contains(&key)
    }

    /// Returns whether the requested mouse button is currently held down.
    pub fn is_mouse_button_pressed(&self, button: glfw::MouseButton) -> bool {
        match button {
            glfw::MouseButton::Left => self.left_mouse_button,
            glfw::MouseButton::Right => self.right_mouse_button,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Input;
    use glfw::{Action, Key};

    #[test]
    fn just_pressed_only_last_for_one_frame() {
        let mut input = Input::new();

        input.begin_frame();
        input.key_handler(Action::Press, Key::T);

        assert!(input.is_key_pressed(Key::T));
        assert!(input.is_key_just_pressed(Key::T));

        input.begin_frame();

        assert!(input.is_key_pressed(Key::T));
        assert!(!input.is_key_just_pressed(Key::T));
    }

    #[test]
    fn holding_key_does_not_retrigger_press_edge() {
        let mut input = Input::new();

        input.begin_frame();
        input.key_handler(Action::Press, Key::T);
        assert!(input.is_key_just_pressed(Key::T));

        input.begin_frame();
        input.key_handler(Action::Repeat, Key::T);

        assert!(input.is_key_pressed(Key::T));
        assert!(!input.is_key_just_pressed(Key::T));

        input.key_handler(Action::Release, Key::T);
        assert!(!input.is_key_pressed(Key::T));
    }
}
