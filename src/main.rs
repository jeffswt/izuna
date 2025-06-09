#![allow(unused)]
#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::thread::spawn;

use config::{default_izuna_config, IzunaConfig};
use driver::{win32::Win32IzunaDriver, IzunaDriver};
use emulator::{izuna_emulator, IzunaEmulatorState};

mod config;
mod driver;
mod emulator;
mod vector;

fn main() {
    let config = default_izuna_config();
    let mut emulator_state = IzunaEmulatorState::new(config.clone());
    let mut driver = Arc::new(Win32IzunaDriver::create(emulator_state.clone()));

    spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(30000));
        std::process::exit(1);
    });

    let driver_clone = driver.clone();
    spawn(move || {
        izuna_emulator(driver_clone.as_ref(), config, emulator_state);
    });

    driver.run_message_loop();
    drop(driver);
}
