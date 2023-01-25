use libc;

use crate::{Key, KeyboardControllable, MouseButton, MouseControllable};

use self::libc::{c_char, c_int, c_uint, c_long, c_void, useconds_t};
use std::{borrow::Cow, ffi::CString, ptr};

const CURRENT_WINDOW: c_int = 0;
const DEFAULT_DELAY: u64 = 12000;
type Window = c_int;
type Xdo = *const c_void;

#[repr(C)]
#[derive(Copy)]
pub struct Search {
    title: *const c_char,           /** pattern to test against a window title */
    winclass: *const c_char,        /** pattern to test against a window class */
    winclassname: *const c_char,    /** pattern to test against a window class */
    winname: *const c_char,      /** pattern to test against a window name */
    winrole: *const c_char,      /** pattern to test against a window role */
    pid: c_int,            /** window pid (From window atom _NET_WM_PID) */
    max_depth: c_long,     /** depth of search. 1 means only toplevel windows */
    only_visible: c_int,   /** boolean; set true to search only visible windows */
    screen: c_int,         /** what screen to search, if any. If none given, search 
                           all screens */
  
    /** Should the tests be 'and' or 'or' ? If 'and', any failure will skip the
     * window. If 'or', any success will keep the window in search results. */
    require: c_uint,
    
    /** bitmask of things you are searching for, such as SEARCH_NAME, etc.
     * @see SEARCH_NAME, SEARCH_CLASS, SEARCH_PID, SEARCH_CLASSNAME, etc
     */
    searchmask: c_uint,
  
    /** What desktop to search, if any. If none given, search all screens. */
    desktop: c_long,
  
    /** How many results to return? If 0, return all. */
    limit: c_uint,
}
impl std::clone::Clone for Search {
    fn clone(&self) -> Self { *self }
}
impl std::default::Default for Search {
    fn default() -> Self { unsafe { std::mem::zeroed() } }
}

