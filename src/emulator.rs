use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use log::error;

use crate::config::{IzunaConfig, VelocityConfig, VelocityModeConfig};
use crate::driver::{IzunaDriver, Key};
use crate::vector::Vector;

pub struct IzunaEmulatorState {
    action: Arc<Mutex<IzunaAction>>,
    config: IzunaConfig,
}

impl IzunaEmulatorState {
    pub fn new(config: IzunaConfig) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(IzunaEmulatorState {
            action: Arc::new(Mutex::new(IzunaAction::default())),
            config,
        }))
    }
}

/// Main loop on a whole new thread.
pub fn izuna_emulator<Driver: IzunaDriver<IzunaEmulatorState>>(
    driver: Arc<Mutex<Driver>>,
    config: IzunaConfig,
    emulator_state: Arc<Mutex<IzunaEmulatorState>>,
) -> Result<(), ()> {
    let mut driver = driver
        .lock()
        .inspect_err(|e| error!("failed to lock driver: {e:?}"))
        .map_err(|_| ())?;
    let loop_interval = Duration::from_secs_f64(1.0 / config.polling_rate as f64);
    let mut state = IzunaState::default();
    let action_guard = Arc::new(Mutex::new(IzunaAction::default()));
    driver.add_key_hook(Box::from(_action_hook));

    // these are reserved for correcting f64-i32 conversion errors
    let mut diff_cursor = Vector { x: 0.0, y: 0.0 };
    let mut diff_scroll = 0.0;

    // start the driver and perform loop
    loop {
        let time_begin = Instant::now();
        sleep(loop_interval);
        let dt = time_begin.elapsed().as_secs_f64();

        // derive effect from action
        let action = {
            action_guard
                .lock()
                .inspect_err(|e| error!("broken lock `action_guard`: {e:?}"))
                .map_err(|_| ())?
        };
        let (next_state, effect) = _next_frame(&config, state, &action, dt);
        state = next_state;

        // remember & aggregate downcast errors
        let effect = IzunaEffect {
            cursor_dx: effect.cursor_dx + diff_cursor.x,
            cursor_dy: effect.cursor_dy + diff_cursor.y,
            scroll_dy: effect.scroll_dy + diff_scroll,
        };
        let cursor_dx = effect.cursor_dx as i32;
        let cursor_dy = effect.cursor_dy as i32;
        let scroll_dy = effect.scroll_dy as i32;
        diff_cursor.x = effect.cursor_dx - cursor_dx as f64;
        diff_cursor.y = effect.cursor_dy - cursor_dy as f64;
        diff_scroll = effect.scroll_dy - scroll_dy as f64;

        // apply effect to the driver
        if cursor_dx != 0 || cursor_dy != 0 {
            driver.move_mouse_pointer(cursor_dx, cursor_dy);
        }
        if scroll_dy != 0 {
            driver.move_mouse_wheel(scroll_dy);
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
//  internal stuff

/// The state of the emulator that is modified across each frame.
#[derive(Clone, Copy, Debug)]
struct IzunaState {
    pub cursor_accel: Vector,
    pub cursor_speed: Vector,
    pub scroll_accel: f64,
    pub scroll_speed: f64,
}

impl Default for IzunaState {
    fn default() -> Self {
        IzunaState {
            cursor_accel: Vector { x: 0.0, y: 0.0 },
            cursor_speed: Vector { x: 0.0, y: 0.0 },
            scroll_accel: 0.0,
            scroll_speed: 0.0,
        }
    }
}

/// Parsed user interaction.
#[derive(Clone, Copy, Debug)]
struct IzunaAction {
    pub button_primary: bool,
    pub button_secondary: bool,
    pub button_tertiary: bool,

    // we use i8 for easier arithmetic ops
    pub cursor_powering_up: i8,
    pub cursor_powering_upper_right: i8,
    pub cursor_powering_right: i8,
    pub cursor_powering_lower_right: i8,
    pub cursor_powering_down: i8,
    pub cursor_powering_lower_left: i8,
    pub cursor_powering_left: i8,
    pub cursor_powering_upper_left: i8,
    pub scroll_powering_up: i8,
    pub scroll_powering_down: i8,

    pub sprinting: bool,
    pub sneaking: bool,
}

impl Default for IzunaAction {
    fn default() -> Self {
        IzunaAction {
            button_primary: false,
            button_secondary: false,
            button_tertiary: false,
            cursor_powering_up: 0,
            cursor_powering_upper_right: 0,
            cursor_powering_right: 0,
            cursor_powering_lower_right: 0,
            cursor_powering_down: 0,
            cursor_powering_lower_left: 0,
            cursor_powering_left: 0,
            cursor_powering_upper_left: 0,
            scroll_powering_up: 0,
            scroll_powering_down: 0,
            sprinting: false,
            sneaking: false,
        }
    }
}

/// The outcomes of action applied upon a state.
#[derive(Clone, Copy, Debug)]
struct IzunaEffect {
    pub cursor_dx: f64,
    pub cursor_dy: f64,
    pub scroll_dy: f64,
}

/// Send driver's events to the emulator.
fn _action_hook(action_guard: &mut IzunaEmulatorState, key: Key, down: bool) -> Option<()> {
    let mut action = if let Ok(guard) = action_guard.action.lock() {
        guard
    } else {
        return Some(()); // do not block hook
    };
    let config = &action_guard.config;

    macro_rules! set {
        (bool, $target:expr) => {{
            match down {
                true => $target = true,
                false => $target = false,
            };
            None
        }};
        (i8, $target:expr) => {{
            match down {
                true => $target = 1,
                false => $target = 0,
            };
            None
        }};
    }

    match key {
        // mouse buttons
        Key::Numpad0 => set!(bool, action.button_primary),
        Key::Numpad5 => set!(bool, action.button_primary),
        Key::NumpadDel => set!(bool, action.button_primary),
        Key::NumpadEnter => set!(bool, action.button_primary),
        Key::NumpadPlus => set!(bool, action.button_secondary),
        Key::NumpadSlash => set!(bool, action.button_tertiary),
        // cursor movement
        Key::Numpad8 => set!(i8, action.cursor_powering_up),
        Key::Numpad9 => set!(i8, action.cursor_powering_upper_right),
        Key::Numpad6 => set!(i8, action.cursor_powering_right),
        Key::Numpad3 => set!(i8, action.cursor_powering_lower_right),
        Key::Numpad2 => set!(i8, action.cursor_powering_down),
        Key::Numpad1 => set!(i8, action.cursor_powering_lower_left),
        Key::Numpad4 => set!(i8, action.cursor_powering_left),
        Key::Numpad7 => set!(i8, action.cursor_powering_upper_left),
        // scroll movement
        Key::NumpadAsterisk => set!(i8, action.scroll_powering_up),
        Key::NumpadHyphen => set!(i8, action.scroll_powering_down),
        // modifiers
        Key::LeftCtrl => set!(bool, action.sprinting),
        Key::LeftAlt => set!(bool, action.sneaking),
        _ => Some(()),
    }
}

/// Emulate next frame of the cursor.
fn _next_frame(
    config: &IzunaConfig,
    prev: IzunaState,
    action: &IzunaAction,
    dt: f64,
) -> (IzunaState, IzunaEffect) {
    // parse aggregated action
    let cursor_powering = action.cursor_powering_up
        + action.cursor_powering_upper_right
        + action.cursor_powering_right
        + action.cursor_powering_lower_right
        + action.cursor_powering_down
        + action.cursor_powering_lower_left
        + action.cursor_powering_left
        + action.cursor_powering_upper_left;
    let scroll_powering = action.scroll_powering_up + action.scroll_powering_down;

    // infer velocity modes
    let cursor_mode = _get_velocity_mode(
        &config.cursor_vel,
        cursor_powering > 0,
        action.sprinting,
        action.sneaking,
    );
    let scroll_mode = _get_velocity_mode(
        &config.scroll_vel,
        scroll_powering > 0,
        action.sprinting,
        action.sneaking,
    );

    // get power and derived stack modifier
    let cursor_power = (config.move_power_up * action.cursor_powering_up
        + config.move_power_upper_right * action.cursor_powering_upper_right
        + config.move_power_right * action.cursor_powering_right
        + config.move_power_lower_right * action.cursor_powering_lower_right
        + config.move_power_down * action.cursor_powering_down
        + config.move_power_lower_left * action.cursor_powering_lower_left
        + config.move_power_left * action.cursor_powering_left
        + config.move_power_upper_left * action.cursor_powering_upper_left);
    let scroll_power = config.scroll_power_up * (action.scroll_powering_up as f64)
        + config.scroll_power_down * (action.scroll_powering_down as f64);

    let cursor_power_x_stk = _get_stack_power(
        action.cursor_powering_upper_right
            + action.cursor_powering_right
            + action.cursor_powering_lower_right
            - action.cursor_powering_lower_left
            - action.cursor_powering_left
            - action.cursor_powering_upper_left,
    );
    let cursor_power_y_stk = _get_stack_power(
        action.cursor_powering_up + action.cursor_powering_upper_right
            - action.cursor_powering_lower_right
            - action.cursor_powering_down
            - action.cursor_powering_lower_left
            + action.cursor_powering_upper_left,
    );
    let cursor_power_y = _get_stack_power(action.scroll_powering_up - action.scroll_powering_down);

    // apply state changes
    let (nx_cur_x_a, nx_cur_x_v, cursor_dx) = _apply_state(
        cursor_mode,
        cursor_power.x * cursor_power_x_stk,
        cursor_powering > 0,
        prev.cursor_accel.x,
        prev.cursor_speed.x,
        dt,
    );
    let (nx_cur_y_a, nx_cur_y_v, cursor_dy) = _apply_state(
        cursor_mode,
        cursor_power.y * cursor_power_y_stk,
        cursor_powering > 0,
        prev.cursor_accel.y,
        prev.cursor_speed.y,
        dt,
    );
    let (nx_scr_a, nx_scr_v, scroll_dy) = _apply_state(
        scroll_mode,
        scroll_power * cursor_power_y,
        scroll_powering > 0,
        prev.scroll_accel,
        prev.scroll_speed,
        dt,
    );

    // combine state and effect
    let next_state = IzunaState {
        cursor_accel: Vector {
            x: nx_cur_x_a,
            y: nx_cur_y_a,
        },
        cursor_speed: Vector {
            x: nx_cur_x_v,
            y: nx_cur_y_v,
        },
        scroll_accel: nx_scr_a,
        scroll_speed: nx_scr_v,
    };
    let effect = IzunaEffect {
        cursor_dx: cursor_dx,
        cursor_dy: cursor_dy,
        scroll_dy: scroll_dy,
    };
    (next_state, effect)
}

fn _get_velocity_mode<'a>(
    config: &'a VelocityConfig,
    is_powering: bool,
    is_sprinting: bool,
    is_sneaking: bool,
) -> &'a VelocityModeConfig {
    match (is_powering, is_sprinting, is_sneaking) {
        (true, true, true) => &config.power,
        (true, true, false) => &config.sprint,
        (true, false, true) => &config.sneak,
        (true, false, false) => &config.power,
        (false, _, _) => &config.drift,
    }
}

/// If 'up', 'upper-left' and 'upper-right' are pressed at the same time, the
/// total power is not the sum of all powers, but rather the sum multipled by
/// a less-than-1 factor, to avoid the cursor moving too fast.
fn _get_stack_power(stack_cnt: i8) -> f64 {
    if stack_cnt.abs() > 0 {
        f64::ln(stack_cnt as f64 + 1.0) / (stack_cnt as f64)
    } else {
        1.0
    }
}

fn _apply_state(
    cfg: &VelocityModeConfig,
    power: f64,
    is_powering: bool,
    prev_accel: f64,
    prev_speed: f64,
    dt: f64,
) -> (f64, f64, f64) {
    let friction = if prev_speed > 0.0 {
        -cfg.friction
    } else {
        cfg.friction
    };
    let target_accel = if is_powering {
        if prev_speed.abs() >= cfg.max_speed - 1e-3 {
            0.0
        } else {
            cfg.accel * power + friction
        }
    } else {
        friction
    };

    let next_accel = if fp_eq(prev_accel, target_accel) {
        0.0
    } else if prev_accel < target_accel {
        prev_accel + cfg.jerk * dt
    } else {
        prev_accel - cfg.jerk * dt
    };
    let target_speed = if fp_eq(next_accel, 0.0) {
        prev_speed
    } else {
        prev_speed + next_accel * dt
    };
    let dx = if fp_eq(target_speed, 0.0) {
        0.0
    } else {
        target_speed * dt
    };

    (next_accel, target_speed, dx)
}

fn fp_eq(lhs: f64, rhs: f64) -> bool {
    (lhs - rhs).abs() < 1e-6
}
