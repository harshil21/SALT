import time
import digitalio
import struct

import queue
import multiprocessing
import board
import threading
import adafruit_bmp280
import adafruit_bno055
import serial
import os

i2c = board.I2C()
# bmp_cs = digitalio.DigitalInOut(board.D10)
bmp280 = adafruit_bmp280.Adafruit_BMP280_I2C(i2c)
#bno = adafruit_bno055.BNO055_I2C(i2c)


def send_data(q):
    with serial.Serial("/dev/serial0", 9600, timeout=1) as ser:
        while True:
            data = q.get()
            sent_data = str(data).encode("utf-8")
            print("sending data")
            ser.write(sent_data)
            ser.flush()
            print(f"sent data {sent_data}")

def main():
    print("Hello from sam-rocket!")

    q = multiprocessing.Queue()
    thread = multiprocessing.Process(target=send_data, args=(q,))
    thread.start()
    iter = 1

    while True:
        # print(f"Temperature: {bmp280.temperature}")
        # print(f"Gravity: {bno.gravity}")
        try:
            # accel = bno.acceleration
            temp = bmp280.temperature
        except Exception:  # i2c erros
            print("I2c read error")
            time.sleep(0.1)
            continue

        # temp = bmp280.temperature
        print(f"Iter: {iter}, Accel: {temp}")
        q.put((temp, iter))
        iter += 1 
        time.sleep(0.0)


if __name__ == "__main__":
    main()
