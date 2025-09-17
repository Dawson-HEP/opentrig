import usb1
import asyncio
import csv
import sys
import struct
import translate_commands
import os
import time
import serial
import datetime

# Allowed command names : switch filename to <filename> , exit, (see translate_commands.py for other commands)

# ==== Device constants ====
VENDOR_ID = 0xC0DE
PRODUCT_ID = 0xCAFE
INTERFACE_NUMBER = 0
ENDPOINT_IN = 0x81
ENDPOINT_OUT = 0x01
PACKET_SIZE = 64  # Max packet size for full speed USB, is what we are limited to according to google

filename = "pico_data.csv"
tc = translate_commands.Translate()

#ser = serial.Serial('/dev/ttyUSB0', 115200, timeout=1)
#time.sleep(2)  # allow device to reset

#ser.write(b"$X\r\n")
#print(ser.readline().decode('utf-8').strip())

active_writers = []
async def read_loop(dev_handle):
    '''Reads data from the device and writes to all active CSV files.'''
    loop = asyncio.get_running_loop()
    while True:
        future = loop.run_in_executor(None, lambda: dev_handle.bulkRead(ENDPOINT_IN, PACKET_SIZE, timeout=1000))
        try:
            data = await future
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
        
                if sample[0] != 0x7e:
                    print("Warning: Invalid start byte:", sample[0])
                    continue
                if sample[15] != 0x7d:
                    print("Warning: Invalid end confirmation bits:", sample[15] & 0b111111)
                    continue

                trigger_id_buf = sample[1:3]
                trigger_clk_buf = sample[3:11]
                trigger_data_buf = sample[11:15]

                trigger_id = struct.unpack(">H", trigger_id_buf)[0]
                trigger_clk = struct.unpack(">Q", trigger_clk_buf)[0]
                data_clk_buf = struct.unpack(">I", trigger_data_buf)[0]  # Big-endian
                trigger_data = data_clk_buf & 0x00FF_FFFF
                veto_in = (data_clk_buf >> 31 & 1) != 0
                internal_trigger = (data_clk_buf >> 30 & 1) != 0

                for writer in list(active_writers):
                    if isinstance(writer, tuple):
                        w, filt = writer
                        keep = filt([trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger])
                        if not keep:
                            active_writers.remove(writer)
                    else:
                        writer.writerow([trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger])


        except usb1.USBErrorTimeout:
            pass

async def write_loop(dev_handle, run_dir):
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

        elif cmd.startswith("rt"):
            try:
                _, fname, seconds = cmd.split()
                seconds = int(seconds)
            except ValueError:
                print("Usage: record <filename> <seconds>")
                continue
            
            file_path = os.path.join(run_dir, fname)
            t = open(file_path, "w", newline="")
            w = csv.writer(t)
            active_writers.append(w)
            
            for writer in list(active_writers):
                if writer is not w:  # only to master (avoid polluting the new file)
                    writer.writerow([f"--- RECORD START: {fname} ({seconds}s) ---"])

            print(f"Recording into {fname} for {seconds} seconds")

            async def stop_later():
                await asyncio.sleep(seconds)
                active_writers.remove(w)
                t.close()
                print(f"Finished recording {fname}")
                for writer in list(active_writers):
                    writer.writerow([f"--- RECORD END: {fname} ({seconds}s) ---"])

            asyncio.create_task(stop_later())
        
        elif cmd.startswith("rc"):
            try:
                _, fname, n_hits = cmd.split()
                n_hits = int(n_hits)
            except ValueError:
                print("Usage: record_n_hits <filename> <n_hits>")
                continue

            file_path = os.path.join(run_dir, fname)
            n = open(file_path, "w", newline="")
            w = csv.writer(n) 
            active_writers.append(w)

            # Notify master log and others
            for writer in list(active_writers):
                if writer is not w:
                    writer.writerow([f"--- RECORD START: {fname} (until {n_hits} hits) ---"])

            print(f"Recording into {fname} until {n_hits} hits")

            hit_count = 0

            def hit_filter(row):
                nonlocal hit_count
                hit_count += 1
                if hit_count >= n_hits:
                    try:
                        active_writers.remove(w)
                    except ValueError:
                        pass  # Already removed
                    n.close()
                    print(f"Finished recording {fname} after {n_hits} hits")
                    for writer in list(active_writers):
                        if not isinstance(writer, tuple):
                            writer.writerow([f"--- RECORD END: {fname} ({n_hits} hits) ---"])
                    return False  # stop writing
                return True  # continue writing1

            # Store the writer as a tuple with a filter
            active_writers.append((w, hit_filter))


        elif "gcode:" in cmd: # Send gcode command to serial device
             #ser.write((cmd + '\r\n').encode('utf-8'))
             #response = ser.readline().decode('utf-8').strip()
             #print(f"Response: {response}")
             # Change machine angle depending on GCODE here
             ...
        else:
            await loop.run_in_executor(None, lambda: dev_handle.bulkWrite(ENDPOINT_OUT, tc.translate(*cmd.split()), timeout=1000)) # Should encode as UTF-8 bytes, which should work for the pico?
            print("Sent:", cmd) # Confirmation because it is nice to have confirmation 

def create_run_directory(base="runs"):
    os.makedirs(base, exist_ok=True)
    
    existing_runs = [d for d in os.listdir(base) if d.startswith("run_")]
    run_nums = []
    for r in existing_runs:
        try:
            run_num = int(r.split("_")[1])
            run_nums.append(run_num)
        except (IndexError, ValueError):
            pass

    next_run = max(run_nums, default=0) + 1
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    run_dir_name = f"run_{next_run:03d}_{timestamp}"
    run_dir_path = os.path.join(base, run_dir_name)
    os.makedirs(run_dir_path)

    return run_dir_path

async def main():
    run_dir = create_run_directory()

    with usb1.USBContext() as ctx:
        handle = ctx.openByVendorIDAndProductID(
            VENDOR_ID, PRODUCT_ID,
            skip_on_error=True
        )
        if handle is None:
            raise RuntimeError("Device not found")
        handle.claimInterface(INTERFACE_NUMBER)

        # Open master log
        master_file_path = os.path.join(run_dir, "pico_data.csv")
        master_file = open(master_file_path, "w", newline="")
        master_writer = csv.writer(master_file)
        master_writer.writerow(["trigger_id", "trigger_clk", "trigger_data", "veto_in", "internal_trigger, data"])
        active_writers.append(master_writer)

        await asyncio.gather(
            read_loop(handle),
            write_loop(handle, run_dir)
        )

if __name__ == "__main__":   
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting...")
