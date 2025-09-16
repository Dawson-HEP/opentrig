//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.
#![allow(warnings)] 
#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use cortex_m::peripheral::nvic;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio::Pin, pac::pio::Irq, pio::IrqFlags, pwm::Pwm, spi, spi::Spi, usb::Bus};
//use embassy_rp::{gpio::Pin, pac::pio::Irq, pio::{Irq, IrqFlags}, pwm::Pwm, spi::Spi};

use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use defmt::info;
use embassy_executor::Executor;
use embassy_rp::interrupt;
use embassy_rp::uart::{self, UartTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex;
use static_cell::StaticCell;




use embassy_futures::join::{join, join3};

use embassy_rp::bind_interrupts;
//use embassy_rp::peripherals::USB;
use embassy_rp::peripherals::*;
//use embassy_rp::usb::InterruptHandler;
use embassy_rp::usb::{Driver, InterruptHandler};

//use embassy_usb::driver::{Driver, Endpoint, EndpointIn, EndpointOut};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::{Builder, Config};

use core::{fmt::Debug, ops::DerefMut, str};

use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{filesystem, sdcard::{DummyCsPin, SdCard}};
use embedded_sdmmc::Mode;



mod data;
mod fpga;
mod dac;
mod handle_inputs;

//use dac::{DacManager, DacError};
use dac::DacManager;
use data::DAQSample;
use fpga::{DAQFpga, daq_fpga_clock_config, daq_fpga_spi_config};


use core::{fmt, ops::Deref, u64};
use heapless::Vec;



static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static DAQ_CHANNEL: Channel<CriticalSectionRawMutex, [u8;16], 1> = Channel::new();
static INPUT_CHANNEL: Channel<CriticalSectionRawMutex, [u8;64], 1> = Channel::new();


// static buffers for embassy-usb for ownership reasons
//     Create embassy-usb DeviceBuilder using the driver and config.
//     It needs some buffers for building the descriptors.
static mut config_descriptor : [u8;256] = [0; 256];
static mut bos_descriptor : [u8;256]  = [0; 256];
static mut msos_descriptor : [u8;256]  = [0; 256];
static mut control_buf : [u8;64]  = [0; 64];




static mut usb_connected : bool = false;



// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});


// necessary for embedded_sdmmc::VolumeManager::new
struct DummyTimesource();
impl embedded_sdmmc::TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}


async fn get_sd_root_dir(pin_spi1:SPI1, pin_10:PIN_10, pin_11:PIN_11, pin_12:PIN_12, pin_8:PIN_8
) -> embedded_sdmmc::VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI1, spi::Blocking>, DummyCsPin, embedded_hal_bus::spi::NoDelay>, Output<'static>, embassy_time::Delay>, DummyTimesource>
//) -> embedded_sdmmc::Directory<'static, SdCard<ExclusiveDevice<Spi<'static, embassy_rp::peripherals::SPI1, spi::Blocking>, DummyCsPin, embedded_hal_bus::spi::NoDelay>, Output<'static>, embassy_time::Delay>, DummyTimesource, 4, 4, 1> 
{
    let mut config = spi::Config::default();
    config.frequency = 400_000;


    // sd_inner : pin SPI1
    // sd_clk : pin 10
    // sd_mosi : pin 11
    // sd_miso : pin 12
    // pin for CS : pin 8 ?

    let spi: Spi<'_, SPI1, spi::Blocking> = Spi::new_blocking(pin_spi1, pin_10, pin_11, pin_12, config);
    // Use a dummy cs pin here, for embedded-hal SpiDevice compatibility reasons
    let spi_dev = ExclusiveDevice::new_no_delay(spi, DummyCsPin);
    // Real cs pin
    let cs = Output::new(pin_8, Level::High);

    let sdcard = SdCard::new(spi_dev, cs, embassy_time::Delay);
    info!("Card size is {} bytes", sdcard.num_bytes().expect("error in getting num_bytes"));

    // Now that the card is initialized, the SPI clock can go faster
    let mut config = spi::Config::default();
    config.frequency = 32_000_000;
    sdcard.spi(|dev| dev.bus_mut().set_config(&config));

    // Now let's look for volumes (also known as partitions) on our block device.
    // To do this we need a Volume Manager. It will take ownership of the block device.
    let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, DummyTimesource());

    // Try and access Volume 0 (i.e. the first partition).
    // The volume object holds information about the filesystem on that volume.
    //let mut volume0 = &volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0)).unwrap();
    //info!("Volume 0: {:?}", defmt::Debug2Format(&volume0));

    // Open the root directory (mutably borrows from the volume).
    //let mut root_dir = volume0.open_root_dir().unwrap();

    volume_mgr
}


