//! Solid-color demo for the M5Stack Atom Matrix.
//!
//! Lights all 25 LEDs in a single color; each press of the user button
//! (GPIO39, active-low with an internal pull-up) picks a new random color.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use m5stack_atom_matrix_rust::{fill_matrix, init_matrix_tx};

esp_bootloader_esp_idf::esp_app_desc!();

// Keep the brightness in the same low range as the other demos so the tiny
// on-board LEDs don't blind anyone at night.
const MAX_CHANNEL: u32 = 50;

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut channel = init_matrix_tx(peripherals.RMT, peripherals.GPIO27);

    // The button ties GPIO39 to GND when pressed, so enable the pull-up and
    // wait for falling edges.
    let mut button = Input::new(
        peripherals.GPIO39,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut rng = Rng::new();

    let mut color = random_color(&mut rng, (0, 0, 0));
    println!("Initial color: r={} g={} b={}", color.0, color.1, color.2);
    fill_matrix(&mut channel, color.0, color.1, color.2)
        .await
        .unwrap();

    loop {
        // Blocks the task until the button is pressed (high -> low).
        button.wait_for_falling_edge().await;

        // Debounce: ignore contact bounce right after the press.
        embassy_time::block_for(embassy_time::Duration::from_millis(50));

        color = random_color(&mut rng, color);
        println!("New color: r={} g={} b={}", color.0, color.1, color.2);
        fill_matrix(&mut channel, color.0, color.1, color.2)
            .await
            .unwrap();

        // Wait for the button to be released before re-arming, so one press
        // only ever produces one color change.
        button.wait_for_rising_edge().await;
    }
}

/// Draw a random RGB color that differs from `previous`.
fn random_color(rng: &mut Rng, previous: (u8, u8, u8)) -> (u8, u8, u8) {
    loop {
        let r = (rng.random() % MAX_CHANNEL) as u8;
        let g = (rng.random() % MAX_CHANNEL) as u8;
        let b = (rng.random() % MAX_CHANNEL) as u8;
        if (r, g, b) != previous {
            return (r, g, b);
        }
    }
}
