#![allow(unused)]
use glfw::ffi::*;
use glfw::{Action, Context, Key, PWindow};
use std::ffi::c_int;
use crate::toolbox::input::Input;
use crate::toolbox::logging::LOGGER;

pub struct DisplayManager {
    width: u32,
    height: u32,
    title: &'static str,
    delta: f32,
    last_frame_time : f32,
    window: Option<PWindow>,
    glfw: Option<glfw::Glfw>,
    events: Option<glfw::GlfwReceiver<(f64, glfw::WindowEvent)>>,
    input: Input,
}

impl DisplayManager {
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

    pub fn create_display(&mut self) {
        use glfw::fail_on_errors;

        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        // glfw.set_error_callback(
        //     |_, description| eprintln!("GLFW Error: {}", description)
        // );
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        let (window, events) = glfw.create_window(self.width, self.height, &self.title, glfw::WindowMode::Windowed).expect("Failed to create GLFW window.");
        self.glfw = Some(glfw);
        self.window = Some(window);
        self.events = Some(events);

        let window = self.window.as_mut().unwrap();

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        let win = window.window_ptr();
        unsafe {
            let vid_mode = glfwGetVideoMode(glfwGetPrimaryMonitor());
            let (width, height) = ((*vid_mode).width, (*vid_mode).height);
            glfwSetWindowPos(win, (width - (self.width as c_int)) / 2, (height - (self.height as c_int)) / 2);
            glfwMakeContextCurrent(win)
        }

        gl::load_with(|s| window.get_proc_address(s) as *const _);
        let version = unsafe {
            let data = gl::GetString(gl::VERSION);
            let version = std::ffi::CStr::from_ptr(data as *const i8).to_str().unwrap();
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

    pub fn size_handler(&mut self) {
        let mut width: c_int = 0;
        let mut height: c_int = 0 ;
        unsafe {
            glfwGetWindowSize(self.window.as_ref().unwrap().window_ptr(), &mut width, &mut height);
        }
        self.width = width as u32;
        self.height = height as u32;
    }

    fn handle_window_events(&mut self) {
        self.glfw.as_mut().unwrap().poll_events();
        for (_, event) in glfw::flush_messages(self.events.as_ref().unwrap()) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.as_mut().unwrap().set_should_close(true)
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    self.input.key_handler(action, key);
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    self.input.set_mouse_pos(x, y);
                }
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    self.input.mouse_button_handler(action, button);
                }
                _ => {}
            }
        }
    }
    
    fn get_current_time(&self) -> f32 {
        unsafe {  (glfwGetTime() * 1000.0 / glfwGetTimerFrequency() as f64) as f32 }
    }
    pub fn update_display(&mut self) {
        self.handle_window_events();
        self.size_handler();
        let current_time = self.get_current_time();
        self.delta = (current_time - self.last_frame_time) / 1000.0;
        self.last_frame_time = current_time;
        unsafe { 
            gl::Viewport(0, 0, self.width as i32, self.height as i32); 
        }
        self.window.as_mut().unwrap().swap_buffers();
    }
    
    pub fn is_close_requested(&self) -> bool {
        self.window.as_ref().unwrap().should_close()
    }
    
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
    
    pub fn get_delta(&self) -> f32 {
        self.delta
    }
    
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    pub fn get_window(&self) -> &PWindow {
        self.window.as_ref().unwrap()
    }
    
    pub fn get_glfw(&self) -> &glfw::Glfw {
        self.glfw.as_ref().unwrap()
    }
}

impl Drop for DisplayManager {
    fn drop(&mut self) {
        println!("Cleaning up resources...");
        self.close_display(); // Ensure no manual cleanup is missed
        self.glfw.take(); // Drop the glfw instance properly
    }
}

