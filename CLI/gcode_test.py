

# # List all available serial ports
# ports = list(serial.tools.list_ports.comports())
# for port in ports:
#     print(port)

import time
import serial

ser = serial.Serial('COM8', 115200, timeout=1)
time.sleep(2)  # allow device to reset

ser.write(b"$X\r\n")
print(ser.readline().decode('utf-8').strip())


while True:
    cmd = input("Enter command: ").strip()
    if not cmd:
        continue
    ser.write((cmd + '\r\n').encode('utf-8'))
    response = ser.readline().decode('utf-8').strip()
    print(f"Response: {response}")
