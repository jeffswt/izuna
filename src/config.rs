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
    pub jerk: f64,      // d of accel
    pub accel: f64,     // max accel, mul by 'power', reached by aggregating jerk
    pub max_speed: f64, // max speed reached by aggregating accel
    pub friction: f64,  // how fast the speed decays when not accelerating,
}

pub fn default_izuna_config() -> IzunaConfig {
    IzunaConfig {
        polling_rate: 60, // 60 FPS

        primary_click: MouseButton::Left,
        secondary_click: MouseButton::Right,
        tertiary_click: MouseButton::Middle,

        cursor_vel: VelocityConfig {
            drift: VelocityModeConfig {
                jerk: 15.0,
                accel: 0.0,
                max_speed: 0.0,
                friction: 20.0,
            },
            power: VelocityModeConfig {
                jerk: 15.0,
                accel: 5.0,
                max_speed: 100.0,
                friction: 20.0,
            },
            sprint: VelocityModeConfig {
                jerk: 15.0,
                accel: 8.0,
                max_speed: 160.0,
                friction: 20.0,
            },
            sneak: VelocityModeConfig {
                jerk: 15.0,
                accel: 3.0,
                max_speed: 45.0,
                friction: 20.0,
            },
        },
        scroll_vel: VelocityConfig {
            drift: VelocityModeConfig {
                jerk: 15.0,
                accel: 0.0,
                max_speed: 0.0,
                friction: 20.0,
            },
            power: VelocityModeConfig {
                jerk: 15.0,
                accel: 5.0,
                max_speed: 100.0,
                friction: 20.0,
            },
            sprint: VelocityModeConfig {
                jerk: 15.0,
                accel: 8.0,
                max_speed: 160.0,
                friction: 20.0,
            },
            sneak: VelocityModeConfig {
                jerk: 15.0,
                accel: 3.0,
                max_speed: 45.0,
                friction: 20.0,
            },
        },

        move_power_up: Vector { x: 1.0, y: 0.0 },
        move_power_upper_right: Vector {
            x: 0.7071,
            y: 0.7071,
        },
        move_power_right: Vector { x: 0.0, y: 1.0 },
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