async fn get_and_setup_daq(
    PIN_20:PIN_20,   PIN_19:PIN_19,   PIN_18:PIN_18,         SPI0:SPI0,
    DMA_CH0:DMA_CH0, DMA_CH1:DMA_CH1, PWM_SLICE5:PWM_SLICE5, PIN_27:PIN_27,
    PIN_17:PIN_17,   PIN_13:PIN_13,   PIN_14:PIN_14,         PIN_26:PIN_26,
    PIN_15:PIN_15, PIN_16:PIN_16,
) -> DAQFpga<'static, SPI0> {
    let (rx, tx, clk) = (PIN_20, PIN_19, PIN_18);
    let spi_config = daq_fpga_spi_config();
    let spi = Spi::new(SPI0, clk, tx, rx, DMA_CH0, DMA_CH1, spi_config);
    
    let pwm_config = daq_fpga_clock_config();
    let fpga_mcu_clk = Pwm::new_output_b(PWM_SLICE5, PIN_27, pwm_config);
    
    let mut daq: DAQFpga<'_, SPI0> = DAQFpga::new(
        spi,
        PIN_17.degrade(),
        PIN_13.degrade(),
        PIN_14.degrade(),
        fpga_mcu_clk,
        PIN_26.degrade(),
        PIN_15.degrade(),
        PIN_16.degrade(),
    );
    
    daq.configure(include_bytes!("fpga/main.bin"))
        .await
        .unwrap();
    daq.setup_clocks().await.unwrap();
    
    daq.reset().unwrap();

    daq
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let ldacs = [
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        Output::new(p.PIN_5, Level::Low),
        Output::new(p.PIN_6, Level::Low),
        Output::new(p.PIN_7, Level::Low),
    ];

    let mut dac_manager: DacManager<'_> = DacManager::new(p.I2C0, p.PIN_1, p.PIN_0, ldacs);
    dac_manager.init().await.unwrap();
    dac_manager.set_all_voltages([1200; 24]).await.unwrap();

    //let daq = get_and_setup_daq(
    //    p.PIN_20, p.PIN_19, p.PIN_18, p.SPI0, p.DMA_CH0, p.DMA_CH1, p.PWM_SLICE5,
    //    p.PIN_27, p.PIN_17, p.PIN_13, p.PIN_14, p.PIN_26, p.PIN_15, p.PIN_16
    //).await;

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(p.USB, p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, p.PIN_8)))); // , p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, p.PIN_16
        },
    );
    
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(dac_manager))));//, daq))));

}


async fn make_usb(usb_pin:USB) ->
    (embassy_usb::UsbDevice<'static, Driver<'static, USB>>,
    embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out>,
    embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>,) {
    let driver = Driver::new(usb_pin, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB raw example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    
    
    // call static buffers from earlier
    let mut builder: Builder<'_, Driver<'_, USB>> = unsafe { Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        )
    };

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature,
    // which causes it to use the built-in WinUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xFF, 0, 0, None);
    let mut read_ep : embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out> = alt.endpoint_bulk_out(64);
    let mut write_ep: embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In> = alt.endpoint_bulk_in(64);
    drop(function);

    // Build the builder.
    let mut usb: embassy_usb::UsbDevice<'_, Driver<'_, USB>> = builder.build();
    (usb, read_ep, write_ep)
}