#[link(name = "xdo")]
extern "C" {
    fn xdo_free(xdo: Xdo);
    fn xdo_new(display: *const c_char) -> Xdo;
    fn xdo_focus_window(xdo: Xdo, window: Window) -> c_int;
    fn xdo_get_pid_window(xdo: Xdo, window: Window) -> c_int;
    fn xdo_search_windows(xdo: Xdo, search: *const c_void,
        windowlist_ret: *mut *mut Window, nwindows_ret: *mut c_uint) -> c_int;
    fn xdo_click_window(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_mouse_down(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_mouse_up(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_move_mouse(xdo: Xdo, x: c_int, y: c_int, screen: c_int) -> c_int;
    fn xdo_move_mouse_relative(xdo: Xdo, x: c_int, y: c_int) -> c_int;
    fn xdo_enter_text_window(xdo: Xdo, 
        window: Window, string: *const c_char, delay: useconds_t) -> c_int;
    fn xdo_send_keysequence_window(xdo: Xdo, 
        window: Window, string: *const c_char, delay: useconds_t) -> c_int;
    fn xdo_send_keysequence_window_down(xdo: Xdo,
        window: Window,string: *const c_char, delay: useconds_t) -> c_int;
    fn xdo_send_keysequence_window_up(xdo: Xdo,
        window: Window, string: *const c_char, delay: useconds_t) -> c_int;
}

fn mousebutton(button: MouseButton) -> c_int {
    match button {
        MouseButton::Left => 1,
        MouseButton::Middle => 2,
        MouseButton::Right => 3,
        MouseButton::ScrollUp => 4,
        MouseButton::ScrollDown => 5,
        MouseButton::ScrollLeft => 6,
        MouseButton::ScrollRight => 7,
    }
}

/// The main struct for handling the event emitting
pub struct Enigo {
    xdo: Xdo,
    delay: u64,
    window: i32,    
}
// This is safe, we have a unique pointer.
// TODO: use Unique<c_char> once stable.
unsafe impl Send for Enigo {}

impl Default for Enigo {
    /// Create a new Enigo instance
    fn default() -> Self {
        Self {
            xdo: unsafe { xdo_new(ptr::null()) },
            delay: DEFAULT_DELAY,
            window: CURRENT_WINDOW,
        }
    }
}
impl Enigo {
    /// Get the delay per keypress.
    /// Default value is 12000.
    /// This is Linux-specific.
    pub fn delay(&self) -> u64 {
        self.delay
    }
    /// Set the delay per keypress.
    /// This is Linux-specific.
    pub fn set_delay(&mut self, delay: u64) {
        self.delay = delay;
    }
    /// Get the window ID.
    /// Default value is 0.
    /// This is Linux-specific.
    pub fn window(&self) -> i32 {
        self.window
    }
    /// Set the window ID.
    /// This is Linux-specific.
    pub fn set_window(&mut self, window: i32) {
        self.window = window;
    }
    /// Get the focus in current window ID
    /// This is Linux-specific
    pub fn window_focus(&mut self) -> i32{
        unsafe {
            xdo_focus_window(self.xdo, self.window)
        }
    }
    /// Get pid of window ID
    /// This is Linux-specific
    pub fn window_pid(&mut self) -> i32 {
        unsafe {
            xdo_get_pid_window(self.xdo, self.window)
        }
    }
    /// Search window by pid
    /// This is Linux-specific
    pub fn search_window_by_pid(&mut self, pid: i32) -> i32 {
        let search = Search {
            pid: pid as c_int,            
            max_depth: 100 as c_long,    
            searchmask: (1u64 << 3) as c_uint, 
            ..Search::default()
        };
        let search_ptr: *const c_void = &search as *const _ as *const c_void;
        let mut list: *mut i32 = std::ptr::null_mut();
        let list_ptr: *mut *mut i32 = &mut list;
        let mut count: u32 = 0;
        let count_ptr: *mut u32 = &mut count;
        
        let output = unsafe {
            xdo_search_windows(self.xdo, search_ptr, list_ptr, count_ptr);  
            *count_ptr as u32 
        };
        println!("number of windows: {}", output);
        output as i32
    }
}
impl Drop for Enigo {
    fn drop(&mut self) {
        unsafe {
            xdo_free(self.xdo);
        }
    }
}
impl MouseControllable for Enigo {
    fn mouse_move_to(&mut self, x: i32, y: i32) {
        unsafe {
            xdo_move_mouse(self.xdo, x as c_int, y as c_int, 0);
        }
    }
    fn mouse_move_relative(&mut self, x: i32, y: i32) {
        unsafe {
            xdo_move_mouse_relative(self.xdo, x as c_int, y as c_int);
        }
    }
    fn mouse_down(&mut self, button: MouseButton) {
        unsafe {
            xdo_mouse_down(self.xdo, self.window, mousebutton(button));
        }
    }
    fn mouse_up(&mut self, button: MouseButton) {
        unsafe {
            xdo_mouse_up(self.xdo, self.window, mousebutton(button));
        }
    }
    fn mouse_click(&mut self, button: MouseButton) {
        unsafe {
            xdo_click_window(self.xdo, self.window, mousebutton(button));
        }
    }
    fn mouse_scroll_x(&mut self, length: i32) {
        let button;
        let mut length = length;

        if length < 0 {
            button = MouseButton::ScrollLeft;
        } else {
            button = MouseButton::ScrollRight;
        }

        if length < 0 {
            length = -length;
        }

        for _ in 0..length {
            self.mouse_click(button);
        }
    }
    fn mouse_scroll_y(&mut self, length: i32) {
        let button;
        let mut length = length;

        if length < 0 {
            button = MouseButton::ScrollUp;
        } else {
            button = MouseButton::ScrollDown;
        }

        if length < 0 {
            length = -length;
        }

        for _ in 0..length {
            self.mouse_click(button);
        }
    }
}
fn keysequence<'a>(key: Key) -> Cow<'a, str> {
    if let Key::Layout(c) = key {
        return Cow::Owned(format!("U{:X}", c as u32));
    }
    if let Key::Raw(k) = key {
        return Cow::Owned(format!("{}", k as u16))
    }
    #[allow(deprecated)]
    // I mean duh, we still need to support deprecated keys until they're removed
    Cow::Borrowed(match key {
        Key::Alt => "Alt",
        Key::Backspace => "BackSpace",
        Key::CapsLock => "Caps_Lock",
        Key::Control => "Control",
        Key::Delete => "Delete",
        Key::DownArrow => "Down",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::F1 => "F1",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::Home => "Home",
        Key::Layout(_) => unreachable!(),
        Key::LeftArrow => "Left",
        Key::Option => "Option",
        Key::PageDown => "Page_Down",
        Key::PageUp => "Page_Up",
        Key::Raw(_) => unreachable!(),
        Key::Return => "Return",
        Key::RightArrow => "Right",
        Key::Shift => "Shift",
        Key::Space => "space",
        Key::Tab => "Tab",
        Key::UpArrow => "Up",

        Key::Command | Key::Super | Key::Windows | Key::Meta => "Super",
    })
}
impl KeyboardControllable for Enigo {
    fn key_sequence(&mut self, sequence: &str) {
        let string = CString::new(sequence).unwrap();
        unsafe {
            xdo_enter_text_window(
                self.xdo,
                self.window,
                string.as_ptr(),
                self.delay as useconds_t,
            );
        }
    }
    fn key_down(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window_down(
                self.xdo,
                self.window,
                string.as_ptr(),
                self.delay as useconds_t,
            );
        }
    }
    fn key_up(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window_up(
                self.xdo,
                self.window,
                string.as_ptr(),
                self.delay as useconds_t,
            );
        }
    }
    fn key_click(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window(
                self.xdo,
                self.window,
                string.as_ptr(),
                self.delay as useconds_t,
            );
        }
    }
}
