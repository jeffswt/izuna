#![allow(unused)]
#![allow(dead_code)]

use std::thread::spawn;

use driver::IzunaDriver;

mod config;
mod driver;
mod vector;

fn main() {
    println!("Hello, world!");
    let mut driver = driver::win32::Win32IzunaDriver::create(());
    driver.add_key_hook(Box::new(|_, k, down| {
        println!("{k:?} {down}");
        Some(())
    }));

    spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10000));
        std::process::exit(1);
    });
    driver.run_message_loop();
    drop(driver);
}
