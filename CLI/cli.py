import usb1
import asyncio
import csv
import sys
import struct
import serial
import translate_commands

import time
import serial

# Allowed command names : switch filename to <filename> , exit, (see translate_commands.py for other commands)

# ==== Device constants ====
VENDOR_ID = 0xC0DE
PRODUCT_ID = 0xCAFE
INTERFACE_NUMBER = 0
ENDPOINT_IN = 0x81
ENDPOINT_OUT = 0x01
PACKET_SIZE = 64 # Max packet size for full speed USB, is what we are limited to according to google

filename = "pico_data.csv"
tc = translate_commands.Translate()

ser = serial.Serial('COM8', 115200, timeout=1)
time.sleep(2)  # allow device to reset

ser.write(b"$X\r\n")
print(ser.readline().decode('utf-8').strip())



async def read_loop(dev_handle, csv_writer):
    '''Reads data from the device and writes to CSV file, should be running continuously.'''
    loop = asyncio.get_running_loop()
    while True:
        future = loop.run_in_executor(None, lambda: dev_handle.bulkRead(ENDPOINT_IN, PACKET_SIZE, timeout=1000)) # Slight chance of missing the first data packet that comes first if timeout happens exactly at the time that packet arrives
        try:
            data = await future
            print("Read:", data)
            # Process each DAQSample, according to Alex rust data struct (16 bytes)
            for i in range(0, len(data), 16):
                sample = data[i:i+16]
                if len(sample) < 16:
                    continue
                #                 // [
                # //  u8 for start (0xFF),
                # //  2 u8's for trigger_id (glue them together left to right),
                # //  8 u8's for trigger_clk (glue them together left to right),
                # //  4 u8's for trigger_data (glue them together from left to right),
                # //  u8 for veto_in, internal_trigger, end_confirmation (
                # //  msb is veto_in, 2nd_msb is internal_trigger, 6 lsb is end confirmation 0x3F
                # //  )
                # //  ]
                # Unpack: <H Q I BB (LSB first, switch the < for MSB first)
                trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger = struct.unpack(" <HQIBB", sample)
                csv_writer.writerow([trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger]) # add 
        except usb1.USBErrorTimeout: # If timeout happens, just ignore it :)
            pass

async def write_loop(dev_handle):
    '''Should enable user to write commands to the device at the same time that it reads the data'''
    loop = asyncio.get_running_loop()
    print("Enter commands to send to Pico (Ctrl+C to exit):")
    while True:
        cmd = await loop.run_in_executor(None, sys.stdin.readline) # Normal python input() doesn't work in async code because it blocks the event loop
        if not cmd:
            continue
        cmd = cmd.strip() # If need be add more filtering here 
        if cmd.lower() == "exit":
            break
        elif "switch file to" in cmd: # Change filename manually
            filename = cmd.split("switch filename to")[-1].strip()
            print(f"Switching output file to: {filename}")
        elif "gcode:" in cmd: # Send gcode command to serial device
            ser.write((cmd + '\r\n').encode('utf-8'))
            response = ser.readline().decode('utf-8').strip()
            print(f"Response: {response}")
            # Change machine angle depending on GCODE here
        else:
            await loop.run_in_executor(None, lambda: dev_handle.bulkWrite(ENDPOINT_OUT, tc.translate(cmd), timeout=1000)) # Should encode as UTF-8 bytes, which should work for the pico?
            print("Sent:", cmd) # Confirmation because it is nice to have confirmation 

async def main():
    with usb1.USBContext() as ctx, open("pico_data.csv", "w", newline="") as csvfile: # Each run will create a fresh file for now, did not fix that since its supposed to become Hdf5 anyways
        handle = ctx.openByVendorIDAndProductID(
            VENDOR_ID, PRODUCT_ID,
            skip_on_error=True # Will return None if device is not found
        )
        if handle is None:
            raise RuntimeError("Device not found")
        handle.claimInterface(INTERFACE_NUMBER) # Claiming in main because if both read and write loops try to claim the interface, it will fail
        csv_writer = csv.writer(csvfile)
        csv_writer.writerow(["trigger_id", "trigger_clk", "trigger_data", "veto_in", "internal_trigger", "pitch_angle", "yaw_angle"])
        await asyncio.gather(
            read_loop(handle, csv_writer),
            write_loop(handle) # Passing handle on should be no problem since they run concurrently and not in parallel (Sending the actual command is done through the executor thread)
        )

if __name__ == "__main__":   
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting...")
