//! Data processing logic for the rocket.

use crate::constants::{ALTITUDE_DEADBAND_METERS, VELOCITY_FROM_ALTITUDE_WINDOW_SIZE};
use fixed_deque::Deque;

use crate::imu::IMUDataPacket;

pub struct DataProcessor {
    pub max_altitude: f32,
    pub max_velocity: f32,
    pub vertical_velocity: f32,
    pub current_altitude: f32,
    last_data_packet: Option<IMUDataPacket>,
    velocity_rolling_average: Deque<f32>,
    last_velocity_calculation_packet: Option<IMUDataPacket>,
}

pub struct ProcessorDataPacket {
    pub current_altitude: f32,
    pub vertical_velocity: f32,
    pub maximum_altitude: f32,
    pub maximum_velocity: f32,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            current_altitude: 0.0,
            velocity_rolling_average: Deque::new(VELOCITY_FROM_ALTITUDE_WINDOW_SIZE),
            max_altitude: 0.0,
            max_velocity: 0.0,
            vertical_velocity: 0.0,
            last_data_packet: None,
            last_velocity_calculation_packet: None,
        }
    }

    /// Initializes the data processor with the first IMU data packet.
    pub fn first_update(&mut self, data_packet: &IMUDataPacket) {
        self.current_altitude = data_packet.pressure_alt;
        self.max_altitude = data_packet.pressure_alt;
        self.max_velocity = 0.0;
        self.last_data_packet = Some(data_packet.clone());
        self.last_velocity_calculation_packet = Some(data_packet.clone());
        self.velocity_rolling_average.clear();
    }

    pub fn update(&mut self, data_packet: &IMUDataPacket) {
        self.current_altitude = data_packet.pressure_alt;
        self.max_altitude = self.max_altitude.max(data_packet.pressure_alt);

        self.vertical_velocity = self.calculate_velocity_from_altitude(data_packet);
        self.max_velocity = self.max_velocity.max(self.vertical_velocity);

        self.last_data_packet = Some(data_packet.clone());
    }

    pub fn get_processor_data_packet(&self) -> ProcessorDataPacket {
        ProcessorDataPacket {
            current_altitude: self.current_altitude,
            vertical_velocity: self.vertical_velocity,
            maximum_altitude: self.max_altitude,
            maximum_velocity: self.max_velocity,
        }
    }

    fn calculate_velocity_from_altitude(&mut self, data_packet: &IMUDataPacket) -> f32 {
        let last_altitude = self
            .last_velocity_calculation_packet
            .as_ref()
            .unwrap()
            .pressure_alt;
        let altitude_diff = data_packet.pressure_alt - last_altitude;
        let velocity: f32;

        if altitude_diff.abs() > ALTITUDE_DEADBAND_METERS {
            let time_diff = data_packet.timestamp
                - self
                    .last_velocity_calculation_packet
                    .as_ref()
                    .unwrap()
                    .timestamp;
            velocity = altitude_diff / time_diff.as_secs_f32();
            self.last_velocity_calculation_packet = Some(data_packet.clone());
        } else {
            velocity = self.vertical_velocity;
        }

        self.velocity_rolling_average.push_back(velocity);
        self.velocity_rolling_average.iter().sum::<f32>()
            / self.velocity_rolling_average.len() as f32
    }
}
