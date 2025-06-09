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
    pub drift: VelocityModeConfig,  // not powering at all
    pub power: VelocityModeConfig,  // normal power
    pub sprint: VelocityModeConfig, // fast power
    pub sneak: VelocityModeConfig,  // little power
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityModeConfig {
    // responsiveness is paramount so we do not need jerk
    pub accel: f64,     // max accel, mul by 'power', reached by aggregating jerk
    pub max_speed: f64, // max speed reached by aggregating accel
    pub brake: f64,     // how fast the speed decays when not accelerating,
}

pub fn default_izuna_config() -> IzunaConfig {
    IzunaConfig {
        polling_rate: 1000, // 1000 FPS

        primary_click: MouseButton::Left,
        secondary_click: MouseButton::Right,
        tertiary_click: MouseButton::Middle,

        cursor_vel: VelocityConfig {
            drift: VelocityModeConfig {
                accel: 0.0,
                max_speed: 0.0,
                brake: 8000.0,
            },
            power: VelocityModeConfig {
                accel: 11200.0,
                max_speed: 3000.0,
                brake: 6600.0,
            },
            sprint: VelocityModeConfig {
                accel: 16000.0,
                max_speed: 4500.0,
                brake: 6600.0,
            },
            sneak: VelocityModeConfig {
                accel: 10000.0,
                max_speed: 2400.0,
                brake: 7200.0,
            },
        },
        scroll_vel: VelocityConfig {
            drift: VelocityModeConfig {
                accel: 0.0,
                max_speed: 0.0,
                brake: 20.0,
            },
            power: VelocityModeConfig {
                accel: 30.0,
                max_speed: 100.0,
                brake: 20.0,
            },
            sprint: VelocityModeConfig {
                accel: 50.0,
                max_speed: 160.0,
                brake: 20.0,
            },
            sneak: VelocityModeConfig {
                accel: 15.0,
                max_speed: 45.0,
                brake: 20.0,
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