async fn daq_sender_from_tlu(
    //mut daq:&mut DAQFpga<'static, embassy_rp::peripherals::SPI0>
) {
    // loop {
    //     //let new_daq_sample = DAQSample {
    //     //    trigger_id: 1,
    //     //    trigger_clk: u64::MAX,
    //     //    trigger_data: 3,
    //     //    veto_in: true,
    //     //    internal_trigger: false,
    //     //};
    //     //DAQ_CHANNEL.send(new_daq_sample).await;

    DAQ_CHANNEL.send([126, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 125]).await;
    info!("sent daq sample");

    //info!("awaiting TLU");
    //daq.await_sample().await;
    //if let Ok(sample) = daq.read_sample() {
    //    DAQ_CHANNEL.send(sample).await;
    //    info!("received from TLU");
    //} else {info!("invalid DAQSample")}
}

async fn handle_inputs_from_core_1(mut dac_manager:&mut DacManager<'static>) {
    let input_data = INPUT_CHANNEL.receive().await;
    println!("input received: {:?}", input_data);

    match input_data[0] {
        0xFF => {
            handle_inputs::match_cli_values_to_functions(&input_data[1..64], &mut dac_manager).await;
        },
        _ => println!("invalid starting u8 of inputs is {}", input_data[0]),
    }
}


#[embassy_executor::task]
async fn core0_task(mut dac_manager:DacManager<'static>,
                    //mut daq:DAQFpga<'static, embassy_rp::peripherals::SPI0>
                ) {
    info!("Hello from core 0");

    loop {
        println!("n");
        let daqs_from_tlu = daq_sender_from_tlu().await;//&mut daq).await;
        println!("o");
        //let input_handler = handle_inputs_from_core_1(&mut dac_manager);
        //println!("p");

        //join(daqs_from_tlu, input_handler).await;
    }

}



async fn receive_user_inputs(mut read_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out>) {
    loop {
        println!("l");
        let mut data : [u8;64] = [0;64];
        let nbytes = read_ep.read(&mut data).await.expect("failed to read endpoint");
        println!("received external input");
        if nbytes < 64 { INPUT_CHANNEL.send(data).await } else {
            println!("error, too many bytes received {} > 64 bytes", nbytes);
        }
        println!("m");
    }
}

async fn sd_saving(
    sample:&[u8],
    mut sd_manager:&mut embedded_sdmmc::VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI1, spi::Blocking>, DummyCsPin, embedded_hal_bus::spi::NoDelay>, Output<'static>, embassy_time::Delay>, DummyTimesource>
) {

    // Try and access Volume 0 (i.e. the first partition).
    // The volume object holds information about the filesystem on that volume.
    let mut volume0 = &mut sd_manager.open_volume(embedded_sdmmc::VolumeIdx(0)).unwrap();
    info!("Volume 0: {:?}", defmt::Debug2Format(&volume0));

    // Open the root directory (mutably borrows from the volume).
    let mut root_dir = volume0.open_root_dir().unwrap();


    println!("k");
    loop {
        println!("hello sd");

        //let a: Result<filesystem::DirEntry, embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> = root_dir.find_directory_entry("SDTEST.TXT");
        //println!("{:?}", a.unwrap().name.base_name());

          // Open a file called "MY_FILE.TXT" in the root directory
          // This mutably borrows the directory.
          let mut my_file = root_dir
              .open_file_in_dir("SDTEST", embedded_sdmmc::Mode::ReadWriteCreateOrAppend)
              //.open_file_in_dir("SDTEST", embedded_sdmmc::Mode::ReadOnly)
              .expect("err");
////
              println!("ggvbhnj");


            //let my_file = sd_manager.open_file_in_dir(root_dir.to_raw_directory(), "SDTEST", embedded_sdmmc::Mode::ReadOnly).unwrap();

            //let my_file = my_file.to_file(sd_manager);

          //// Print the contents of the file
          if true {
              let mut buf = [0u8; 32];
              if let Ok(n) = my_file.read(&mut buf) {
                  info!("in file is : {:a}", buf[..n]);
              } else {}
          } else {}

          let a = my_file.to_raw_file();

          //my_file.

          //volume0.deref_mut();
          sd_manager.close_file(a);



        
        //let mut file = root_dir.open_file_in_dir("newfile", embedded_sdmmc::Mode::ReadWriteCreateOrAppend).expect("failed to create file on sd card");
        //info!("midpoint");
        //file.write(sample).expect("failed to write to file");



        warn!("done sd per turn!");
    }
}

