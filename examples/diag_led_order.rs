//! Diagnostic for the physical LED ordering of the Atom Matrix strip.
//!
//! Runs a repeating sequence of labeled test patterns while printing what is
//! currently displayed to the console. Run with the monitor
//! (`cargo run --example diag_led_order`) and note which physical pixel(s)
//! light up for each printed label:
//!
//! - `all`     - every LED on, sanity check
//! - `chase`   - strip index 0..=24 one at a time; the path traced on the
//!   matrix IS the physical strip order
//! - `group N` - five consecutive strip indices at a time (0-4, 5-9, ...)
//! - `corners` - the four strip indices the current LED_MAP believes are the
//!   matrix corners (top-left, top-right, bottom-left, bottom-right)

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use m5stack_atom_matrix_rust::{
    fill_matrix, init_matrix_tx, send_frame, MatrixChannel, NUM_PIXELS,
};

esp_bootloader_esp_idf::esp_app_desc!();

// Dim white so the pattern is clearly visible but not blinding.
const ON: (u8, u8, u8) = (25, 25, 25);
const OFF: (u8, u8, u8) = (0, 0, 0);

/// Light only the given strip indices, leave the rest off.
async fn show(channel: &mut MatrixChannel, lit: &[usize]) {
    let mut frame = [OFF; NUM_PIXELS];
    for &i in lit {
        frame[i] = ON;
    }
    send_frame(channel, &frame).await.unwrap();
}

async fn pause(ms: u64) {
    Timer::after(Duration::from_millis(ms)).await;
}

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut channel = init_matrix_tx(peripherals.RMT, peripherals.GPIO27);

    loop {
        println!("=== phase: all (every LED on) ===");
        fill_matrix(&mut channel, ON.0, ON.1, ON.2).await.unwrap();
        pause(2000).await;

        show(&mut channel, &[]).await;
        pause(500).await;

        println!("=== phase: chase (strip index 0..=24, one at a time) ===");
        for i in 0..NUM_PIXELS {
            println!("chase: strip index {}", i);
            show(&mut channel, &[i]).await;
            pause(700).await;
        }

        show(&mut channel, &[]).await;
        pause(500).await;

        println!("=== phase: groups (5 consecutive strip indices at a time) ===");
        for g in 0..5 {
            let start = g * 5;
            println!("group {}: strip indices {}..={}", g, start, start + 4);
            let indices: [usize; 5] = core::array::from_fn(|k| start + k);
            show(&mut channel, &indices).await;
            pause(2000).await;
        }

        show(&mut channel, &[]).await;
        pause(500).await;

        println!("=== phase: corners (strip indices 0, 4, 20, 24) ===");
        show(&mut channel, &[0, 4, 20, 24]).await;
        pause(2000).await;

        show(&mut channel, &[]).await;
        pause(1500).await;
    }
}
