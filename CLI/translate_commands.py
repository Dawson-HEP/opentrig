# translate.py

class Translate:
    def __init__(self):
        # ---------- argument mappers ----------
        self.DAC_IDS = {"dac0": 10, "dac1": 20, "dac2": 30, "dac3": 40, "dac4": 50, "dac5": 60}
        self.CHANNELS = {"A": 11, "B": 21, "C": 31, "D": 41}
        self.VREF_MODES = {"external": 12, "internal": 22}
        self.GAIN_MODES = {"x1": 13, "x2": 23}
        self.POWER_DOWN_MODES = {"normal": 14, "1k": 24, "100k": 34, "500k": 44}
        self.IS_ALL = {"identical": 100, "individual": 200}

        def as_dac(x): return self.DAC_IDS[x]
        def as_channel(x): return self.CHANNELS[x]
        def as_vref(x): return self.VREF_MODES[x]
        def as_gain(x): return self.GAIN_MODES[x]
        def as_power(x): return self.POWER_DOWN_MODES[x]
        def as_isall(x): return self.IS_ALL[x]

        def as_voltage_u16(x):
            val = int(x)
            return [(val >> 8) & 0xFF, val & 0xFF]  # MSB first

        def as_voltage_u16_list(lst):
            """Maps a list of voltage strings to bytes (for individual mode)."""
            result = []
            for val in lst:
                result.extend(as_voltage_u16(val))
            return result

        # save mappers
        self.mappers = {
            "dac": as_dac,
            "channel": as_channel,
            "vref": as_vref,
            "gain": as_gain,
            "power": as_power,
            "isall": as_isall,
            "voltage": as_voltage_u16,
        }

        # ---------- command registry ----------
        self.command_registry = {
            "set_voltage":            (1, [as_dac, as_channel, as_voltage_u16]),
            "set_vref_mode":          (2, [as_dac, as_channel, as_vref]),
            "set_gain_mode":          (3, [as_dac, as_channel, as_gain]),
            "set_power_down_mode":    (4, [as_dac, as_channel, as_power]),
            "set_all_voltages":       (5, [as_isall, as_voltage_u16_list]),  # handle individual mode
            "set_all_vref_modes":     (6, [as_isall, lambda lst: [as_vref(x) for x in lst]]),
            "set_all_gain_modes":     (7, [as_isall, lambda lst: [as_gain(x) for x in lst]]),
            "set_all_power_down_modes": (8, [as_isall, lambda lst: [as_power(x) for x in lst]]),
        }

    def translate(self, command_name, *args):
        if command_name not in self.command_registry:
            raise ValueError(f"Command '{command_name}' not found in registry.")

        fn_id, signature = self.command_registry[command_name]
        encoded_args = []

        # Check if this is a "set_all" command (needs variable-length handling)
        is_set_all = command_name.startswith("set_all_")

        for i, mapper in enumerate(signature):
            if is_set_all and i == len(signature) - 1:
                # last mapper consumes remaining args
                mapped = mapper(args[i:])
            else:
                mapped = mapper(args[i])

            if isinstance(mapped, list):
                encoded_args.extend(mapped)
            else:
                encoded_args.append(mapped)

        return [0xFF, fn_id] + encoded_args
