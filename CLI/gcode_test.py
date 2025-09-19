

# # List all available serial ports
# ports = list(serial.tools.list_ports.comports())
# for port in ports:
#     print(port)

import time
import serial

INITIAL_ANGLE_X = -63
INITIAL_ANGLE_Y = 16.9
SPEED_X = 200
SPEED_Y = 250

ser = serial.Serial('COM8', 115200, timeout=1)
time.sleep(2)  # allow device to reset

ser.write(f'$110 = {SPEED_X}\r\n'.encode('utf-8'))
ser.write(f'$111 = {SPEED_Y}\r\n'.encode('utf-8'))
ser.write(b"$H\r\n")

ser.write('G10 P0 L20 X0 Y0\r\n'.encode('utf-8'))
ser.write(f'G0 X{INITIAL_ANGLE_X} Y{INITIAL_ANGLE_Y}\r\n'.encode('utf-8'))
ser.write('G10 P0 L20 X0 Y0\r\n'.encode('utf-8'))



while True:
    cmd = input("Enter command: ").strip()
    if not cmd:
        continue
    ser.write((cmd + '\r\n').encode('utf-8'))
    response = ser.readline().decode('utf-8').strip()
    print(f"Response: {response}")
