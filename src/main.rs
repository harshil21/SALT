use std::process::Command;
use bmp280::Bmp280Builder;
use std::thread;

fn main() {
    init_pressure_sensor();
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


fn init_pressure_sensor() {
    // We can have I2C read/write errors sometimes, so just keep trying:
    let mut bmp280 = loop {
        if let Ok(dev) = Bmp280Builder::new().build() {
            break dev
        }
    };

    bmp280.zero().expect("Failed to reset pressure to zero");

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

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
