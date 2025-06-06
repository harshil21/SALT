use std::error::Error;
use std::io::Read;
use std::io::Write;

use serialport::TTYPort;
pub struct TransmitterDataPacket {
    pub state_name: &'static str,
    pub alt: f32,
    pub vel: f32,
    pub max_alt: f32,
    pub temp: f32,             // Temperature in Celsius
    pub orientation: [f32; 3], // Roll, Pitch, Yaw in degrees
}

impl TransmitterDataPacket {
    pub fn quaternion_to_euler(quaternion: [f32; 4]) -> [f32; 3] {
        let w = quaternion[0];
        let x = quaternion[1];
        let y = quaternion[2];
        let z = quaternion[3];

        // Convert quaternion to Euler angles (roll, pitch, yaw)
        let roll = (2.0 * (w * x + y * z))
            .atan2(1.0 - 2.0 * (x * x + y * y))
            .to_degrees();
        let pitch = (2.0 * (w * y - z * x)).asin().to_degrees();
        let yaw = (2.0 * (w * z + x * y))
            .atan2(1.0 - 2.0 * (y * y + z * z))
            .to_degrees();

        [roll, pitch, yaw]
    }
}

pub struct Transmitter {
    port: TTYPort,
}

impl Transmitter {
    pub fn new(path: &str) -> Self {
        let port = serialport::new(path, 9600)
            .timeout(std::time::Duration::from_millis(7000))
            .open_native()
            .expect("Failed to open serial port");

        Transmitter { port }
    }

    pub fn transmit(&mut self, data_packet: &TransmitterDataPacket) {
        // TODO: Add the callsign to the output string
        let output = format!(
            "{},{},{},{},{},{},{},{}\n",
            data_packet.state_name,
            data_packet.alt,
            data_packet.vel,
            data_packet.max_alt,
            data_packet.temp,
            data_packet.orientation[0],
            data_packet.orientation[1],
            data_packet.orientation[2]
        );

        match self.port.write_all(output.as_bytes()) {
            Ok(_) => (),
            Err(_) => eprintln!("Failed to write to port for transmission"),
        }
    }

    /// Reads data from the serial port for 7 seconds.
    pub fn read(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = vec![0; 512];
        match self.port.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
                } else {
                    Err("No data read".into())
                }
            }
            Err(_) => match self.port.write(b"Waiting for command") {
                Ok(_) => Ok(String::from("wait")),
                Err(_) => Err("Failed to write to port for health check".into()),
            },
        }
    }
}
