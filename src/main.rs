//! Drive the M5Stack Atom Matrix 5x5 WS2812 matrix from GPIO27.
//!
//! Each pass writes 25 pixels, one RMT transfer at a time, then waits.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};
use esp_println::println;
use m5stack_atom_matrix_rust::{init_matrix_tx, send_pixel};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut channel = init_matrix_tx(peripherals.RMT, peripherals.GPIO27);

    let rng = Rng::new();

    loop {
        println!("Settings LED colors:");
        for _ in 0..25 {
            let r = rng.random() % 5;
            let g = rng.random() % 5;
            let b = rng.random() % 5;

            send_pixel(&mut channel, r as u8, g as u8, b as u8)
                .await
                .unwrap();
        }
        Timer::after(Duration::from_millis(2500)).await;
    }
}
