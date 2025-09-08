import usb1
import asyncio
import csv
import sys
import struct
import translate_commands as translate

# ==== Device constants ====
VENDOR_ID = 0xC0DE
PRODUCT_ID = 0xCAFE
INTERFACE_NUMBER = 0
ENDPOINT_IN = 0x81
ENDPOINT_OUT = 0x01
PACKET_SIZE = 64 # Max packet size for full speed USB, is what we are limited to according to google


# Setup class to translate commands

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
                # Unpack: <H Q I BB (LSB first, switch the < for MSB first)
                trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger = struct.unpack("<HQIBB", sample)
                csv_writer.writerow([trigger_id, trigger_clk, trigger_data, veto_in, internal_trigger])
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
        encoded_cmd = translate.translate(*cmd.split()) # Splitting on spaces, first part is command name, rest are args
        await loop.run_in_executor(None, lambda: dev_handle.bulkWrite(ENDPOINT_OUT, bytes(encoded_cmd), timeout=1000)) # Should encode as UTF-8 bytes, which should work for the pico?
        print("Sent:", cmd, "as", encoded_cmd) # Confirmation because it is nice to have confirmation 

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
        csv_writer.writerow(["trigger_id", "trigger_clk", "trigger_data", "veto_in", "internal_trigger"])
        await asyncio.gather(
            read_loop(handle, csv_writer),
            write_loop(handle) # Passing handle on should be no problem since they run concurrently and not in parallel (Sending the actual command is done through the executor thread)
        )

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Exiting...")