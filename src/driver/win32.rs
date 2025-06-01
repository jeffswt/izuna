use windows::Win32::UI::Input::KeyboardAndMouse::{
    self, GetKeyState, SendInput, INPUT, INPUT_0, MOUSEEVENTF_MOVE, MOUSEINPUT, VIRTUAL_KEY,
};

use super::{IzunaDriver, Key, MouseButton};

pub struct Win32IzunaDriver<State> {
    state: State,
    hooks: Vec<Box<dyn FnMut(&mut State, Key) -> Option<Key>>>,
}

impl<State> IzunaDriver<State> for Win32IzunaDriver<State> {
    fn create(state: State) -> Self {
        Win32IzunaDriver {
            state: state,
            hooks: vec![],
        }
    }

    fn add_key_hook(&mut self, hook: Box<dyn FnMut(&mut State, Key) -> Option<Key>>) -> () {
        self.hooks.push(hook);
    }

    /// Get current key state. The first return value shows whether the key is
    /// currently pressed down, and the second indicates whether the 'toggle'
    /// (e.g. Caps Lock) indicator light is on.
    fn get_key_state(&self, key: Key) -> (bool, bool) {
        let vk_value = _key_to_win32_vk(key);
        let state = unsafe { GetKeyState(vk_value.0.into()) } as u16;
        // If the high-order bit is 1, the key is down; otherwise, it is up.
        let is_down = state & 0x8000 != 0;
        // If the low-order bit is 1, the key is toggled. A key, such as the
        // CAPS LOCK key, is toggled if it is turned on. The key is off and
        // un-toggled if the low-order bit is 0. A toggle key's indicator light
        // (if any) on the keyboard will be on when the key is toggled, and off
        // when the key is untoggled.
        let is_toggled = state & 0x0001 != 0;
        (is_down, is_toggled)
    }

    /// Set the mouse button state.
    fn set_mouse_button(&self, button: MouseButton, down: bool) -> () {
        _send_mouse_event(
            "set_mouse_button",
            MOUSEINPUT {
                dwFlags: match (button, down) {
                    (MouseButton::Left, true) => KeyboardAndMouse::MOUSEEVENTF_LEFTDOWN,
                    (MouseButton::Left, false) => KeyboardAndMouse::MOUSEEVENTF_LEFTUP,
                    (MouseButton::Middle, true) => KeyboardAndMouse::MOUSEEVENTF_MIDDLEDOWN,
                    (MouseButton::Middle, false) => KeyboardAndMouse::MOUSEEVENTF_MIDDLEUP,
                    (MouseButton::Right, true) => KeyboardAndMouse::MOUSEEVENTF_RIGHTDOWN,
                    (MouseButton::Right, false) => KeyboardAndMouse::MOUSEEVENTF_RIGHTUP,
                },
                ..Default::default()
            },
        );
    }

    /// Move the mouse by the given delta x and y values.
    fn move_mouse_pointer(&self, dx: i32, dy: i32) -> () {
        _send_mouse_event(
            "move_mouse_pointer",
            MOUSEINPUT {
                dwFlags: KeyboardAndMouse::MOUSEEVENTF_MOVE,
                // Absolute data is specified as the x coordinate of the mouse;
                // relative data is specified as the number of pixels moved.
                dx: dx,
                dy: dy,
                ..Default::default()
            },
        );
    }

    /// Move the scroll position by the given delta x and y values.
    fn move_mouse_wheel(&self, dx: i32) -> () {
        _send_mouse_event(
            "move_mouse_wheel",
            MOUSEINPUT {
                // If dwFlags contains MOUSEEVENTF_WHEEL, then mouseData
                // specifies the amount of wheel movement. A positive value
                // indicates that the wheel was rotated forward, away from the
                // user; a negative value indicates that the wheel was rotated
                // backward, toward the user. One wheel click is defined as
                // WHEEL_DELTA, which is 120.
                dwFlags: KeyboardAndMouse::MOUSEEVENTF_WHEEL,
                mouseData: dx as u32,
                ..Default::default()
            },
        );
    }
}

