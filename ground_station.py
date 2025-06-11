from PyQt6.QtWidgets import QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QLineEdit, QPushButton, QFrame, QGridLayout
from PyQt6.QtCore import QTimer, QThread, pyqtSignal
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
import numpy as np
import serial
import time

class SerialThread(QThread):
    data_received = pyqtSignal(tuple)

    def __init__(self, serial_port, baud_rate):
        super().__init__()
        self.serial_port = serial_port
        self.baud_rate = baud_rate
        self.ser = None
        self.running = True
        self.connected = False

    def run(self):
        buffer = ""
        while self.running:
            try:
                if self.connected:
                    if not self.ser or not self.ser.is_open:
                        try:
                            self.ser = serial.Serial(self.serial_port, self.baud_rate, timeout=1)
                        except Exception as e:
                            print(f"Failed to open serial port: {e}")
                            time.sleep(1)
                            continue
                    if self.ser.in_waiting > 0:
                        data = self.ser.read(self.ser.in_waiting).decode('utf-8')
                        buffer += data
                        lines = buffer.split('\n')
                        buffer = lines.pop()
                        for line in lines:
                            if line.strip():
                                try:
                                    parts = line.strip().split(',')
                                    if len(parts) == 8:
                                        state_name = parts[0]
                                        alt = float(parts[1])
                                        vel = float(parts[2])
                                        max_alt = float(parts[3])
                                        temp = float(parts[4])
                                        roll = float(parts[5])
                                        pitch = float(parts[6])
                                        yaw = float(parts[7])
                                        self.data_received.emit((state_name, alt, vel, max_alt, temp, roll, pitch, yaw))
                                except (ValueError, IndexError) as e:
                                    print(f"Parse error: {e} on line: {line}")
                else:
                    if self.ser and self.ser.is_open:
                        self.ser.close()
                        self.ser = None
                    time.sleep(0.1)
            except Exception as e:
                print(f"Serial thread error: {e}")
            time.sleep(0.01)

