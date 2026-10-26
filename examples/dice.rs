//! Dice demo for the M5Stack Atom Matrix.
//!
//! Shaking the board (detected via the on-board MPU6886 IMU on the internal
//! I2C bus) rolls a die: a short random-face animation plays, then the
//! result is shown as a dice face (pips) on the 5x5 matrix until the next
//! shake.
//!
//! Pin order, bus speed and init sequence follow the official M5Atom
//! Arduino library: `Wire.begin(25, 21, 100000)` - SDA=GPIO25, SCL=GPIO21,
//! 100 kHz.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    i2c::master::{Config, Error, I2c},
    rng::Rng,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use m5stack_atom_matrix_rust::{
    fill_matrix, init_matrix_tx, send_frame, strip_index, MatrixChannel, NUM_PIXELS,
};

esp_bootloader_esp_idf::esp_app_desc!();

// MPU6886 registers (same layout as MPU6500).
const MPU6886_ADDR: u8 = 0x68;
const PWR_MGMT_1: u8 = 0x6B;
const ACCEL_XOUT_H: u8 = 0x3B;
const SMPLRT_DIV: u8 = 0x19;
const CONFIG: u8 = 0x1A;
const GYRO_CONFIG: u8 = 0x1B;
const ACCEL_CONFIG: u8 = 0x1C;
const ACCEL_CONFIG2: u8 = 0x1D;
const INT_ENABLE: u8 = 0x38;
const WHO_AM_I: u8 = 0x75;
const USER_CTRL: u8 = 0x6A;
const MPU6886_WHO_AM_I_VALUE: u8 = 0x19;

/// Register sequence from M5Atom's `MPU6886::Init`: reset, select the
/// auto clock, then configure the ranges. (reg, value, delay_ms)
///
/// ACCEL_CONFIG 0x00 selects the +/-2g range, i.e. 16384 LSB per g, which
/// the shake-detection constants below assume.
const INIT_SEQUENCE: &[(u8, u8, u64)] = &[
    (PWR_MGMT_1, 0x00, 10),
    (PWR_MGMT_1, 0x80, 10), // full device reset
    (PWR_MGMT_1, 0x01, 10), // wake, auto-select best clock source
    (ACCEL_CONFIG, 0x00, 1),
    (GYRO_CONFIG, 0x18, 1),
    (CONFIG, 0x01, 1),
    (SMPLRT_DIV, 0x05, 1),
    (INT_ENABLE, 0x00, 1),
    (ACCEL_CONFIG2, 0x00, 1),
    (USER_CTRL, 0x00, 1),
];

// At the default +/-2g range, 1 g = 16384 LSB.
const G_SQ: i64 = 16384 * 16384;
/// Jerk threshold: magnitude-squared change between samples equivalent to a
/// ~1 g swing. A casual hand shake produces a few of these in a row; small
/// bumps and wiggles stay below it.
const JERK_2: i64 = G_SQ;
/// Jerks that must occur close together to count as a shake.
const JERKS_NEEDED: u32 = 4;
/// Poll period for the accelerometer.
const POLL_MS: u64 = 20;

// Pip positions (column, row) of the six die faces on the 5x5 grid.
const FACE_PIPS: [&[(u8, u8)]; 6] = [
    &[(2, 2)],
    &[(1, 1), (3, 3)],
    &[(1, 1), (2, 2), (3, 3)],
    &[(1, 1), (1, 3), (3, 1), (3, 3)],
    &[(1, 1), (1, 3), (2, 2), (3, 1), (3, 3)],
    &[(1, 0), (1, 2), (1, 4), (3, 0), (3, 2), (3, 4)],
];

// Dim red, in the same low-brightness range as the other demos.
const PIP: (u8, u8, u8) = (30, 5, 5);
const OFF: (u8, u8, u8) = (0, 0, 0);

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut channel = init_matrix_tx(peripherals.RMT, peripherals.GPIO27);
    let mut i2c = match init_imu().await {
        Some(i2c) => i2c,
        None => {
            println!("MPU6886 not responding, stopping");
            error_blink(&mut channel).await;
            return;
        }
    };

    let rng = Rng::new();

    let mut face = 1;
    println!("Face: {}", face);
    show_face(&mut channel, face).await;

    loop {
        wait_for_shake(&mut i2c).await;

        // Rolling animation: flip through random faces briefly, then settle.
        for _ in 0..8 {
            face = 1 + (rng.random() % 6) as u8;
            show_face(&mut channel, face).await;
            Timer::after(Duration::from_millis(70)).await;
        }

        face = 1 + (rng.random() % 6) as u8;
        println!("Face: {}", face);
        show_face(&mut channel, face).await;

        // Don't re-arm until the board has been still for a moment, so the
        // aftermath of the shake doesn't immediately trigger another roll.
        wait_for_quiet(&mut i2c).await;
    }
}

