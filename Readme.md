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

### 1. Install Rust and Nightly Toolchain

1.  **Install Rustup:**
    * Visit [rustup.rs](https://rustup.rs) and follow the installation instructions for your operating system.

2.  **Add the Nightly Toolchain:**

    ```bash
    rustup toolchain install nightly
    rustup default nightly
    ```

3.  **Add the Xtensa Target:**

    ```bash
    rustup target add xtensa-esp32-espidf
    ```

### 2. Install `espup` and ESP-IDF

1.  **Install `espup`:**

    ```bash
    cargo install espup
    ```

2.  **Install ESP-IDF:**

    ```bash
    espup install
    ```

    * Follow the prompts to install the required ESP-IDF components.

### 3. Install `cargo-espflash`

1.  **Install `cargo-espflash`:**

    ```bash
    cargo install cargo-espflash
    ```

2.  **Verify Installation:**

    ```bash
    cargo espflash --version
    ```

### 4. Clone and Build

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