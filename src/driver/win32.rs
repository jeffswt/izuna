use std::mem;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use windows::core::{s, w};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::{
    self, GetModuleHandleExW, GetModuleHandleW, LoadLibraryExW, LoadLibraryW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    self, GetKeyState, SendInput, INPUT, INPUT_0, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    self, CallNextHookEx, SetWindowsHookExW, HHOOK, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::{IzunaDriver, Key, MouseButton};

pub struct Win32IzunaDriver<State> {
    state: Arc<Mutex<State>>,
}

static GLOBAL_HOOKS: Mutex<Vec<Box<dyn Send + Sync + Fn(Key, bool) -> Option<()>>>> =
    Mutex::new(Vec::new());
static GLOBAL_HOOKED: Mutex<Option<usize>> = Mutex::new(None);

impl<State: 'static + Send + Sync> IzunaDriver<State> for Win32IzunaDriver<State> {
    fn create(state: Arc<Mutex<State>>) -> Self {
        {
            let mut guard = GLOBAL_HOOKED.lock().unwrap();
            if guard.is_some() {
                panic!("[win32] only 1 instance of Win32IzunaDriver can be created at a time");
            }
            let h_mod = unsafe { GetModuleHandleW(None).unwrap() };
            let h_hook = unsafe {
                SetWindowsHookExW(
                    WindowsAndMessaging::WH_KEYBOARD_LL,
                    Some(_hook_func),
                    Some(HINSTANCE(h_mod.0)),
                    0, // dwThreadId: thread ID for the hook
                )
                .unwrap()
            };
            *guard = Some(h_hook.0 as usize);
        }
        Win32IzunaDriver { state: state }
    }

    fn run_message_loop(&self) -> () {
        loop {
            let mut msg = WindowsAndMessaging::MSG::default();
            let msg = unsafe { WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0) };
            if msg.0 != 0 {
                eprintln!("[win32] GetMessageW failed: {:?}", msg);
                break;
            }
        }
    }

    fn add_key_hook(
        &mut self,
        mut hook: Box<dyn Send + Sync + Fn(&mut State, Key, bool) -> Option<()>>,
    ) -> () {
        let state_clone = self.state.clone();
        let wrapped = move |key, is_down| {
            if let Ok(mut state) = state_clone.lock() {
                hook(&mut state, key, is_down)
            } else {
                eprintln!("hook: failed to lock state");
                None
            }
        };
        if let Ok(mut hooks) = GLOBAL_HOOKS.lock() {
            hooks.push(Box::new(wrapped));
        }
    }

    fn get_key_state(&self, key: Key) -> (bool, bool) {
        let vk_value = _key_to_win32_vk(key);
        let state = unsafe { GetKeyState(vk_value.0.into()) } as u16;
        // If the high-order bit is 1, the key is down; otherwise, it is up.
        let is_down = state & 0x8000 != 0;
        // If the low-order bit is 1, the key is toggled. A key, such as the
        // CAPS LOCK key, is toggled if it is turned on. The key is off and
        // un-toggled if the low-order bit is 0. A toggle key's indicator light
        // (if any) on the keyboard will be on when the key is toggled, and off
        // when the key is un-toggled.
        let is_toggled = state & 0x0001 != 0;
        (is_down, is_toggled)
    }

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

    fn move_mouse_pointer(&self, dx: i32, dy: i32) -> () {
        println!("move {dx} {dy}");
        // _send_mouse_event(
        //     "move_mouse_pointer",
        //     MOUSEINPUT {
        //         dwFlags: KeyboardAndMouse::MOUSEEVENTF_MOVE,
        //         // Absolute data is specified as the x coordinate of the mouse;
        //         // relative data is specified as the number of pixels moved.
        //         dx: dx,
        //         dy: dy,
        //         ..Default::default()
        //     },
        // );
    }

    fn move_mouse_wheel(&self, dy: i32) -> () {
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
                mouseData: dy as u32,
                ..Default::default()
            },
        );
    }
}

impl<State> Drop for Win32IzunaDriver<State> {
    fn drop(&mut self) {
        if let Some(h_hook) = GLOBAL_HOOKED.lock().unwrap().take() {
            eprintln!("[win32] removing global key hook {h_hook}");
            let h_hook = HHOOK(h_hook as _);
            unsafe { WindowsAndMessaging::UnhookWindowsHookEx(h_hook).unwrap() }
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
unsafe extern "system" fn _hook_func(nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    // convert to key value
    let key_down = match wParam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => Some(true),
        WM_KEYUP | WM_SYSKEYUP => Some(false),
        _ => None,
    };
    let kb_dll_hook = lParam.0 as *const KBDLLHOOKSTRUCT;
    let key = kb_dll_hook
        .as_ref()
        .map(|hook| hook.vkCode)
        .map(|vk| VIRTUAL_KEY(vk as u16))
        .map(_key_from_win32_vk)
        .unwrap_or(None);
    let forward = if let (Some(key_down), Some(key)) = (key_down, key) {
        let mut key = Some(key);
        // call every hook in hooks registered
        if let Ok(global_hooks) = GLOBAL_HOOKS.lock() {
            for hook in global_hooks.iter() {
                if let Some(k) = key {
                    key = hook(k, key_down).map(|_| k);
                }
            }
        }
        key.is_some()
    } else {
        true
    };

    // println!("hooking {key_down:?} {key:?} {nCode} {forward}");
    // return CallNextHookEx(None, nCode, wParam, lParam);

    if nCode < 0 || forward {
        let h_hook = GLOBAL_HOOKED
            .lock()
            .inspect_err(|e| eprintln!("hok err {e:?}"))
            .unwrap()
            .as_ref()
            .map(|h| HHOOK(*h as _));
        // If nCode is less than zero, the hook procedure must return the value
        // returned by CallNextHookEx function.
        // If nCode is greater than or equal to zero, it is highly recommended
        // that you call CallNextHookEx function and return the value it
        // returns; otherwise, other applications that have installed
        // WH_CALLWNDPROCRET hooks will not receive hook notifications and may
        // behave incorrectly as a result. If the hook procedure does not call
        // CallNextHookEx, the return value should be zero.
        CallNextHookEx(None, nCode, wParam, lParam)
    } else {
        LRESULT(0)
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
