//! Embassy SPI
//!
//! Depending on your target and the board you are using you have to change the
//! pins.
//!
//! Connect MISO and MOSI pins to see the outgoing data is read as incoming
//! data.
//!
//! The following wiring is assumed:
//! SCLK => GPIO0
//! MISO => GPIO2
//! MOSI => GPIO4
//! CS   => GPIO5

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{timer::timg::TimerGroup, uart};
use heapless::{String, Vec};
use log::info;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    info!("Init");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    info!("embassy Init");

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let mut uart = uart::Uart::new(peripherals.UART0, uart::Config::default())
        .unwrap()
        .with_rx(peripherals.GPIO5)
        .with_tx(peripherals.GPIO4)
        .into_async();

    loop {
        let mut buffer = [0u8; 50];
        let len: usize = match uart.read_async(&mut buffer).await {
            Ok(len) => len,
            Err(error) => {
                log::error!("Error Reading UART: {error:?}");
                0
            }
        };
        info!("Raw Buffer: {buffer:?}");
        let mut vec = Vec::<u8, 50>::new();
        vec.extend_from_slice(&buffer).unwrap();
        info!("Transformed Buffer: {vec:?}");
        let string = String::<50>::from_utf8(vec).unwrap();
        info!("Len: {len:?}, Message: {string:?}");
    }
}
