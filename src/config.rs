use super::driver::MouseButton;
use super::vector::Vector;

/// Configures Izuna behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct IzunaConfig {
    // performance settings
    pub polling_rate: u64, // frames per second

    // mouse button mappings
    pub primary_click: MouseButton,
    pub secondary_click: MouseButton,
    pub tertiary_click: MouseButton,

    // generic magic values
    pub cursor_vel: VelocityConfig,
    pub scroll_vel: VelocityConfig,

    // movement speeds
    pub move_power_up: Vector,
    pub move_power_upper_right: Vector,
    pub move_power_right: Vector,
    pub move_power_lower_right: Vector,
    pub move_power_down: Vector,
    pub move_power_lower_left: Vector,
    pub move_power_left: Vector,
    pub move_power_upper_left: Vector,
    pub scroll_power_up: f64,
    pub scroll_power_down: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityConfig {
    pub generic_scale: f64,
    pub drift: VelocityModeConfig,  // not powering at all
    pub power: VelocityModeConfig,  // normal power
    pub sprint: VelocityModeConfig, // fast power
    pub sneak: VelocityModeConfig,  // little power
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityModeConfig {
    // responsiveness is paramount so we do not need jerk
    pub accel: f64, // m/s^2, max accel, mul by 'power', reached by aggregating jerk
    pub max_speed: f64, // m/s, max speed reached by aggregating accel
    pub brake: f64, // m/s^2, how fast the speed decays when not accelerating,
}

pub fn default_izuna_config() -> IzunaConfig {
    IzunaConfig {
        polling_rate: 360, // 360 fps

        primary_click: MouseButton::Left,
        secondary_click: MouseButton::Right,
        tertiary_click: MouseButton::Middle,

        cursor_vel: VelocityConfig {
            generic_scale: 1.0,
            drift: VelocityModeConfig {
                accel: 0.0,
                max_speed: 0.0,
                brake: 7200.0,
            },
            power: VelocityModeConfig {
                accel: 11200.0, // 4000
                max_speed: 2400.0,
                brake: 7200.0,
            },
            sprint: VelocityModeConfig {
                accel: 16000.0, // 8800
                max_speed: 4800.0,
                brake: 7200.0,
            },
            sneak: VelocityModeConfig {
                accel: 34000.0, // 2000
                max_speed: 1600.0,
                brake: 32000.0,
            },
        },
        scroll_vel: VelocityConfig {
            generic_scale: 1.0,
            drift: VelocityModeConfig {
                accel: 0.0,
                max_speed: 0.0,
                brake: 7200.0,
            },
            power: VelocityModeConfig {
                accel: 14400.0, // 7200
                max_speed: 3600.0,
                brake: 7200.0,
            },
            sprint: VelocityModeConfig {
                accel: 16800.0, // 9600
                max_speed: 5400.0,
                brake: 7200.0,
            },
            sneak: VelocityModeConfig {
                accel: 27600.0, // 3600
                max_speed: 1800.0,
                brake: 24000.0,
            },
        },

        move_power_up: Vector { x: 0.0, y: 1.0 },
        move_power_upper_right: Vector {
            x: 0.7071,
            y: 0.7071,
        },
        move_power_right: Vector { x: 1.0, y: 0.0 },
        move_power_lower_right: Vector {
            x: 0.7071,
            y: -0.7071,
        },
        move_power_down: Vector { x: 0.0, y: -1.0 },
        move_power_lower_left: Vector {
            x: -0.7071,
            y: -0.7071,
        },
        move_power_left: Vector { x: -1.0, y: 0.0 },
        move_power_upper_left: Vector {
            x: -0.7071,
            y: 0.7071,
        },
        scroll_power_up: 1.0,
        scroll_power_down: -1.0,
    }
}
