#![allow(unused)]
//! GLFW window creation, event polling, and frame-timing utilities.

use crate::toolbox::input::Input;
use crate::toolbox::logging::LOGGER;
use glfw::ffi::*;
use glfw::{Action, Context, Key, PWindow};
use std::ffi::{c_int, c_void};

pub struct DisplayManager {
    width: u32,
    height: u32,
    title: &'static str,
    delta: f32,
    last_frame_time: f32,
    window: Option<PWindow>,
    glfw: Option<glfw::Glfw>,
    events: Option<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
    input: Input,
}

impl DisplayManager {
    /// Creates the window manager with the requested initial size and title.
    pub fn new(width: u32, height: u32, title: &'static str) -> DisplayManager {
        DisplayManager {
            width,
            height,
            title,
            delta: 0.0,
            last_frame_time: 0.0,
            window: None,
            glfw: None,
            events: None,
            input: Input::new(),
        }
    }

    /// Initializes GLFW, creates the OpenGL window, and loads GL function pointers.
    ///
    /// The created context remains owned by this thread for the rest of the program. Other
    /// threads may manage UI state, but they do not participate in OpenGL calls through this
    /// manager.
    pub fn create_display(&mut self) {
        use glfw::fail_on_errors;

        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        // glfw.set_error_callback(
        //     |_, description| eprintln!("GLFW Error: {}", description)
        // );
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        let (window, events) = glfw
            .create_window(
                self.width,
                self.height,
                &self.title,
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window.");
        self.glfw = Some(glfw);
        self.window = Some(window);
        self.events = Some(events);

        let window = self.window.as_mut().unwrap();

        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);
        let win = window.window_ptr();
        unsafe {
            let vid_mode = glfwGetVideoMode(glfwGetPrimaryMonitor());
            let (width, height) = ((*vid_mode).width, (*vid_mode).height);
            glfwSetWindowPos(
                win,
                (width - (self.width as c_int)) / 2,
                (height - (self.height as c_int)) / 2,
            );
            glfwMakeContextCurrent(win)
        }

        gl::load_with(|s| {
            window
                .get_proc_address(s)
                .map_or(std::ptr::null(), |proc| proc as *const () as *const c_void)
        });
        let version = unsafe {
            let data = gl::GetString(gl::VERSION);
            let version = std::ffi::CStr::from_ptr(data as *const i8)
                .to_str()
                .unwrap();
            version
        };

        LOGGER.debug(format!("OpenGL version {}", version).as_str());
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
        glfw::SwapInterval::Sync(1);
        unsafe {
            gl::Enable(gl::MULTISAMPLE);
        }
    }

    /// Refreshes the cached window size from GLFW.
    pub fn size_handler(&mut self) {
        let mut width: c_int = 0;
        let mut height: c_int = 0;
        unsafe {
            glfwGetWindowSize(
                self.window.as_ref().unwrap().window_ptr(),
                &mut width,
                &mut height,
            );
        }
        self.width = width as u32;
        self.height = height as u32;
    }

    /// Polls GLFW events and updates the cached keyboard and mouse input state.
    ///
    /// `Input::begin_frame` is called here so edge-triggered key state is defined in display
    /// frames rather than wall-clock time. This function is the only place that mutates the
    /// per-frame input snapshot.
    fn handle_window_events(&mut self) {
        self.input.begin_frame();
        self.glfw.as_mut().unwrap().poll_events();
        let (x, y) = self.window.as_ref().unwrap().get_cursor_pos();
        self.input.set_mouse_pos(x, y);
        for (_, event) in glfw::flush_messages(self.events.as_ref().unwrap()) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.as_mut().unwrap().set_should_close(true)
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    self.input.key_handler(action, key);
                }
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    self.input.mouse_button_handler(action, button);
                }
                _ => {}
            }
        }
    }

    /// Returns the current GLFW time in seconds.
    fn get_current_time(&self) -> f32 {
        unsafe { glfwGetTime() as f32 }
    }
    /// Finishes one frame by polling events, updating timing, resizing the viewport, and
    /// swapping buffers.
    ///
    /// `delta` is computed after event handling, so systems reading it on the next frame observe
    /// the duration of the frame that just completed.
    pub fn update_display(&mut self) {
        self.handle_window_events();
        self.size_handler();
        let current_time = self.get_current_time();
        self.delta = current_time - self.last_frame_time;
        self.last_frame_time = current_time;
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
        self.window.as_mut().unwrap().swap_buffers();
    }

    /// Returns whether the window has requested shutdown.
    pub fn is_close_requested(&self) -> bool {
        self.window.as_ref().unwrap().should_close()
    }

    /// Drops the window and event receiver owned by the display manager.
    pub fn close_display(&mut self) {
        println!("Closing display");
        // self.glfw.as_mut().unwrap().unset_error_callback();
        // unsafe {
        //     glfwDestroyWindow(self.window.as_ref().unwrap().window_ptr());
        //     glfwTerminate();
        // }
        self.events.take();
        self.window.take();
    }

    /// Returns the frame delta computed during the last `update_display` call.
    pub fn get_delta(&self) -> f32 {
        self.delta
    }

    /// Returns the cached window width in pixels.
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the cached window height in pixels.
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Returns the cached per-frame input state.
    pub fn get_input(&self) -> &Input {
        &self.input
    }

    /// Returns a mutable reference to the GLFW window.
    pub fn get_window(&mut self) -> &mut PWindow {
        self.window.as_mut().unwrap()
    }

    /// Returns a mutable reference to the GLFW window.
    ///
    /// This is identical to `get_window` and exists for call-site clarity.
    pub fn get_window_mut(&mut self) -> &mut PWindow {
        self.window.as_mut().unwrap()
    }

    /// Returns the underlying GLFW handle.
    pub fn get_glfw(&self) -> &glfw::Glfw {
        self.glfw.as_ref().unwrap()
    }

    /// Returns a mutable reference to the underlying GLFW handle.
    pub fn get_glfw_mut(&mut self) -> &mut glfw::Glfw {
        self.glfw.as_mut().unwrap()
    }
}

impl Drop for DisplayManager {
    /// Cleans up the window and GLFW state owned by the display manager.
    fn drop(&mut self) {
        println!("Cleaning up resources...");
        self.close_display(); // Ensure no manual cleanup is missed
        self.glfw.take(); // Drop the glfw instance properly
    }
}
