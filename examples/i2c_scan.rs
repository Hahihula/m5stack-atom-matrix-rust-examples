//! I2C bus scanner for the M5Stack Atom Matrix.
//!
//! The internal IMU did not respond to the MPU6886 address (0x68) in the
//! dice demo, so this scans every valid 7-bit address on both possible
//! SDA/SCL pin orders and prints what ACKs. Early Atom Matrix units shipped
//! with a SH200Q IMU (address 0x6C) instead of the MPU6886.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    i2c::master::{Config, I2c},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

const KNOWN_DEVICES: &[(u8, &str)] = &[
    (0x68, "MPU6886 / MPU6050"),
    (0x6C, "SH200Q"),
    (0x19, "LSM6DS3 (some Atom boards)"),
];

fn build_i2c_sda21_scl25() -> I2c<'static, esp_hal::Blocking> {
    I2c::new(
        unsafe { esp_hal::peripherals::I2C0::steal() },
        Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(unsafe { esp_hal::peripherals::GPIO21::steal() })
    .with_scl(unsafe { esp_hal::peripherals::GPIO25::steal() })
}

fn build_i2c_sda25_scl21() -> I2c<'static, esp_hal::Blocking> {
    I2c::new(
        unsafe { esp_hal::peripherals::I2C0::steal() },
        Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(unsafe { esp_hal::peripherals::GPIO25::steal() })
    .with_scl(unsafe { esp_hal::peripherals::GPIO21::steal() })
}

/// Probe one 7-bit address with a 1-byte read.
fn responds(i2c: &mut I2c<'static, esp_hal::Blocking>, addr: u8) -> bool {
    let mut buf = [0u8; 1];
    i2c.read(addr, &mut buf).is_ok()
}

/// Scan all valid 7-bit addresses, print every device that ACKs.
fn scan(mut i2c: I2c<'static, esp_hal::Blocking>, label: &str) {
    println!("--- scanning with {} ---", label);
    let mut found = 0;
    for addr in 0x08..=0x77u8 {
        if responds(&mut i2c, addr) {
            found += 1;
            match KNOWN_DEVICES.iter().find(|(a, _)| *a == addr) {
                Some((_, name)) => println!("  ACK at 0x{:02X} ({})", addr, name),
                None => println!("  ACK at 0x{:02X}", addr),
            }
        }
    }
    if found == 0 {
        println!("  no devices found");
    }
}

#[esp_hal::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    loop {
        scan(build_i2c_sda21_scl25(), "SDA=GPIO21, SCL=GPIO25");
        scan(build_i2c_sda25_scl21(), "SDA=GPIO25, SCL=GPIO21");
        println!("--- scan complete, repeating in 5 s ---");
        embassy_time::Timer::after(embassy_time::Duration::from_millis(5000)).await;
    }
}
