use crate::{
    data_processor::{DataProcessor, ProcessorDataPacket},
    imu::{IMU, IMUDataPacket},
    logger::Logger,
    state::{CountdownState, RocketState, StandbyState},
    transmitter::{Transmitter, TransmitterDataPacket},
};

pub struct Context {
    pub state: RocketState,
    pub data_processor: DataProcessor,
    pub imu: IMU,
    pub transmitter: Option<Transmitter>,
    pub logger: Logger,
}

impl Context {
    pub fn new() -> Self {
        Context {
            state: RocketState::Standby(StandbyState {}),
            data_processor: DataProcessor::new(),
            imu: IMU::new(),
            transmitter: None,
            // transmitter: Transmitter::new("/dev/serial0"),
            logger: Logger::new(),
        }
    }

    pub fn update(&mut self) {
        let imu_data_packet = self.imu.get_data_packet();

        self.data_processor.update(&imu_data_packet);
        // Get the processed data packets:
        let processor_data_packet = self.data_processor.get_processor_data_packet();

        // Update the state with the new data
        self.state.update_internal(&processor_data_packet);

        if let Some(new_state) = self.state.should_transition(self) {
            self.state = new_state;
        }
        // Transmit data:
        // let transmitter_data_packet =
        //     self.prepare_transmitter_data_packet(&imu_data_packet, &processor_data_packet);
        // self.transmitter.transmit(&transmitter_data_packet);

        // Match state name to a single character for logging:
        let state_char = match self.state.name() {
            "Standby" => 'S',
            "Countdown" => 'C',
            "MotorBurn" => 'M',
            "Coast" => 'O',
            "FreeFall" => 'F',
            "Landed" => 'L',
            "Shutdown" => 'X',
            _ => 'U', // Unknown
        };

        // Log data
        self.logger
            .log_packets(&imu_data_packet, &processor_data_packet, &state_char);

        println!("Pressure alt: {} m", imu_data_packet.pressure_alt);
        println!("Current Velocity: {} m/s", processor_data_packet.vertical_velocity);
        println!("Max Velocity: {} m/s", processor_data_packet.maximum_velocity);
        println!("");
    }

    pub fn start_camera_recording(&self) {
        // Logic to start camera recording
    }

    pub fn stop_camera_recording(&self) {
        // Logic to stop camera recording
    }

    fn prepare_transmitter_data_packet(
        &self,
        imu_data_packet: &IMUDataPacket,
        processor_data_packet: &ProcessorDataPacket,
    ) -> TransmitterDataPacket {
        TransmitterDataPacket {
            state_name: self.state.name(),
            alt: imu_data_packet.pressure_alt,
            vel: processor_data_packet.vertical_velocity,
            max_alt: processor_data_packet.maximum_altitude,
            temp: imu_data_packet.temperature,
            orientation: TransmitterDataPacket::quaternion_to_euler(imu_data_packet.quaternion),
        }
    }

    // / Waits for the "SALT BOOT" command from the transmitter, so we can start the hot loop.
    // / The main loop will call this function in a loop until it returns true.
    // pub fn wait_for_boot_command(&mut self) -> bool {
    //     match self.transmitter.read() {
    //         Ok(data) if data == "SALT BOOT" => {
    //             println!("Received SALT BOOT command, transitioning to Countdown state.");
    //             self.start_camera_recording();
    //             self.state = RocketState::Countdown(CountdownState {});
    //             true
    //         }
    //         Ok(data) if data == "wait" => {
    //             println!("waiting for start command");
    //             false
    //         }

    //         Ok(data) => {
    //             println!(
    //                 "Unknown command received: {}. Staying in Standby state.",
    //                 data
    //             );
    //             false
    //         }

    //         Err(_) => {
    //             eprintln!("Failed to read from transmitter, starting countdown anyway.");
    //             self.start_camera_recording();
    //             self.state = RocketState::Countdown(CountdownState {});
    //             true
    //         }
    //     }
    // }
}
