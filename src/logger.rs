//! This module will simply log the IMUDataPacket and ProcessorDataPacket to a file as a csv.

use crate::data_processor::ProcessorDataPacket;
use crate::imu::IMUDataPacket;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};

pub struct Logger {
    writer: BufWriter<std::fs::File>,
}

impl Logger {
    pub fn new() -> Self {
        // File name is the date and time in the format YYYY-MM-DD_HH-MM-SS.csv (naive)
        let file_path = format!(
            "logs/{}.csv",
            chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S")
        );

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
            .expect("Failed to open log file");

        let writer = BufWriter::new(file);

        let mut logger = Logger { writer };
        logger
            .write_headers()
            .expect("Failed to write headers to log file");
        logger
    }

    fn write_headers(&mut self) -> io::Result<()> {
        writeln!(
            self.writer,
            "timestamp,state,accel_x,accel_y,accel_z,gyro_x,gyro_y,gyro_z,mag_x,mag_y,mag_z,quat_w,quat_x,quat_y,quat_z,pressure,altitude,max_altitude,velocity,max_velocity,temperature"
        )?;
        Ok(())
    }

    pub fn log_packets(
        &mut self,
        imu_data: &IMUDataPacket,
        processor_data: &ProcessorDataPacket,
        state: &char
    ) -> () {
        if let Err(e) = writeln!(
            self.writer,
            "{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            imu_data.timestamp,
            state,
            imu_data.acceleration[0],
            imu_data.acceleration[1],
            imu_data.acceleration[2],
            imu_data.gyro[0],
            imu_data.gyro[1],
            imu_data.gyro[2],
            imu_data.magnetic_field[0],
            imu_data.magnetic_field[1],
            imu_data.magnetic_field[2],
            imu_data.quaternion[0],
            imu_data.quaternion[1],
            imu_data.quaternion[2],
            imu_data.quaternion[3],
            imu_data.pressure,
            processor_data.current_altitude,
            processor_data.maximum_altitude,
            processor_data.vertical_velocity,
            processor_data.maximum_velocity,
            imu_data.temperature
        ) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}
