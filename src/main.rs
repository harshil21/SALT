use std::process::Command;
use bmp280::Bmp280Builder;
use std::thread;
use bno055;
use linux_embedded_hal::I2cdev;

fn main() {
    // Start new thread for pressure sensor:
    run_sensors();
}

fn record_video() {
    let status = Command::new("rpicam-vid")
        .args(&["-t", "5000", "-o", "test.h264", "--inline", "--awb", "auto", "--width", "1920", "--height", "1080"])
        .status()
        .expect("failed to execute camera command");

    if status.success() {
        println!("Video captured successfully!");
    } else {
        eprintln!("Camera command failed with status: {:?}", status);
    }
}

fn run_sensors() {
    let mut bmp280 = init_pressure_sensor();
    let mut sensor = init_orientation_sensor();


    loop {
        // Pressure
        if let Ok(pressure) = bmp280.pressure_kpa() {
            println!("{:?} kPa", pressure);
        } else {
            println!("Read/write error");
        }

        // Altitude
        if let Ok(altitude) = bmp280.altitude_m() {
            println!("{:?} m", altitude);
        } else {
            println!("Read/write error");
        }

        // Temperature
        if let Ok(temp) = bmp280.temperature_celsius() {
            println!("{:?} °C", temp);
        } else {
            println!("Read/write error");
        }

        // Orientation
        match sensor.euler_angles() {
            Ok(euler) => {
                println!("Orientation: {:?}", euler);
            }
            Err(e) => {
                eprintln!("Error reading orientation: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn init_pressure_sensor() -> bmp280::Bmp280 {
    // Initialize the BMP280 sensor
    // We can have I2C read/write errors sometimes, so just keep trying:
    let mut bmp280 = loop {
        if let Ok(dev) = Bmp280Builder::new().build() {
            break dev
        }
    };

    bmp280.zero().expect("Failed to reset pressure to zero");
    return bmp280;
}

fn init_orientation_sensor() -> bno055::Bno055<linux_embedded_hal::I2cdev> {
    // Initialize the BNO055 sensor
    let i2c = I2cdev::new("/dev/i2c-1")
        .expect("Failed to open I2C device");
    
    let sensor = bno055::Bno055::new(i2c);
    sensor
}
