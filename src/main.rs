//! Main script for the Rust project.

use salt::context::Context;
use std::thread;

fn main() {
    let mut context = Context::new();

    // Wait for the rocket to be armed before starting the main loop:
    // println!("Waiting for rocket to be armed...");
    // while !context.wait_for_boot_command() {
    //     println!("waiting for boot command...");
    // }

    // Handle the first update:
    context.imu.read_data();
    context
        .data_processor
        .first_update(&context.imu.get_data_packet());

    // Main loop
    loop {
        context.update();

        // break from the loop if we are in shutdown state:
        if let salt::state::RocketState::Shutdown = context.state {
            println!("Shutting down...");
            break;
        }

        // Sleep for a short duration to avoid I2C flooding, and because the sensors have a max
        // update rate:
        thread::sleep(std::time::Duration::from_millis(49));
    }
}
