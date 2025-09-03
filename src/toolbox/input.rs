use glfw::{Action, Key};

pub struct Input {
    input_keyboard: Vec<Key>,
    pub mouse_pos: (f64, f64),
    pub d_mouse_pos: (f64, f64),
    left_mouse_button: bool,
    right_mouse_button: bool,
}

impl Input {
    pub fn new() -> Input {
        Input {
            input_keyboard: Vec::new(),
            mouse_pos: (0.0, 0.0),
            d_mouse_pos: (0.0, 0.0),
            left_mouse_button: false,
            right_mouse_button: false,
        }
    }

    pub fn key_handler(&mut self, action: Action, key: Key) {
        if action == Action::Press {
            if !self.input_keyboard.contains(&key) {
                self.input_keyboard.push(key);
            }
        } else if action == Action::Release {
            self.input_keyboard.retain(|&x| x != key);
        }
    }

    pub fn set_mouse_pos(&mut self, x: f64, y: f64) {
        self.d_mouse_pos = (x - self.mouse_pos.0, y - self.mouse_pos.1);
        self.mouse_pos = (x, y);
    }

    pub fn mouse_button_handler(&mut self, action: Action, button: glfw::MouseButton) {
        match button {
            glfw::MouseButton::Left => {
                if action == Action::Press {
                    self.left_mouse_button = true;
                } else if action == Action::Release {
                    self.left_mouse_button = false;
                }
            },
            glfw::MouseButton::Right => {
                if action == Action::Press {
                    self.right_mouse_button = true;
                } else if action == Action::Release {
                    self.right_mouse_button = false;
                }
            },
            _ => {}
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.input_keyboard.contains(&key)
    }
    
    pub fn is_mouse_button_pressed(&self, button: glfw::MouseButton) -> bool {
        match button {
            glfw::MouseButton::Left => self.left_mouse_button,
            glfw::MouseButton::Right => self.right_mouse_button,
            _ => false,
        }
    }
}