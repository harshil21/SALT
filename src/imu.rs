use bmp280::{Bmp280, Bmp280Builder};
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::*;
use mpu6050::device::{WHOAMI, AccelRange, GyroRange, ACCEL_HPF};
use std::thread;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct IMUDataPacket {
    pub timestamp: u64,
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
    mpu6050: Option<Mpu6050<I2cdev>>,
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
        println!("BMP280 sensor initialized.");
        bmp280.zero().expect("Failed to reset pressure to zero");
        bmp280.zero().expect("Failed to reset pressure to zero");
        // Try to create and initialize MPU6050; if anything fails keep None.
        let mut delay = Delay;
        let mpu6050 = match I2cdev::new("/dev/i2c-1") {
            Ok(i2cdev) => {
                let mut sensor = Mpu6050::new_with_addr(i2cdev, 0x68);
                match sensor.init(&mut delay) {
                    Ok(()) => {
                        println!("MPU6050 initialized at address 0x68.");
                        Some(sensor)
                    }
                    Err(Mpu6050Error::InvalidChipId(id)) => {
                        eprintln!("MPU6050 WHO_AM_I mismatch at 0x68: 0x{:02X}. Trying manual init...", id);
                        let manual_ok = sensor.set_sleep_enabled(false).is_ok()
                            && { thread::sleep(std::time::Duration::from_millis(100)); true }
                            && sensor.set_accel_range(AccelRange::G2).is_ok()
                            && sensor.set_gyro_range(GyroRange::D250).is_ok()
                            && sensor.set_accel_hpf(ACCEL_HPF::_RESET).is_ok();
                        if manual_ok {
                            if let Ok(whoami) = sensor.read_byte(WHOAMI) {
                                eprintln!("Manual init succeeded; WHO_AM_I now 0x{:02X}", whoami);
                            } else {
                                eprintln!("Manual init succeeded; WHO_AM_I read failed");
                            }
                            Some(sensor)
                        } else {
                            eprintln!("Manual init failed at 0x68. Trying address 0x69...");
                            match I2cdev::new("/dev/i2c-1") {
                                Ok(i2cdev2) => {
                                    let mut sensor2 = Mpu6050::new_with_addr(i2cdev2, 0x69);
                                    match sensor2.init(&mut delay) {
                                        Ok(()) => {
                                            println!("MPU6050 initialized at address 0x69.");
                                            Some(sensor2)
                                        }
                                        Err(err69) => {
                                            eprintln!("MPU6050 init failed at 0x69: {:?}. Continuing without it.", err69);
                                            None
                                        }
                                    }
                                }
                                Err(eopen2) => {
                                    eprintln!("Failed to reopen /dev/i2c-1 for alt address: {:?}. Continuing without sensor.", eopen2);
                                    None
                                }
                            }
                        }
                    }
                    Err(err68) => {
                        eprintln!("MPU6050 init failed at 0x68: {:?}", err68);
                        match I2cdev::new("/dev/i2c-1") {
                            Ok(i2cdev2) => {
                                let mut sensor2 = Mpu6050::new_with_addr(i2cdev2, 0x69);
                                match sensor2.init(&mut delay) {
                                    Ok(()) => {
                                        println!("MPU6050 initialized at address 0x69.");
                                        Some(sensor2)
                                    }
                                    Err(err69) => {
                                        eprintln!("MPU6050 init failed at 0x69: {:?}. Continuing without it.", err69);
                                        None
                                    }
                                }
                            }
                            Err(eopen2) => {
                                eprintln!("Failed to reopen /dev/i2c-1 for alt address: {:?}. Continuing without sensor.", eopen2);
                                None
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to open /dev/i2c-1 for MPU6050: {:?}. Continuing without sensor.", e);
                None
            }
        };

        // The initial data packet is created directly.
        let initial_packet = IMUDataPacket {
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            acceleration: [0.0, 0.0, 0.0],
            quaternion: [1.0, 0.0, 0.0, 0.0],
            pressure_alt: 0.0,
            temperature: 20.0,
            magnetic_field: [0.0, 0.0, 0.0],
            gyro: [0.0, 0.0, 0.0],
            pressure: 101325.0, // Default pressure at sea level in Pascals
        };

        IMU { bmp280, mpu6050, imu_data_packet: initial_packet }
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

        // Read from MPU6050 if available
        if let Some(mpu) = self.mpu6050.as_mut() {
            if let Ok(acc) = mpu.get_acc() {
                self.imu_data_packet.acceleration = [acc.x, acc.y, acc.z];
            } else {
                eprintln!("Failed to read acceleration from MPU6050");
            }
            if let Ok(gyro) = mpu.get_gyro() {
                self.imu_data_packet.gyro = [gyro.x, gyro.y, gyro.z];
            } else {
                eprintln!("Failed to read gyroscope from MPU6050");
            }
        }
        // Quaternion and magnetic field not available from MPU6050; keep previous values.
        // Always update the timestamp to the time of the last read attempt.
        self.imu_data_packet.timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
    }
}
