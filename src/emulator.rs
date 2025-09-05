use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use log::error;

use crate::config::{IzunaConfig, VelocityConfig, VelocityModeConfig};
use crate::driver::{IzunaDriver, IzunaKeyHook, Key, MouseButton};
use crate::vector::Vector;

pub struct IzunaEmulatorState {
    action: IzunaAction,
    config: IzunaConfig,
}

impl IzunaEmulatorState {
    pub fn new(config: IzunaConfig) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(IzunaEmulatorState {
            action: IzunaAction::default(),
            config,
        }))
    }
}

/// Main loop on a whole new thread.
pub fn izuna_emulator<Driver: 'static + Send + Sync + IzunaDriver<IzunaEmulatorState>>(
    driver: Arc<Driver>,
    config: IzunaConfig,
    emulator_state: Arc<Mutex<IzunaEmulatorState>>,
) -> Result<(), ()> {
    let loop_interval = Duration::from_secs_f64(1.0 / config.polling_rate as f64);
    let mut state = IzunaState::default();
    driver.clone().add_key_hook(Box::from(ActionHook {
        _driver: PhantomData,
    }));

    // these are reserved for correcting f64-i32 conversion errors
    let mut diff_cursor = Vector { x: 0.0, y: 0.0 };
    let mut diff_scroll = 0.0;
    // and these for remembering states
    let mut diff_did_mb_primary_down = false;
    let mut diff_did_mb_secondary_down = false;
    let mut diff_did_mb_tertiary_down = false;

    // start the driver and perform loop
    let mut timestamp = Instant::now();
    loop {
        sleep(loop_interval);
        let new_timestamp = Instant::now();
        let dt = (new_timestamp - timestamp).as_secs_f64();
        timestamp = new_timestamp;

        // derive effect from action
        let (next_state, effect) = {
            let em_state = emulator_state
                .lock()
                .inspect_err(|e| error!("broken lock `action_guard`: {e:?}"))
                .map_err(|_| ())?;
            _next_frame(&config, state, &em_state.action, dt)
        };
        state = next_state;

        // remember & aggregate downcast errors
        let effect = IzunaEffect {
            mb_primary_down: effect.mb_primary_down,
            mb_secondary_down: effect.mb_secondary_down,
            mb_tertiary_down: effect.mb_tertiary_down,
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
        // apply mouse buttons
        fn apply_mb<Driver: IzunaDriver<IzunaEmulatorState>>(
            driver: &Driver,
            effect_down: bool,
            diff_did: &mut bool,
            cfg: &MouseButton,
        ) {
            if effect_down && !*diff_did {
                driver.set_mouse_button(*cfg, true);
                *diff_did = true;
            } else if !effect_down && *diff_did {
                driver.set_mouse_button(*cfg, false);
                *diff_did = false;
            }
        }
        apply_mb(
            driver.as_ref(),
            effect.mb_primary_down,
            &mut diff_did_mb_primary_down,
            &config.primary_click,
        );
        apply_mb(
            driver.as_ref(),
            effect.mb_secondary_down,
            &mut diff_did_mb_secondary_down,
            &config.secondary_click,
        );
        apply_mb(
            driver.as_ref(),
            effect.mb_tertiary_down,
            &mut diff_did_mb_tertiary_down,
            &config.tertiary_click,
        );
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
//  internal stuff

/// The state of the emulator that is modified across each frame.
#[derive(Debug)]
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
#[derive(Debug)]
struct IzunaAction {
    pub action_enabled: Option<bool>,

    pub button_primary_1: bool,
    pub button_primary_2: bool,
    pub button_primary_3: bool,
    pub button_primary_4: bool,
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
            action_enabled: None,
            button_primary_1: false,
            button_primary_2: false,
            button_primary_3: false,
            button_primary_4: false,
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
#[derive(Debug)]
struct IzunaEffect {
    pub mb_primary_down: bool,
    pub mb_secondary_down: bool,
    pub mb_tertiary_down: bool,
    pub cursor_dx: f64,
    pub cursor_dy: f64,
    pub scroll_dy: f64,
}

/// Send driver's events to the emulator.
struct ActionHook<Driver: IzunaDriver<IzunaEmulatorState>> {
    _driver: PhantomData<Driver>,
}

impl<Driver: IzunaDriver<IzunaEmulatorState>> IzunaKeyHook<IzunaEmulatorState, Driver>
    for ActionHook<Driver>
{
    fn call(
        &self,
        driver: &Driver,
        state: &mut IzunaEmulatorState,
        key: Key,
        down: bool,
    ) -> Option<()> {
        let action = &mut state.action;
        let config = &state.config;
        let action_enabled = match action.action_enabled {
            Some(v) => v,
            None => !driver.get_key_state(Key::NumLock).1,
        };
        action.action_enabled = Some(action_enabled);

        macro_rules! set {
            (bool, $target:expr, $ret:expr) => {{
                if !action_enabled {
                    return Some(());
                }
                match down {
                    true => $target = true,
                    false => $target = false,
                };
                $ret
            }};
            (i8, $target:expr, $ret:expr) => {{
                if !action_enabled {
                    return Some(());
                }
                match down {
                    true => $target = 1,
                    false => $target = 0,
                };
                $ret
            }};
        }

        match key {
            // toggle izuna
            Key::NumLock => {
                action.action_enabled = Some(!driver.get_key_state(Key::NumLock).1);
                Some(())
            }
            // mouse buttons
            Key::Numpad5 => set!(bool, action.button_primary_1, None),
            Key::NavpadClear => set!(bool, action.button_primary_1, None),
            Key::NumpadEnter => set!(bool, action.button_primary_2, None),
            Key::NumpadDel => set!(bool, action.button_primary_3, None),
            Key::NavpadDelete => set!(bool, action.button_primary_3, None),
            Key::Numpad0 => set!(bool, action.button_primary_4, None),
            Key::NumpadPlus => set!(bool, action.button_secondary, None),
            Key::NumpadSlash => set!(bool, action.button_tertiary, None),
            // cursor movement (numpad ver)
            Key::Numpad8 => set!(i8, action.cursor_powering_up, None),
            Key::Numpad9 => set!(i8, action.cursor_powering_upper_right, None),
            Key::Numpad6 => set!(i8, action.cursor_powering_right, None),
            Key::Numpad3 => set!(i8, action.cursor_powering_lower_right, None),
            Key::Numpad2 => set!(i8, action.cursor_powering_down, None),
            Key::Numpad1 => set!(i8, action.cursor_powering_lower_left, None),
            Key::Numpad4 => set!(i8, action.cursor_powering_left, None),
            Key::Numpad7 => set!(i8, action.cursor_powering_upper_left, None),
            // cursor movement (navpad ver)
            Key::NavpadUp => set!(i8, action.cursor_powering_up, None),
            Key::NavpadPrior => set!(i8, action.cursor_powering_upper_right, None),
            Key::NavpadRight => set!(i8, action.cursor_powering_right, None),
            Key::NavpadNext => set!(i8, action.cursor_powering_lower_right, None),
            Key::NavpadDown => set!(i8, action.cursor_powering_down, None),
            Key::NavpadEnd => set!(i8, action.cursor_powering_lower_left, None),
            Key::NavpadLeft => set!(i8, action.cursor_powering_left, None),
            Key::NavpadHome => set!(i8, action.cursor_powering_upper_left, None),
            // scroll movement
            Key::NumpadAsterisk => set!(i8, action.scroll_powering_up, None),
            Key::NumpadHyphen => set!(i8, action.scroll_powering_down, None),
            // modifiers
            Key::LeftShift => set!(bool, action.sprinting, Some(())),
            Key::LeftCtrl => set!(bool, action.sprinting, Some(())),
            Key::LeftAlt => set!(bool, action.sneaking, Some(())),
            _ => Some(()),
        }
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
    let cursor_powering_x = (action.cursor_powering_upper_right
        + action.cursor_powering_right
        + action.cursor_powering_lower_right
        - action.cursor_powering_lower_left
        - action.cursor_powering_left
        - action.cursor_powering_upper_left)
        .abs();
    let cursor_powering_y = (action.cursor_powering_up + action.cursor_powering_upper_right
        - action.cursor_powering_lower_right
        - action.cursor_powering_down
        - action.cursor_powering_lower_left
        + action.cursor_powering_upper_left)
        .abs();
    let scroll_powering = action.scroll_powering_up + action.scroll_powering_down;

    // infer velocity modes
    let cursor_mode = _get_velocity_mode(
        &config.cursor_vel,
        cursor_powering_x > 0 || cursor_powering_y > 0,
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
        + config.move_power_upper_left * action.cursor_powering_upper_left)
        .norm();
    let scroll_power = config.scroll_power_up * (action.scroll_powering_up as f64)
        + config.scroll_power_down * (action.scroll_powering_down as f64);

    // apply state changes
    let (nx_cur_a, nx_cur_v, cursor_d) = _apply_cursor_state(
        config.cursor_vel.generic_scale,
        cursor_mode,
        cursor_power,
        _get_stack_power(cursor_powering_x),
        _get_stack_power(cursor_powering_y),
        prev.cursor_accel,
        prev.cursor_speed,
        dt,
    );
    let (nx_scr_a, nx_scr_v, scroll_dy) = _apply_scroll_state(
        config.scroll_vel.generic_scale,
        scroll_mode,
        scroll_power,
        _get_stack_power(scroll_powering),
        prev.scroll_accel,
        prev.scroll_speed,
        dt,
    );

    // combine state and effect
    let next_state = IzunaState {
        cursor_accel: nx_cur_a,
        cursor_speed: nx_cur_v,
        scroll_accel: nx_scr_a,
        scroll_speed: nx_scr_v,
    };
    let effect = IzunaEffect {
        mb_primary_down: action.button_primary_1
            || action.button_primary_2
            || action.button_primary_3
            || action.button_primary_4,
        mb_secondary_down: action.button_secondary,
        mb_tertiary_down: action.button_tertiary,
        cursor_dx: cursor_d.x,
        cursor_dy: cursor_d.y,
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
        (false, _, true) => &config.sneak,
        (false, _, false) => &config.drift,
    }
}

/// If 'up', 'upper-left' and 'upper-right' are pressed at the same time, the
/// total power is not the sum of all powers, but rather the sum multipled by
/// a less-than-1 factor, to avoid the cursor moving too fast.
fn _get_stack_power(stack_cnt: i8) -> f64 {
    match stack_cnt.abs() {
        0 | 1 => 1.0,
        2 => 1.2,  // 1.2x
        3 => 1.35, // 1.35x
        _ => 1.4,
    }
}

fn _apply_cursor_state(
    generic_scale: f64,
    cfg: &VelocityModeConfig,
    power: Vector,
    stack_power_x: f64,
    stack_power_y: f64,
    prev_accel: Vector,
    prev_speed: Vector,
    dt: f64,
) -> (Vector, Vector, Vector) {
    // now add power
    let accel = Vector {
        x: power.x * stack_power_x * cfg.accel,
        y: power.y * stack_power_y * cfg.accel,
    };
    // adjust speed
    let speed = prev_speed + accel * dt;
    let (speed, mut speed_len) = (speed.norm(), speed.length());
    // do not accelerate if over max speed
    if speed_len > cfg.max_speed {
        speed_len = prev_speed.length();
    }
    // apply friction
    speed_len = (speed_len - cfg.brake * dt).max(0.0);
    let speed = speed * speed_len;
    // adjust position
    let d_pos = speed * generic_scale * dt;
    (accel, speed, d_pos)
}

fn _apply_scroll_state(
    generic_scale: f64,
    cfg: &VelocityModeConfig,
    power: f64,
    stack_power: f64,
    prev_accel: f64,
    prev_speed: f64,
    dt: f64,
) -> (f64, f64, f64) {
    // now add power
    let accel = power * stack_power * cfg.accel;
    // adjust speed
    let speed = prev_speed + accel * dt;
    let (speed, mut speed_len) = (speed >= 0.0, speed.abs());
    // do not accelerate if over max speed
    if speed_len > cfg.max_speed {
        speed_len = prev_speed.abs();
    }
    // apply friction
    speed_len = (speed_len - cfg.brake * dt).max(0.0);
    let speed = if speed { speed_len } else { -speed_len };
    // adjust position
    let d_pos = speed * generic_scale * dt;
    (accel, speed, d_pos)
}

fn fp_eq(lhs: f64, rhs: f64) -> bool {
    (lhs - rhs).abs() < 1e-6
}
