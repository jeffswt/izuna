pub mod win32;

pub enum Key {
    // modifiers
    LeftCtrl,
    LeftAlt,
    LeftShift,
    RightCtrl,
    RightAlt,
    RightShift,
    // direction keys
    Up,
    Down,
    Left,
    Right,
    // numpad
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadEnter,
    NumpadDel,
    NumpadPlus,
    NumpadHyphen,
    NumpadAsterisk,
    NumpadSlash,
}

pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// The Izuna driver is implemented on a by-platform basis, allowing for
/// separate integrations with different OSes.
pub trait IzunaDriver<State> {
    /// Initializes the driver with the given state.
    fn create(state: State) -> Self;

    /// Listen to key updates and updates the state accordingly. If a hook
    /// existed previously, the new hook will be called prior the the previous
    /// ones.
    ///
    /// The hook should return a key code if it should be propagated to the
    /// next layer, or `None` if it is handled and consumed.
    fn add_key_hook(&mut self, hook: Box<dyn FnMut(&mut State, Key) -> Option<Key>>) -> ();

    /// Get current key state. The first return value shows whether the key is
    /// currently pressed down, and the second indicates whether the 'toggle'
    /// (e.g. Caps Lock) indicator light is on.
    fn get_key_state(&self, key: Key) -> (bool, bool);

    /// Set the mouse button state.
    fn set_mouse_button(&self, button: MouseButton, down: bool) -> ();

    /// Move the mouse by the given delta x and y values.
    fn move_mouse_pointer(&self, dx: i32, dy: i32) -> ();

    /// Move the scroll position by the given delta x value.
    fn move_mouse_wheel(&self, dx: i32) -> ();
}
