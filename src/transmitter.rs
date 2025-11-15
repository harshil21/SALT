use std::error::Error;
use std::io::Read;
use std::io::Write;

use serialport::TTYPort;
pub struct TransmitterDataPacket {
    pub state_name: &'static str,
    pub alt: f32,
    pub vel: f32,
    pub max_alt: f32,
    pub temp: f32,       // Temperature in Celsius
    pub gyro: [f32; 3],  // Gyroscope data in rad/s (x, y, z)
}

pub struct Transmitter {
    port: TTYPort,
    buffer: String,
}

impl Transmitter {
    pub fn new(path: &str) -> Self {
        let port = serialport::new(path, 9600)
            .timeout(std::time::Duration::from_millis(7000))
            .open_native()
            .expect("Failed to open serial port");

        Transmitter { port, buffer: String::new() }
    }

    pub fn transmit(&mut self, data_packet: &TransmitterDataPacket) {
        // TODO: Add the callsign to the output string
        let state_letter = data_packet.state_name.chars().next().unwrap_or('U');
        let output = format!(
            "{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}\n",
            state_letter,

            data_packet.alt,
            data_packet.vel,
            data_packet.max_alt,
            data_packet.temp,
            data_packet.gyro[0],
            data_packet.gyro[1],
            data_packet.gyro[2]
        );

        match self.port.write_all(output.as_bytes()) {
            Ok(_) => (),
            Err(_) => eprintln!("Failed to write to port for transmission"),
        }
    }

    /// Reads data from the serial port and accumulates it until a newline is received.
    pub fn read(&mut self) -> Result<String, Box<dyn Error>> {
        let mut temp_buffer = vec![0; 512];
        match self.port.read(&mut temp_buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    // Append new data to the buffer
                    self.buffer
                        .push_str(&String::from_utf8_lossy(&temp_buffer[..bytes_read]));

                    // Check if we have a complete line (ending with newline)
                    if let Some(newline_pos) = self.buffer.find('\n') {
                        // Extract the complete command
                        let command = self.buffer[..newline_pos].trim().to_string();
                        // Remove the processed command from the buffer
                        self.buffer = self.buffer[newline_pos + 1..].to_string();
                        Ok(command)
                    } else {
                        // No complete command yet, return "wait"
                        Ok(String::from("wait"))
                    }
                } else {
                    // No data read, return "wait"
                    Ok(String::from("wait"))
                }
            }
            Err(_) => {
                // Read error, return "wait"
                Ok(String::from("wait"))
            }
        }
    }
}