/// Bring up the I2C bus and initialize the MPU6886.
///
/// Pin order and bus speed match the official M5Atom Arduino library
/// (`Wire.begin(25, 21, 100000UL)`); the register sequence is the same one
/// its `MPU6886::Init` performs. Notably the IMU answers WHO_AM_I even while
/// asleep, so it is checked before the reset sequence.
///
/// The `steal()` calls are safe because the single driver built here lives
/// for the rest of the program.
async fn init_imu() -> Option<I2c<'static, esp_hal::Async>> {
    let mut i2c = I2c::new(
        unsafe { esp_hal::peripherals::I2C0::steal() },
        Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(unsafe { esp_hal::peripherals::GPIO25::steal() })
    .with_scl(unsafe { esp_hal::peripherals::GPIO21::steal() })
    .into_async();

    let mut id = [0u8; 1];
    if i2c.write_read(MPU6886_ADDR, &[WHO_AM_I], &mut id).is_err() {
        println!("No ACK from 0x68 (SDA=GPIO25, SCL=GPIO21)");
        return None;
    }
    if id[0] != MPU6886_WHO_AM_I_VALUE {
        println!(
            "WHO_AM_I 0x{:02X} != expected 0x{:02X}",
            id[0], MPU6886_WHO_AM_I_VALUE
        );
        return None;
    }
    println!("MPU6886 found (WHO_AM_I 0x{:02X})", id[0]);

    for &(reg, val, delay_ms) in INIT_SEQUENCE {
        if i2c.write(MPU6886_ADDR, &[reg, val]).is_err() {
            println!("I2C write failed during init");
            return None;
        }
        Timer::after(Duration::from_millis(delay_ms)).await;
    }

    Some(i2c)
}

/// Show that IMU init failed: blink the whole matrix red three times.
async fn error_blink(channel: &mut MatrixChannel) {
    for _ in 0..3 {
        fill_matrix(channel, 30, 0, 0).await.unwrap();
        Timer::after(Duration::from_millis(200)).await;
        fill_matrix(channel, 0, 0, 0).await.unwrap();
        Timer::after(Duration::from_millis(200)).await;
    }
}

/// Squared magnitude of the acceleration vector, in (LSB)^2.
///
/// Squared so everything stays in integer math - sqrt is neither needed nor
/// available in no_std by default.
async fn accel_sq(i2c: &mut I2c<'static, esp_hal::Async>) -> Result<i64, Error> {
    let mut buf = [0u8; 6];
    // ACCEL_XOUT_H..ACCEL_ZOUT_L, three big-endian i16 values.
    i2c.write_read(MPU6886_ADDR, &[ACCEL_XOUT_H], &mut buf)?;
    let x = i16::from_be_bytes([buf[0], buf[1]]) as i64;
    let y = i16::from_be_bytes([buf[2], buf[3]]) as i64;
    let z = i16::from_be_bytes([buf[4], buf[5]]) as i64;
    Ok(x * x + y * y + z * z)
}

/// Poll the accelerometer until a shake is detected.
///
/// A "jerk" is a large frame-to-frame change of the acceleration magnitude;
/// `JERKS_NEEDED` jerks in quick succession (with slow decay) count as a
/// shake.
async fn wait_for_shake(i2c: &mut I2c<'static, esp_hal::Async>) {
    let mut prev_sq = accel_sq(i2c).await.unwrap_or(G_SQ);
    let mut jerks: u32 = 0;

    loop {
        Timer::after(Duration::from_millis(POLL_MS)).await;
        let Ok(sq) = accel_sq(i2c).await else {
            continue;
        };

        if (sq - prev_sq).abs() > JERK_2 {
            jerks += 1;
        } else {
            jerks = jerks.saturating_sub(1);
        }
        prev_sq = sq;

        if jerks >= JERKS_NEEDED {
            return;
        }
    }
}

/// Poll until the board has been still for a stretch of samples.
async fn wait_for_quiet(i2c: &mut I2c<'static, esp_hal::Async>) {
    let mut prev_sq = accel_sq(i2c).await.unwrap_or(G_SQ);
    let mut calm_samples: u32 = 0;

    while calm_samples < 15 {
        Timer::after(Duration::from_millis(POLL_MS)).await;
        let Ok(sq) = accel_sq(i2c).await else {
            continue;
        };

        if (sq - prev_sq).abs() > JERK_2 {
            calm_samples = 0;
        } else {
            calm_samples += 1;
        }
        prev_sq = sq;
    }
}

/// Draw die face `face` (1..=6) on the matrix.
async fn show_face(channel: &mut MatrixChannel, face: u8) {
    let mut frame = [OFF; NUM_PIXELS];
    for &(x, y) in FACE_PIPS[(face - 1) as usize] {
        frame[strip_index(x as usize, y as usize)] = PIP;
    }
    send_frame(channel, &frame).await.unwrap();
}