fn _key_to_win32_vk(key: Key) -> VIRTUAL_KEY {
    match key {
        Key::LeftCtrl => KeyboardAndMouse::VK_LCONTROL,
        Key::LeftAlt => KeyboardAndMouse::VK_LMENU,
        Key::LeftShift => KeyboardAndMouse::VK_LSHIFT,
        Key::RightCtrl => KeyboardAndMouse::VK_RCONTROL,
        Key::RightAlt => KeyboardAndMouse::VK_RMENU,
        Key::RightShift => KeyboardAndMouse::VK_RSHIFT,
        Key::Up => KeyboardAndMouse::VK_UP,
        Key::Down => KeyboardAndMouse::VK_DOWN,
        Key::Left => KeyboardAndMouse::VK_LEFT,
        Key::Right => KeyboardAndMouse::VK_RIGHT,
        Key::NumLock => KeyboardAndMouse::VK_NUMLOCK,
        Key::Numpad0 => KeyboardAndMouse::VK_NUMPAD0,
        Key::Numpad1 => KeyboardAndMouse::VK_NUMPAD1,
        Key::Numpad2 => KeyboardAndMouse::VK_NUMPAD2,
        Key::Numpad3 => KeyboardAndMouse::VK_NUMPAD3,
        Key::Numpad4 => KeyboardAndMouse::VK_NUMPAD4,
        Key::Numpad5 => KeyboardAndMouse::VK_NUMPAD5,
        Key::Numpad6 => KeyboardAndMouse::VK_NUMPAD6,
        Key::Numpad7 => KeyboardAndMouse::VK_NUMPAD7,
        Key::Numpad8 => KeyboardAndMouse::VK_NUMPAD8,
        Key::Numpad9 => KeyboardAndMouse::VK_NUMPAD9,
        Key::NumpadEnter => KeyboardAndMouse::VK_RETURN,
        Key::NumpadDel => KeyboardAndMouse::VK_DELETE,
        Key::NumpadPlus => KeyboardAndMouse::VK_ADD,
        Key::NumpadHyphen => KeyboardAndMouse::VK_SUBTRACT,
        Key::NumpadAsterisk => KeyboardAndMouse::VK_MULTIPLY,
        Key::NumpadSlash => KeyboardAndMouse::VK_DIVIDE,
    }
}

fn _key_from_win32_vk(vk: VIRTUAL_KEY) -> Option<Key> {
    match vk {
        KeyboardAndMouse::VK_LCONTROL => Some(Key::LeftCtrl),
        KeyboardAndMouse::VK_LMENU => Some(Key::LeftAlt),
        KeyboardAndMouse::VK_LSHIFT => Some(Key::LeftShift),
        KeyboardAndMouse::VK_RCONTROL => Some(Key::RightCtrl),
        KeyboardAndMouse::VK_RMENU => Some(Key::RightAlt),
        KeyboardAndMouse::VK_RSHIFT => Some(Key::RightShift),
        KeyboardAndMouse::VK_UP => Some(Key::Up),
        KeyboardAndMouse::VK_DOWN => Some(Key::Down),
        KeyboardAndMouse::VK_LEFT => Some(Key::Left),
        KeyboardAndMouse::VK_RIGHT => Some(Key::Right),
        KeyboardAndMouse::VK_NUMLOCK => Some(Key::NumLock),
        KeyboardAndMouse::VK_NUMPAD0 => Some(Key::Numpad0),
        KeyboardAndMouse::VK_NUMPAD1 => Some(Key::Numpad1),
        KeyboardAndMouse::VK_NUMPAD2 => Some(Key::Numpad2),
        KeyboardAndMouse::VK_NUMPAD3 => Some(Key::Numpad3),
        KeyboardAndMouse::VK_NUMPAD4 => Some(Key::Numpad4),
        KeyboardAndMouse::VK_NUMPAD5 => Some(Key::Numpad5),
        KeyboardAndMouse::VK_NUMPAD6 => Some(Key::Numpad6),
        KeyboardAndMouse::VK_NUMPAD7 => Some(Key::Numpad7),
        KeyboardAndMouse::VK_NUMPAD8 => Some(Key::Numpad8),
        KeyboardAndMouse::VK_NUMPAD9 => Some(Key::Numpad9),
        KeyboardAndMouse::VK_RETURN => Some(Key::NumpadEnter),
        KeyboardAndMouse::VK_DELETE => Some(Key::NumpadDel),
        KeyboardAndMouse::VK_ADD => Some(Key::NumpadPlus),
        KeyboardAndMouse::VK_SUBTRACT => Some(Key::NumpadHyphen),
        KeyboardAndMouse::VK_MULTIPLY => Some(Key::NumpadAsterisk),
        KeyboardAndMouse::VK_DIVIDE => Some(Key::NumpadSlash),
        _ => None,
    }
}

fn _send_mouse_event(func: &str, event: MOUSEINPUT) -> () {
    let events = [INPUT {
        r#type: KeyboardAndMouse::INPUT_MOUSE,
        Anonymous: INPUT_0 { mi: event },
    }];
    let sent = unsafe { SendInput(&events, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != events.len() {
        eprintln!("{}: expected {} events, got {}", func, events.len(), sent);
    }
}