async fn daq_sender_over_usb(
    mut write_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>,
    sample:&[u8],
) {
        println!("i");
    match write_ep.write(sample).await {
        Ok(_) => {println!("wrote DAQSample successfully to computer")},
        Err(err) => {println!("failed to send DAQSample due to {:?}", err)},
    }
        println!("j");
}


async fn handle_tlu_daqs(
    mut write_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>,
    mut sd_manager:embedded_sdmmc::VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI1, spi::Blocking>, DummyCsPin, embedded_hal_bus::spi::NoDelay>, Output<'static>, embassy_time::Delay>, DummyTimesource>
) {

        println!("4");
    loop {

            warn!("");
            println!("");
            println!("");
            println!("");
            println!("");


            let mut data: [u8; 64] = [0;64];
            let mut data_idx: usize = 0;
            for i in 0..4 {
                let daq_sample = DAQ_CHANNEL.receive().await;
                println!("d, {:?}", daq_sample);
                for j in 0..16 {
                    data[data_idx] = daq_sample[j];
                    data_idx += 1;
                }
            }

            println!("m, {:?}", data);

            match write_ep.write(&data).await {
                Ok(_) => {println!("wrote DAQSample successfully to computer")},
                Err(err) => {println!("failed to send DAQSample due to {:?}", err)},
            }

            warn!("yello!");

            println!("");
            println!("");
            println!("");
            println!("");
            println!("");


        println!("f");
        let daq_sender = daq_sender_over_usb(&mut write_ep, &data);
        println!("g");
        let sd_manager = sd_saving(&data, &mut sd_manager);
        println!("h");

        join(daq_sender, sd_manager).await;

    }

}

//async fn reader_writer(
//    mut read_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::Out>,
//    mut write_ep:&mut embassy_rp::usb::Endpoint<'static, USB, embassy_rp::usb::In>,
//) {
//    
//    info!("connected");
//        println!("3");
//    loop {
//        println!("c");
//        let input_receiver = receive_user_inputs(&mut read_ep);
//        println!("d");
//        let daq_handler = handle_tlu_daqs(&mut write_ep);
//        println!("e");
//        join(input_receiver, daq_handler).await;
//    }
//}





#[embassy_executor::task]
async fn core1_task(usb_pin:USB, pin_spi1:SPI1, pin_10:PIN_10, pin_11:PIN_11, pin_12:PIN_12, pin_8:PIN_8) {

    let mut sd_volume_manager: embedded_sdmmc::VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI1, spi::Blocking>, DummyCsPin, embedded_hal_bus::spi::NoDelay>, Output<'static>, embassy_time::Delay>, DummyTimesource> = get_sd_root_dir(pin_spi1, pin_10, pin_11, pin_12, pin_8).await;

    let (mut usb, mut read_ep, mut write_ep) = make_usb(usb_pin).await;

    info!("Hello from core 1");

    //read_ep.wait_enabled().await;

    println!("ab");
    let usb_runner = usb.run();
    println!("a");
    //let reader_writer = reader_writer(&mut read_ep, &mut write_ep);
    println!("b");

    //let input_receiver = receive_user_inputs(&mut read_ep);
    println!("d");
    let daq_handler = handle_tlu_daqs(&mut write_ep, sd_volume_manager);


    join(usb_runner, daq_handler).await;

    //join3(usb_runner, input_receiver, daq_handler).await;
    //join(usb_runner, reader_writer).await;

    //let usb_fut = usb.run();
    //let user = use_usb(&mut read_ep, &mut write_ep);//, sd_volume_manager);
    //join(usb_fut, user).await;
}