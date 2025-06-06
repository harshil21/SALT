use bmp280::{Bmp280, Bmp280Builder};
use linux_bno055::Bno055;
use std::thread;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct IMUDataPacket {
    pub timestamp: Instant,
    // From the BNO055 sensor:
    pub acceleration: [f32; 3],   // array of (x, y, z) in m/s^2
    pub quaternion: [f32; 4],     // array of (w, x, y, z)
    pub magnetic_field: [f32; 3], // Magnetic field in microteslas
    pub gyro: [f32; 3],           // rad/s

    // From the BMP280 sensor:
    pub pressure_alt: f32, // Altitude derived from pressure
    pub temperature: f32,  // Temperature in Celsius
    pub pressure: f32,     // Pressure in Kilo Pascals
}

pub struct IMU {
    bmp280: Bmp280,
    bno055: Bno055,
    // The data packet is now owned directly by the struct.
    imu_data_packet: IMUDataPacket,
}

impl IMU {
    pub fn new() -> Self {
        // Initialize sensors.
        let mut bmp280 = loop {
            println!("Waiting for BMP280 sensor to be ready...");
            if let Ok(dev) = Bmp280Builder::new().build() {
                break dev;
            }
            thread::sleep(std::time::Duration::from_secs(1));
        };
        bmp280.zero().expect("Failed to reset pressure to zero");
        let sensor = Bno055::new("/dev/i2c-1").expect("Failed to create BNO055 sensor instance");

        // The initial data packet is created directly.
        let initial_packet = IMUDataPacket {
            timestamp: Instant::now(),
            acceleration: [0.0, 0.0, 0.0],
            quaternion: [1.0, 0.0, 0.0, 0.0],
            pressure_alt: 0.0,
            temperature: 20.0,
            magnetic_field: [0.0, 0.0, 0.0],
            gyro: [0.0, 0.0, 0.0],
            pressure: 101325.0, // Default pressure at sea level in Pascals
        };

        IMU {
            bmp280,
            bno055: sensor,
            imu_data_packet: initial_packet,
        }
    }

    /// Provides a clone of the most recent IMU data packet.
    pub fn get_data_packet(&mut self) -> IMUDataPacket {
        self.read_data();
        self.imu_data_packet.clone()
    }

    /// Reads new sensor data and updates the internal data packet.
    pub fn read_data(&mut self) {
        // Update fields directly on the struct's data packet.
        // If a sensor read fails, the old value is kept.
        if let Ok(altitude) = self.bmp280.altitude_m() {
            self.imu_data_packet.pressure_alt = altitude;
        } else {
            eprintln!("Failed to read altitude from BMP280");
        }

        if let Ok(temp) = self.bmp280.temperature_celsius() {
            self.imu_data_packet.temperature = temp;
        } else {
            eprintln!("Failed to read temperature from BMP280");
        }

        if let Ok(pressure) = self.bmp280.pressure_kpa() {
            self.imu_data_packet.pressure = pressure;
        } else {
            eprintln!("Failed to read pressure from BMP280");
        }

        if let Ok(quat) = self.bno055.get_quaternion() {
            self.imu_data_packet.quaternion = [quat.w, quat.x, quat.y, quat.z];
        } else {
            eprintln!("Failed to read quaternion from BNO055");
        }

        if let Ok(mag) = self.bno055.get_magnetometer() {
            self.imu_data_packet.magnetic_field = [mag.x, mag.y, mag.z];
        } else {
            eprintln!("Failed to read magnetic field from BNO055");
        }

        if let Ok(acc) = self.bno055.get_accelerometer() {
            self.imu_data_packet.acceleration = [acc.x, acc.y, acc.z];
        } else {
            eprintln!("Failed to read acceleration from BNO055");
        }

        if let Ok(gyro) = self.bno055.get_gyroscope() {
            self.imu_data_packet.gyro = [gyro.x, gyro.y, gyro.z];
        } else {
            eprintln!("Failed to read gyroscope from BNO055");
        }

        // Always update the timestamp to the time of the last read attempt.
        self.imu_data_packet.timestamp = Instant::now();
    }
}
