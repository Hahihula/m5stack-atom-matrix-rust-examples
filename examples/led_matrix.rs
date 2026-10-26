//! Drive the M5Stack Atom Matrix 5x5 WS2812 matrix from GPIO27.
//!
//! Brighter variant of the binary in `src/main.rs`.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::Level,
    rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator},
    rng::Rng,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// RMT tick is 12.5 ns at 80 MHz with a clock divider of 1.
const T0H: u16 = 32; // ~400 ns (spec: 400 ns ± 150 ns)
const T0L: u16 = 70; // ~875 ns (spec: 850 ns ± 150 ns)
const T1H: u16 = 64; // ~800 ns (spec: 800 ns ± 150 ns)
const T1L: u16 = 38; // ~475 ns (spec: 450 ns ± 150 ns)

fn create_led_bits(r: u8, g: u8, b: u8) -> [PulseCode; 25] {
    let mut data = [PulseCode::new(Level::Low, 0, Level::Low, 0); 25];

    // WS2812 expects GRB order.
    let bytes = [g, r, b];

    let mut idx = 0;
    for byte in bytes {
        for bit in (0..8).rev() {
            data[idx] = if (byte & (1 << bit)) != 0 {
                PulseCode::new(Level::High, T1H, Level::Low, T1L)
            } else {
                PulseCode::new(Level::High, T0H, Level::Low, T0L)
            };
            idx += 1;
        }
    }
    // A zero-length second pulse is the RMT end marker. 800 ticks is ~10 µs low.
    data[24] = PulseCode::new(Level::Low, 800, Level::Low, 0);
    data
}

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80))
        .unwrap()
        .into_async();

    let mut channel = rmt
        .channel0
        .configure_tx(&TxChannelConfig::default().with_clk_divider(1))
        .unwrap()
        .with_pin(peripherals.GPIO27);

    let rng = Rng::new();

    loop {
        println!("Settings LED colors:");
        for _ in 0..25 {
            let r = rng.random() % 50;
            let g = rng.random() % 50;
            let b = rng.random() % 50;

            let data = create_led_bits(r as u8, g as u8, b as u8);
            channel.transmit(&data).await.unwrap();
        }
        Timer::after(Duration::from_millis(2500)).await;
    }
}
