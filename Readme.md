# M5Stack Atom Matrix Rust Examples (no_std)

<p align="center">
  <img src="https://static-cdn.m5stack.com/resource/docs/products/core/ATOM%20Matrix/img-695ed4da-4ff3-4a46-bc09-a4afab9fded1.webp" alt="M5Stack Atom Matrix" height="200">
  <img src="https://static-cdn.m5stack.com/resource/docs/products/core/ATOM%20Matrix/img-fa9922b3-727d-4598-8d16-84611248a3c6.webp" alt="M5Stack Atom Matrix Scheme" height="200">
</p>

This repository provides `no_std` Rust examples for the M5Stack Atom Matrix board. The examples demonstrate how to interact with the device's LED matrix and other peripherals using Rust in a bare-metal environment.

**Key Features:**

* **`no_std` Support:** Designed for embedded systems, avoiding the standard library.
* **Clear Examples:** Step-by-step examples for common functionalities.
* **Easy Setup:** Comprehensive instructions for getting started.

**Resources:**

* [M5Stack Atom Matrix Documentation](https://docs.m5stack.com/en/core/ATOM%20Matrix)

## Quick Start

1.  **Clone the Repository:**

    ```bash
    git clone [https://github.com/hahihula/m5stack-atom-matrix-rust-examples.git](https://github.com/hahihula/m5stack-atom-matrix-rust-examples.git)
    cd m5stack-atom-matrix-rust-examples
    ```

2.  **Build and Flash the `led_matrix` Example:**

    ```bash
    cargo build --release --example led_matrix
    cargo espflash flash --release --example led_matrix
    ```

3.  **Monitor Serial Output (Optional):**

    ```bash
    cargo espflash monitor
    ```

## Detailed Setup

If you don't have Rust and the necessary tools installed, follow these steps:

### 1. Install Rust and the Xtensa toolchain

The Atom Matrix uses an ESP32 (Xtensa). `esp-hal` 1.2.2 needs Rust 1.95 or newer, and Xtensa still needs the Espressif compiler fork. This repo pins that compiler with `rust-toolchain.toml` (`channel = "esp"`).

1.  **Install Rustup:**
    * Visit [rustup.rs](https://rustup.rs) and follow the installation instructions for your operating system.

2.  **Install `espup` and the toolchain:**

    ```bash
    cargo install espup --locked
    espup install
    ```

    On Unix, source the exported environment (`source ~/export-esp.sh`) so `cargo` can find the Xtensa linker. See the [Rust on ESP book](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html).

### 2. Install `cargo-espflash`

1.  **Install `cargo-espflash`:**

    ```bash
    cargo install cargo-espflash
    ```

2.  **Verify Installation:**

    ```bash
    cargo espflash --version
    ```

### 3. Clone and Build

1.  **Clone the Repository:**

    ```bash
    git clone [https://github.com/hahihula/m5stack-atom-matrix-rust-examples.git](https://github.com/hahihula/m5stack-atom-matrix-rust-examples.git)
    cd m5stack-atom-matrix-rust-examples
    ```

2.  **Connect the M5Stack Atom Matrix:**
    * Connect the device to your computer via USB-C.

3.  **Build and Flash Examples:**
    * For example, to build and flash the `led_matrix` example:

        ```bash
        cargo build --release --example led_matrix
        cargo espflash flash --release --example led_matrix
        ```

4.  **Monitor Serial Output (Optional):**
    * To view serial output:

        ```bash
        cargo espflash monitor
        ```

## Examples

* **`led_matrix`:** Basic LED matrix control.
    * Build and flash: `cargo espflash flash --release --example led_matrix`

**Contributing**

Contributions are welcome! If you have any examples or improvements, feel free to submit a pull request.

**License**

MIT