class RocketGroundStation(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Rocket Ground Station")
        self.setGeometry(100, 100, 1200, 800)
        self.setStyleSheet("background-color: #2c3e50;")

        # Initialize variables
        self.state_name = "N/A"
        self.alt = 0.0
        self.vel = 0.0
        self.max_alt = 0.0
        self.temp = 0.0
        self.roll = 0.0
        self.pitch = 0.0
        self.yaw = 0.0
        self.horizon_angle = 0.0
        self.last_update = time.time()
        self.fps = 0
        self.display_update_time = 0
        self.start_time = time.time()
        self.time_data = []
        self.pitch_data = []
        self.roll_data = []
        self.yaw_data = []
        self.last_graph_update = 0
        self.last_visualization_update = 0

        # Setup UI
        self.setup_ui()
        self.precompute_rocket_model()
        self.setup_3d_visualization()

        # Start serial thread
        self.serial_thread = SerialThread('COM4', 9600)
        self.serial_thread.data_received.connect(self.on_data_received)
        self.serial_thread.start()

        # Setup timer
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_ui)
        self.timer.start(33)  # ~30 FPS

    def setup_ui(self):
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)

        # Control frame
        control_frame = QFrame()
        control_frame.setStyleSheet("background-color: #34495e;")
        control_layout = QHBoxLayout(control_frame)
        self.port_edit = QLineEdit('COM4')
        self.baud_edit = QLineEdit('9600')
        self.connect_button = QPushButton('Connect')
        self.connect_button.clicked.connect(self.toggle_connection)
        self.boot_button = QPushButton('Boot Rocket')
        self.boot_button.clicked.connect(self.boot_rocket)
        self.boot_button.setEnabled(False)
        control_layout.addWidget(QLabel("Port:"))
        control_layout.addWidget(self.port_edit)
        control_layout.addWidget(QLabel("Baud:"))
        control_layout.addWidget(self.baud_edit)
        control_layout.addWidget(self.connect_button)
        control_layout.addWidget(self.boot_button)
        main_layout.addWidget(control_frame)

        # Content layout
        content_layout = QHBoxLayout()

        # Left panel (3D visualization)
        left_panel = QWidget()
        left_layout = QVBoxLayout(left_panel)
        self.fig_3d = Figure(figsize=(6, 6), facecolor='#2c3e50', dpi=80)
        self.ax_3d = self.fig_3d.add_subplot(111, projection='3d')
        self.ax_3d.set_facecolor('#2c3e50')
        for axis in [self.ax_3d.xaxis, self.ax_3d.yaxis, self.ax_3d.zaxis]:
            axis.line.set_color('white')
            axis.label.set_color('white')
            axis.set_pane_color((0.1, 0.1, 0.1, 0.1))
        self.ax_3d.tick_params(colors='white')
        self.ax_3d.set_xlim([-1, 1])
        self.ax_3d.set_ylim([-1, 1])
        self.ax_3d.set_zlim([-1, 1])
        self.ax_3d.set_xticklabels([])
        self.ax_3d.set_yticklabels([])
        self.ax_3d.set_zticklabels([])
        self.ax_3d.set_xlabel('X', color='white')
        self.ax_3d.set_ylabel('Y', color='white')
        self.ax_3d.set_zlabel('Z', color='white')
        self.canvas_3d = FigureCanvas(self.fig_3d)
        left_layout.addWidget(self.canvas_3d)
        content_layout.addWidget(left_panel)

        # Right panel
        right_panel = QWidget()
        right_layout = QVBoxLayout(right_panel)

        # Telemetry frame
        telemetry_frame = QFrame()
        telemetry_frame.setStyleSheet("background-color: #34495e;")
        telemetry_layout = QGridLayout(telemetry_frame)
        self.state_label = QLabel("State: N/A")
        self.alt_label = QLabel("Altitude: 0.00 m")
        self.vel_label = QLabel("Velocity: 0.00 m/s")
        self.max_alt_label = QLabel("Max Altitude: 0.00 m")
        self.temp_label = QLabel("Temperature: 0.0 °C")
        self.horizon_label = QLabel("Horizon Angle: 0.0°")
        telemetry_layout.addWidget(self.state_label, 0, 0)
        telemetry_layout.addWidget(self.alt_label, 0, 1)
        telemetry_layout.addWidget(self.vel_label, 1, 0)
        telemetry_layout.addWidget(self.max_alt_label, 1, 1)
        telemetry_layout.addWidget(self.temp_label, 2, 0)
        telemetry_layout.addWidget(self.horizon_label, 2, 1)
        right_layout.addWidget(telemetry_frame)

        # Orientation graph
        self.fig_graph = Figure(figsize=(6, 4), facecolor='#2c3e50', dpi=80)
        self.ax_graph = self.fig_graph.add_subplot(111)
        self.ax_graph.set_facecolor('#2c3e50')
        self.ax_graph.grid(True, color='#7f8c8d', alpha=0.3)
        for spine in self.ax_graph.spines.values():
            spine.set_color('white')
        self.ax_graph.tick_params(colors='white')
        self.ax_graph.set_title('Orientation History', color='white')
        self.ax_graph.set_ylabel('Angle (degrees)', color='white')
        self.ax_graph.set_xlabel('Time (s)', color='white')
        self.pitch_line, = self.ax_graph.plot([], [], label='Pitch', color='#2ecc71')
        self.roll_line, = self.ax_graph.plot([], [], label='Roll', color='#e74c3c')
        self.yaw_line, = self.ax_graph.plot([], [], label='Yaw', color='#f39c12')
        self.ax_graph.legend(loc='upper right', facecolor='#34495e', edgecolor='#34495e', labelcolor='white')
        self.ax_graph.set_xlim(0, 30)
        self.ax_graph.set_ylim(-90, 90)
        self.canvas_graph = FigureCanvas(self.fig_graph)
        right_layout.addWidget(self.canvas_graph)

        content_layout.addWidget(right_panel)
        main_layout.addLayout(content_layout)

    def precompute_rocket_model(self):
        rocket_length = 0.7
        rocket_width = 0.1
        body_length = rocket_length * 0.7
        nose_length = rocket_length * 0.3

        theta = np.linspace(0, 2 * np.pi, 12)
        z_body = np.linspace(-body_length / 2, body_length / 2, 6)
        theta_grid, z_grid = np.meshgrid(theta, z_body)
        self.x_body = rocket_width * np.cos(theta_grid)
        self.y_body = rocket_width * np.sin(theta_grid)
        self.z_body = z_grid

        z_nose = np.linspace(body_length / 2, body_length / 2 + nose_length, 5)
        z_factor = (z_nose - body_length / 2) / nose_length
        self.x_nose = np.outer(1 - z_factor, rocket_width * np.cos(theta))
        self.y_nose = np.outer(1 - z_factor, rocket_width * np.sin(theta))
        self.z_nose = np.broadcast_to(z_nose[:, np.newaxis], self.x_nose.shape)

        self.axis_length = 0.3
        self.x_axis = np.array([self.axis_length, 0, 0])
        self.y_axis = np.array([0, self.axis_length, 0])
        self.z_axis = np.array([0, 0, self.axis_length])
        self.rocket_direction = np.array([0, 0, 1.0])

    def setup_3d_visualization(self):
        self.body_scatter = self.ax_3d.scatter([], [], [], color='silver', alpha=0.3, s=10)
        self.nose_scatter = self.ax_3d.scatter([], [], [], color='red', alpha=0.7, s=10)
        self.x_axis_line, = self.ax_3d.plot([], [], [], color='red', linewidth=2)
        self.y_axis_line, = self.ax_3d.plot([], [], [], color='green', linewidth=2)
        self.z_axis_line, = self.ax_3d.plot([], [], [], color='blue', linewidth=2)
        self.rocket_axis_line, = self.ax_3d.plot([], [], [], color='yellow', linewidth=3, linestyle='--')

    def toggle_connection(self):
        if not self.serial_thread.connected:
            self.serial_thread.serial_port = self.port_edit.text()
            self.serial_thread.baud_rate = int(self.baud_edit.text())
            self.serial_thread.connected = True
            self.connect_button.setText("Disconnect")
            self.boot_button.setEnabled(True)
        else:
            self.serial_thread.connected = False
            self.connect_button.setText("Connect")
            self.boot_button.setEnabled(False)

    def boot_rocket(self):
        if self.serial_thread.connected and self.serial_thread.ser and self.serial_thread.ser.is_open:
            try:
                self.serial_thread.ser.write("SALT BOOT\n".encode('utf-8'))
                print("Sent boot command")
            except Exception as e:
                print(f"Failed to send boot command: {e}")
        else:
            print("Cannot boot: not connected to serial port")

    def on_data_received(self, data):
        self.state_name, self.alt, self.vel, self.max_alt, self.temp, self.roll, self.pitch, self.yaw = data

    def update_rocket_visualization(self):
        current_time = time.time()
        if current_time - self.last_visualization_update < 0.1:
            return
        self.last_visualization_update = current_time

        yaw_rad = np.radians(self.yaw)
        pitch_rad = np.radians(self.pitch)
        roll_rad = np.radians(self.roll)

        yaw_matrix = np.array([
            [np.cos(yaw_rad), -np.sin(yaw_rad), 0],
            [np.sin(yaw_rad), np.cos(yaw_rad), 0],
            [0, 0, 1]
        ])
        pitch_matrix = np.array([
            [np.cos(pitch_rad), 0, np.sin(pitch_rad)],
            [0, 1, 0],
            [-np.sin(pitch_rad), 0, np.cos(pitch_rad)]
        ])
        roll_matrix = np.array([
            [1, 0, 0],
            [0, np.cos(roll_rad), -np.sin(roll_rad)],
            [0, np.sin(roll_rad), np.cos(roll_rad)]
        ])
        rotation_matrix = yaw_matrix @ pitch_matrix @ roll_matrix

        xyz_body = np.vstack([self.x_body.flatten(), self.y_body.flatten(), self.z_body.flatten()])
        rotated_body = rotation_matrix @ xyz_body
        xyz_nose = np.vstack([self.x_nose.flatten(), self.y_nose.flatten(), self.z_nose.flatten()])
        rotated_nose = rotation_matrix @ xyz_nose

        self.body_scatter._offsets3d = (rotated_body[0, :], rotated_body[1, :], rotated_body[2, :])
        self.nose_scatter._offsets3d = (rotated_nose[0, :], rotated_nose[1, :], rotated_nose[2, :])

        rotated_x = rotation_matrix @ self.x_axis
        rotated_y = rotation_matrix @ self.y_axis
        rotated_z = rotation_matrix @ self.z_axis
        self.x_axis_line.set_data_3d([0, rotated_x[0]], [0, rotated_x[1]], [0, rotated_x[2]])
        self.y_axis_line.set_data_3d([0, rotated_y[0]], [0, rotated_y[1]], [0, rotated_y[2]])
        self.z_axis_line.set_data_3d([0, rotated_z[0]], [0, rotated_z[1]], [0, rotated_z[2]])

        rotated_direction = rotation_matrix @ self.rocket_direction
        rocket_length = 0.9
        self.rocket_axis_line.set_data_3d([0, rotated_direction[0] * rocket_length], 
                                         [0, rotated_direction[1] * rocket_length],
                                         [0, rotated_direction[2] * rocket_length])

        horizon_reference = np.array([0, 0, 1])
        dot_product = np.dot(rotated_direction, horizon_reference)
        angle_rad = np.arccos(np.clip(dot_product / np.linalg.norm(rotated_direction), -1.0, 1.0))
        self.horizon_angle = 90 - np.degrees(angle_rad)

        self.canvas_3d.draw()

    def update_orientation_graph(self):
        current_time = time.time()
        if current_time - self.last_graph_update < 0.2:
            return
        self.last_graph_update = current_time

        t = current_time - self.start_time
        self.time_data.append(t)
        self.pitch_data.append(self.pitch)
        self.roll_data.append(self.roll)
        self.yaw_data.append(self.yaw)

        if len(self.time_data) > 300:
            keep_all_threshold = t - 10
            old_indices = [i for i, ti in enumerate(self.time_data) if ti < keep_all_threshold]
            if old_indices:
                to_keep = old_indices[::5] + list(range(old_indices[-1] + 1, len(self.time_data)))
                self.time_data = [self.time_data[i] for i in to_keep]
                self.pitch_data = [self.pitch_data[i] for i in to_keep]
                self.roll_data = [self.roll_data[i] for i in to_keep]
                self.yaw_data = [self.yaw_data[i] for i in to_keep]

        self.pitch_line.set_data(self.time_data, self.pitch_data)
        self.roll_line.set_data(self.time_data, self.roll_data)
        self.yaw_line.set_data(self.time_data, self.yaw_data)

        if self.time_data:
            x_min = max(0, self.time_data[-1] - 30)
            x_max = max(30, self.time_data[-1])
            if abs(self.ax_graph.get_xlim()[0] - x_min) > 2 or abs(self.ax_graph.get_xlim()[1] - x_max) > 2:
                self.ax_graph.set_xlim(x_min, x_max)

            recent_pitch = self.pitch_data[-20:] if len(self.pitch_data) > 20 else self.pitch_data
            recent_roll = self.roll_data[-20:] if len(self.roll_data) > 20 else self.roll_data
            recent_yaw = self.yaw_data[-20:] if len(self.yaw_data) > 20 else self.yaw_data
            y_min = min(min(recent_pitch, default=0), min(recent_roll, default=0), min(recent_yaw, default=0))
            y_max = max(max(recent_pitch, default=0), max(recent_roll, default=0), max(recent_yaw, default=0))
            margin = max(5, (y_max - y_min) * 0.1)
            y_min_limit = y_min - margin
            y_max_limit = y_max + margin
            if abs(self.ax_graph.get_ylim()[0] - y_min_limit) > 5 or abs(self.ax_graph.get_ylim()[1] - y_max_limit) > 5:
                self.ax_graph.set_ylim(y_min_limit, y_max_limit)

        self.canvas_graph.draw()

    def update_ui(self):
        start_time = time.time()
        elapsed = start_time - self.last_update
        self.last_update = start_time
        fps = 1.0 / elapsed if elapsed > 0 else 0
        self.fps = 0.9 * self.fps + 0.1 * fps

        self.state_label.setText(f"State: {self.state_name}")
        self.alt_label.setText(f"Altitude: {self.alt:.2f} m")
        self.vel_label.setText(f"Velocity: {self.vel:.2f} m/s")
        self.max_alt_label.setText(f"Max Altitude: {self.max_alt:.2f} m")
        self.temp_label.setText(f"Temperature: {self.temp:.1f} °C")
        self.horizon_label.setText(f"Horizon Angle: {self.horizon_angle:.1f}°")

        self.update_rocket_visualization()
        self.update_orientation_graph()

        update_time = (time.time() - start_time) * 1000
        self.display_update_time = 0.9 * self.display_update_time + 0.1 * update_time

    def closeEvent(self, event):
        self.serial_thread.running = False
        self.serial_thread.wait()
        event.accept()

if __name__ == "__main__":
    from PyQt6.QtWidgets import QApplication
    import sys
    app = QApplication(sys.argv)
    window = RocketGroundStation()
    window.show()
    sys.exit(app.exec())