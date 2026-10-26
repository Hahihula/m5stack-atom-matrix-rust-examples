//! Counter demo for the M5Stack Atom Matrix.
//!
//! Shows a 3x5 digit (0-9, wrapping) centered on the 5x5 matrix; each press
//! of the user button (GPIO39, active-low with an internal pull-up)
//! increments it.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    timer::timg::TimerGroup,
};
use esp_println::println;
use m5stack_atom_matrix_rust::{send_frame, strip_index, MatrixChannel, NUM_PIXELS};

esp_bootloader_esp_idf::esp_app_desc!();

// Digit color, kept in the same low-brightness range as the other demos.
const COLOR: (u8, u8, u8) = (10, 30, 30);

/// 3x5 digit glyphs, one row per entry, bit 2 = left column of the glyph.
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b001, 0b001, 0b001], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
];

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut channel = m5stack_atom_matrix_rust::init_matrix_tx(peripherals.RMT, peripherals.GPIO27);

    let mut button = Input::new(
        peripherals.GPIO39,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut count: u8 = 0;
    println!("Count: {}", count);
    show_count(&mut channel, count).await;

    loop {
        // Blocks the task until the button is pressed (high -> low).
        button.wait_for_falling_edge().await;

        // Debounce: ignore contact bounce right after the press.
        embassy_time::block_for(embassy_time::Duration::from_millis(50));

        count = count.wrapping_add(1) % 10;
        println!("Count: {}", count);
        show_count(&mut channel, count).await;

        // Wait for the button to be released before re-arming, so one press
        // only ever produces one increment.
        button.wait_for_rising_edge().await;
    }
}

/// Render `count % 10` as a 3x5 digit centered on the matrix.
///
/// The glyph occupies columns 1..3 so it sits in the middle of the 5x5 grid.
async fn show_count(channel: &mut MatrixChannel, count: u8) {
    let digit = &DIGITS[(count % 10) as usize];
    let mut frame = [(0u8, 0u8, 0u8); NUM_PIXELS];

    for y in 0..5 {
        for x in 1..4 {
            if (digit[y] >> (2 - (x - 1))) & 1 != 0 {
                frame[strip_index(x, y)] = COLOR;
            }
        }
    }

    send_frame(channel, &frame).await.unwrap();
}
