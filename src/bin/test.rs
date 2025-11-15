use bmp280::Bmp280Builder;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

struct TransmittedPacket {
    pressure: f32,
    altitude: f32,
    temperature: f32,
    orientation: (f32, f32, f32), // (roll, pitch, yaw)
}

fn main() {
    // Start new thread for pressure sensor:
    thread::spawn(|| {
        run_sensors();
    });
    // thread::spawn(|| {
    //     transmit_data();
    // });
    // record_video();
    thread::sleep(Duration::from_secs(10));
}

fn record_video() {
    let status = Command::new("rpicam-vid")
        .args([
            "-t",
            "10000",
            "-o",
            "test.h264",
            "--inline",
            "--awb",
            "auto",
            "--width",
            "1920",
            "--height",
            "1080",
        ])
        .stdout(Stdio::null())
        .output();

    match status {
        Ok(output) => {
            if output.status.success() {
                println!("Video recording started successfully.");
            } else {
                eprintln!("Failed to start video recording: {:?}", output);
            }
        }
        Err(e) => {
            eprintln!("Error executing rpicam-vid command: {}", e);
        }
    }
}

fn transmit_data() {
    // Open the serial port with desired settings
    let mut iter = 0;
    loop {
        let mut port = serialport::new("/dev/serial0", 9600)
            .timeout(Duration::from_millis(1000))
            .open()
            .expect("Failed to open port");

        // Data to send
        iter += 1;
        let output = format!("Hello, serial world! {}\n", iter);
        println!("Transmitting: {}", output);

        // Write data to the serial port
        port.write_all(output.as_bytes()).expect("Write failed!");

        thread::sleep(Duration::from_millis(1000));
    }
}

fn run_sensors() {
    let mut bmp280 = init_pressure_sensor();
    let mut transmitted_packet = TransmittedPacket {
        pressure: 0.0,
        altitude: 0.0,
        temperature: 0.0,
        orientation: (0.0, 0.0, 0.0),
    };

    loop {
        // Pressure
        if let Ok(pressure) = bmp280.pressure_kpa() {
            println!("{:?} kPa", pressure);
            transmitted_packet.pressure = pressure;
        } else {
            println!("Read/write error");
        }

        // Altitude
        if let Ok(altitude) = bmp280.altitude_m() {
            println!("{:?} m", altitude);
            transmitted_packet.altitude = altitude;
        } else {
            println!("Read/write error");
        }

        // Temperature
        if let Ok(temp) = bmp280.temperature_celsius() {
            println!("{:?} °C", temp);
            transmitted_packet.temperature = temp;
        } else {
            println!("Read/write error");
        }

        // Orientation sensor removed (BNO055 crate not included). Orientation fields left at defaults.


        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn init_pressure_sensor() -> bmp280::Bmp280 {
    // Initialize the BMP280 sensor
    // We can have I2C read/write errors sometimes, so just keep trying:
    let mut bmp280 = loop {
        if let Ok(dev) = Bmp280Builder::new().build() {
            break dev;
        }
    };

    bmp280.zero().expect("Failed to reset pressure to zero");
    return bmp280;
}

// Orientation sensor initialization removed.
