# opentrig

> ***open***-source ***trig***ger system

An open-source, integrated, particle physics data acquisition system (DAQ).

Designed to interface with timing systems controlled by the [AIDA-2020 TLU](https://gitlab.com/ohwr/project/fmc-mtlu).
Up to 24 digital trigger channels with variable-gain threshold setting.


![Main picture of opentrig](docs/res/ortho_front.jpg)


## Features

- 144 MHz internal FPGA clock, and timing resolution
- 24 digital trigger channels with variable gain
- Input voltage range 3.3V-5V
- TTL level compatible clock, trigger, input and output
- [AIDA-2020 TLU synchronous AIDA mode](https://gitlab.com/ohwr/project/fmc-mtlu/-/raw/master/Documentation/Main_TLU.pdf?ref_type=heads) (page 31) compatible FPGA


## Hardware Overview

[**Schematics can be found here.**](hardware/digital.pdf)

Front panel IO (left)
- Input clock signal (<40 MHz square wave)
- Output clock signal (120 MHz square wave, 3.3V TTL, <0.5% jitter, phase-locked)
- Input/output trigger (3.3V TTL)
- Input/output veto (3.3V TTL)
- Extra 50R impedance-matched inputs

Back panel (right)
- 24 channel digital inputs, with variable trigger thresholds (1.024 mV steps):
    - 0.000V-2.000V (1x gain)
    - 0.000V-4.000V (2x gain)

Internal components
- Low drift 5.000V reference (TI REF5050AIR)
- 12 high-speed thresholding comparators (TI TLV3502)
- 120 MHz MCU (RP2040)
- FPGA (Lattice ICE40 HX4K)

<p align="center">
  <img src="docs/res/panel_front.jpg" alt="Front panel" width="49%"/>
  <img src="docs/res/panel_back.jpg" alt="Back panel" width="49%"/>
</p>

## Directory Structure

The hardware is designed in [KiCad](https://www.kicad.org/) for the digital trigger front-end and mechanical layout, while the firmware and control utilities are written in [Rust](https://rust-lang.org/) using the [Embassy](https://embassy.dev/) embedded framework. The FPGA subsystems are written in pure Verilog with the [Yosys](https://github.com/YosysHQ/yosys.git) open-source toolchain, and [Project Icestorm](https://github.com/YosysHQ/icestorm)'s bitstream documentation.

```
opentrig/
├── hardware/ # KiCad hardware design
│   ├── fab/ # Fabrication files and gerbers
│   ├── logos.pretty/ # Custom logo footprints and graphics
│   └── digital.pretty/ # Custom digital footprints and component libraries
│
├── opentrig/ # Core DAQ firmware
│   ├── src/ # Source code (Rust + Embassy)
│   │   └── fpga/ # FPGA interface and logic modules
│   └── target/ # Compiled firmware output and build artifacts
│
├── cli/ # Command-line interface for control and configuration via host computer
│   ├── src/ # Rust source for CLI utilities
│   └── target/ # Compiled CLI binaries
│
├── tests/ # Automated and hardware-in-the-loop test framework
│   ├── src/ # Unit and integration test sources
│   └── target/ # Compiled test binaries and reports
│
└── README.md # Project overview, setup, and usage instructions
```

## Building

Before building the firmware and FPGA bitstream, you need to install a few dependencies for Rust and the FPGA toolchain. Follow these steps:

- Install [Rust](https://www.rust-lang.org/) – the firmware is written in Rust using the Embassy framework.
- Install [Just](https://github.com/casey/just) – used to run the build commands.
- Install [Yosys](https://github.com/YosysHQ/yosys) – for Verilog synthesis of the FPGA.
- Install [Project Icestorm](https://github.com/YosysHQ/icestorm) tools – for routing and bitstream generation for iCE40 FPGAs.
- Optional: [KiCad](https://www.kicad.org/) – for viewing and editing hardware schematics.

Once dependencies are installed, clone the repository, navigate to the firmware folder, and run the build commands:

```bash
# Clone the Protovolt repository from GitHub
git clone https://github.com/Dawson-HEP/opentrig.git

# Go to firmware directory
cd opentrig/opentrig
```

```bash
# Compile verilog bitstream
just build-verilog

# Compile embedded firmware
just build-rust
```

Flashing:
- SWD debug port on the PCB
- USB flashing via data port

## Gallery

![Frontend](docs/res/ortho_back.jpg)
![Frontend](docs/res/frontend.jpg)
![Frontend](docs/res/side_back.jpg)
![Bare PCB](docs/res/bare_pcb.jpg)


## License

This project is open-source under the Eclipse Public License - v 2.0.

## Authors

- [Tian Yi, Xia](https://github.com/ThatAquarel), xtxiatianyi@gmail.com
- [Leandro Perez-Moran](https://github.com/LudioRex), pemle2007@gmail.com
