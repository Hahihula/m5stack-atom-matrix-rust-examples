//! Shared helpers for the M5Stack Atom Matrix demos.
//!
//! The board drives a 5x5 WS2812 matrix on GPIO27 and has a user button on
//! GPIO39 (tied to GND when pressed, so it needs a pull-up).

#![no_std]

use esp_hal::{
    gpio::{interconnect::PeripheralOutput, Level},
    rmt::{Channel, PulseCode, Rmt, TxChannelConfig, TxChannelCreator},
    time::Rate,
};

/// GPIO the WS2812 matrix data line is wired to.
pub const MATRIX_GPIO: u8 = 27;

/// Number of pixels in the matrix (5x5).
pub const NUM_PIXELS: usize = 25;

// RMT tick is 12.5 ns at 80 MHz with a clock divider of 1.
const T0H: u16 = 32; // ~400 ns (spec: 400 ns ± 150 ns)
const T0L: u16 = 70; // ~875 ns (spec: 850 ns ± 150 ns)
const T1H: u16 = 64; // ~800 ns (spec: 800 ns ± 150 ns)
const T1L: u16 = 38; // ~475 ns (spec: 450 ns ± 150 ns)

/// Encode a single WS2812 pixel (24 data bits + reset gap) as RMT pulse codes.
///
/// WS2812 expects the bytes in GRB order.
pub fn create_led_bits(r: u8, g: u8, b: u8) -> [PulseCode; 25] {
    let mut data = [PulseCode::new(Level::Low, 0, Level::Low, 0); 25];

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

/// Build the RMT peripheral at the 80 MHz clock the WS2812 timings assume and
/// return an async TX channel wired to the matrix data pin.
pub fn init_matrix_tx(
    rmt: esp_hal::peripherals::RMT<'static>,
    pin: impl PeripheralOutput<'static>,
) -> Channel<'static, esp_hal::Async, esp_hal::rmt::Tx> {
    Rmt::new(rmt, Rate::from_mhz(80))
        .unwrap()
        .into_async()
        .channel0
        .configure_tx(&TxChannelConfig::default().with_clk_divider(1))
        .unwrap()
        .with_pin(pin)
}

/// Push one RGB color to a single pixel over RMT.
pub async fn send_pixel(
    channel: &mut Channel<'static, esp_hal::Async, esp_hal::rmt::Tx>,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), esp_hal::rmt::Error> {
    let data = create_led_bits(r, g, b);
    channel.transmit(&data).await
}

/// Fill the whole matrix with one RGB color.
pub async fn fill_matrix(
    channel: &mut Channel<'static, esp_hal::Async, esp_hal::rmt::Tx>,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), esp_hal::rmt::Error> {
    for _ in 0..NUM_PIXELS {
        send_pixel(channel, r, g, b).await?;
    }
    Ok(())
}
