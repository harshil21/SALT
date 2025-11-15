from PyQt6.QtWidgets import QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QLineEdit, QPushButton, QFrame
from PyQt6.QtGui import QFont
from PyQt6.QtCore import QTimer, QThread, pyqtSignal
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
import numpy as np
import serial
import time
import re

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
                        data = self.ser.read(self.ser.in_waiting).decode('utf-8', errors='replace')
                        buffer += data
                        lines = buffer.split('\n')
                        buffer = lines.pop()
                        for line in lines:
                            if line.strip():
                                try:
                                    clean_line = re.sub(r'[^\x20-\x7E]', '', line)
                                    parts = clean_line.strip().split(',')
                                    if len(parts) == 8 and parts[0] == 'C':
                                        state_name = parts[0]
                                        alt = float(parts[1])
                                        vel = float(parts[2])
                                        max_alt = float(parts[3])
                                        temp = float(parts[4])
                                        gyro_x = float(parts[5])
                                        gyro_y = float(parts[6])
                                        gyro_z = float(parts[7])
                                        self.data_received.emit((state_name, alt, vel, max_alt, temp, gyro_x, gyro_y, gyro_z))
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
        self.setStyleSheet("background-color: #2c3e50; color: white;")

        # Initialize variables
        self.state_name = "N/A"
        self.alt = 0.0
        self.vel = 0.0
        self.max_alt = 0.0
        self.temp = 0.0
        self.gyro_x = 0.0
        self.gyro_y = 0.0
        self.gyro_z = 0.0
        self.orientation_roll = 0.0
        self.orientation_pitch = 0.0
        self.orientation_yaw = 0.0
        self.last_update = time.time()
        self.last_data_time = time.time()
        self.fps = 0
        self.display_update_time = 0
        self.start_time = time.time()
        self.time_data = []
        self.pitch_data = []
        self.roll_data = []
        self.yaw_data = []
        self.alt_data = []
        self.last_graph_update = 0

        # Setup UI
        self.setup_ui()

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
        control_frame.setStyleSheet("background-color: #34495e; color: white;")
        control_frame.setMaximumHeight(40)
        control_layout = QHBoxLayout(control_frame)
        port_label = QLabel("Port:")
        port_label.setStyleSheet("color: white;")
        self.port_edit = QLineEdit('COM4')
        self.port_edit.setStyleSheet("color: white; background-color: #2c3e50;")
        baud_label = QLabel("Baud:")
        baud_label.setStyleSheet("color: white;")
        self.baud_edit = QLineEdit('9600')
        self.baud_edit.setStyleSheet("color: white; background-color: #2c3e50;")
        self.connect_button = QPushButton('Connect')
        self.connect_button.setStyleSheet("color: white; background-color: #2c3e50;")
        self.connect_button.clicked.connect(self.toggle_connection)
        self.boot_button = QPushButton('Boot Rocket')
        self.boot_button.setStyleSheet("color: white; background-color: #2c3e50;")
        self.boot_button.clicked.connect(self.boot_rocket)
        self.boot_button.setEnabled(False)
        control_layout.addWidget(port_label)
        control_layout.addWidget(self.port_edit)
        control_layout.addWidget(baud_label)
        control_layout.addWidget(self.baud_edit)
        control_layout.addWidget(self.connect_button)
        control_layout.addWidget(self.boot_button)
        main_layout.addWidget(control_frame)

        # Content layout
        content_layout = QHBoxLayout()

        # Telemetry frame on the left
        telemetry_frame = QFrame()
        telemetry_frame.setStyleSheet("background-color: #34495e; color: white;")
        telemetry_layout = QVBoxLayout(telemetry_frame)
        large_font = QFont()
        large_font.setPointSize(16)

        self.state_label = QLabel("State: N/A")
        self.state_label.setFont(large_font)
        self.state_label.setStyleSheet("color: #e74c3c;")  # Red

        self.alt_label = QLabel("Altitude: 0.00 m")
        self.alt_label.setFont(large_font)
        self.alt_label.setStyleSheet("color: #2ecc71;")  # Green

        self.vel_label = QLabel("Velocity: 0.00 m/s")
        self.vel_label.setFont(large_font)
        self.vel_label.setStyleSheet("color: #3498db;")  # Blue

        self.max_alt_label = QLabel("Max Altitude: 0.00 m")
        self.max_alt_label.setFont(large_font)
        self.max_alt_label.setStyleSheet("color: #f39c12;")  # Orange

        self.temp_label = QLabel("Temperature: 0.0 °C")
        self.temp_label.setFont(large_font)
        self.temp_label.setStyleSheet("color: #9b59b6;")  # Purple

        telemetry_layout.addWidget(self.state_label)
        telemetry_layout.addWidget(self.alt_label)
        telemetry_layout.addWidget(self.vel_label)
        telemetry_layout.addWidget(self.max_alt_label)
        telemetry_layout.addWidget(self.temp_label)
        content_layout.addWidget(telemetry_frame, stretch=1)

        # Graphs on the right
        graphs_panel = QWidget()
        graphs_layout = QVBoxLayout(graphs_panel)

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
        graphs_layout.addWidget(self.canvas_graph)

        # Altitude graph
        self.fig_alt = Figure(figsize=(6, 4), facecolor='#2c3e50', dpi=80)
        self.ax_alt = self.fig_alt.add_subplot(111)
        self.ax_alt.set_facecolor('#2c3e50')
        self.ax_alt.grid(True, color='#7f8c8d', alpha=0.3)
        for spine in self.ax_alt.spines.values():
            spine.set_color('white')
        self.ax_alt.tick_params(colors='white')
        self.ax_alt.set_title('Altitude History', color='white')
        self.ax_alt.set_ylabel('Altitude (m)', color='white')
        self.ax_alt.set_xlabel('Time (s)', color='white')
        self.alt_line, = self.ax_alt.plot([], [], label='Altitude', color='#3498db')
        self.ax_alt.legend(loc='upper right', facecolor='#34495e', edgecolor='#34495e', labelcolor='white')
        self.ax_alt.set_xlim(0, 30)
        self.ax_alt.set_ylim(0, 100)
        self.canvas_alt = FigureCanvas(self.fig_alt)
        graphs_layout.addWidget(self.canvas_alt)

        content_layout.addWidget(graphs_panel, stretch=4)
        main_layout.addLayout(content_layout)

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
        self.state_name, self.alt, self.vel, self.max_alt, self.temp, self.gyro_x, self.gyro_y, self.gyro_z = data
        current_time = time.time()
        dt = current_time - self.last_data_time
        self.last_data_time = current_time
        self.orientation_roll += self.gyro_x * dt
        self.orientation_pitch += self.gyro_y * dt
        self.orientation_yaw += self.gyro_z * dt
        t = current_time - self.start_time
        self.time_data.append(t)
        self.pitch_data.append(self.orientation_pitch)
        self.roll_data.append(self.orientation_roll)
        self.yaw_data.append(self.orientation_yaw)
        self.alt_data.append(self.alt)

    def update_orientation_graph(self):
        current_time = time.time()
        if current_time - self.last_graph_update < 0.033:
            return
        self.last_graph_update = current_time

        if len(self.time_data) > 300:
            keep_all_threshold = self.time_data[-1] - 10 if self.time_data else 0
            old_indices = [i for i, ti in enumerate(self.time_data) if ti < keep_all_threshold]
            if old_indices:
                to_keep = old_indices[::5] + list(range(old_indices[-1] + 1, len(self.time_data)))
                self.time_data = [self.time_data[i] for i in to_keep]
                self.pitch_data = [self.pitch_data[i] for i in to_keep]
                self.roll_data = [self.roll_data[i] for i in to_keep]
                self.yaw_data = [self.yaw_data[i] for i in to_keep]
                self.alt_data = [self.alt_data[i] for i in to_keep]

        self.pitch_line.set_data(self.time_data, self.pitch_data)
        self.roll_line.set_data(self.time_data, self.roll_data)
        self.yaw_line.set_data(self.time_data, self.yaw_data)

        if self.time_data:
            padding = 5.0
            x_min = max(0, self.time_data[-1] - 30)
            x_max = max(30, self.time_data[-1] + padding)
            if abs(self.ax_graph.get_xlim()[0] - x_min) > 0.1 or abs(self.ax_graph.get_xlim()[1] - x_max) > 0.1:
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

    def update_alt_graph(self):
        self.alt_line.set_data(self.time_data, self.alt_data)

        if self.time_data:
            padding = 5.0
            x_min = max(0, self.time_data[-1] - 30)
            x_max = max(30, self.time_data[-1] + padding)
            if abs(self.ax_alt.get_xlim()[0] - x_min) > 0.1 or abs(self.ax_alt.get_xlim()[1] - x_max) > 0.1:
                self.ax_alt.set_xlim(x_min, x_max)

            recent_alt = self.alt_data[-20:] if len(self.alt_data) > 20 else self.alt_data
            y_min = min(recent_alt, default=0)
            y_max = max(recent_alt, default=0)
            margin = max(1, (y_max - y_min) * 0.1)
            y_min_limit = y_min - margin
            y_max_limit = y_max + margin
            if abs(self.ax_alt.get_ylim()[0] - y_min_limit) > 5 or abs(self.ax_alt.get_ylim()[1] - y_max_limit) > 5:
                self.ax_alt.set_ylim(y_min_limit, y_max_limit)

        self.canvas_alt.draw()

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

        self.update_orientation_graph()
        self.update_alt_graph()

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