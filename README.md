# SALT

Samuel's Atmospheric Live Telemetry (SALT) is a rocket made by [@samuel300p](https://github.com/samuel300p)
which features transmitting live telemetry from the rocket to a ground station using LoRa.

The main sensors and components include:

- A Raspberry Pi Zero 2 W
- 2 LoRa modules (EBytes' E22-400TXX series) transmitting @ 433.125MHz
- BMP280 for pressure and temperature
- BNO055 for orientation, acceleration, and magnetometer data
- An OV5647 camera for recording footage

There are 2 programs in this repository, the flight software and the ground station software.

The flight software, when first started, will wait for the command from the ground station to start
transmitting telemetry in real time and logging all sensor data at 50Hz, and start recording footage
from the camera.

The ground station software will receive the telemetry data and display it in real time.

## Flight Software

This is written in Rust (experimental, since I'm still learning Rust). To get started, you 
simply need to build the project with `cargo build --release` and run `cargo run --release`.

Note: If you want to skip waiting for compilation on the Pi Zero 2W, you can cross compile via `cross` and Docker - `cargo install cross`,
and then: `cross build --target aarch64-unknown-linux-gnu`, and then copy the binary (from `target/`) to the Pi Zero 2W.

A prototyping script in Python is available as `main.py` for testing purposes, but it is not used in the final flight software.

Similarly, there's also a testing script written in Rust in `src/bin/test.rs`. Run it with `cargo run --bin test`.

## Ground Station Software

This is 100% vibe coded (the code looks really awful, but it works). There is a rocket visualization
and telemetry data display with graphs. The GUI backend is written in Python using PyQt6. It also logs
all the telemetry data to a CSV file for later analysis (TODO!).

To run the ground station software, using [`uv`](https://docs.astral.sh/uv/):

```bash
uv run ground_station.py
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
