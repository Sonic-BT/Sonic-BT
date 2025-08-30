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
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{prelude::*, Drawable};
use epd_waveshare::{color::Color, prelude::*};
use esp_backtrace as _;
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers, gpio,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
    timer::timg::TimerGroup,
};
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

    let sclk = peripherals.GPIO18;
    let mosi = peripherals.GPIO23;
    let cs = gpio::Output::new(
        peripherals.GPIO5,
        gpio::Level::High,
        gpio::OutputConfig::default(),
    );
    let dc = gpio::Output::new(
        peripherals.GPIO22,
        gpio::Level::High,
        gpio::OutputConfig::default(),
    );
    let mut reset = gpio::Output::new(
        peripherals.GPIO21,
        gpio::Level::High,
        gpio::OutputConfig::default(),
    );
    let busy = gpio::Input::new(
        peripherals.GPIO4,
        gpio::InputConfig::default().with_pull(gpio::Pull::None),
    );
    let _power = gpio::Output::new(
        peripherals.GPIO10,
        gpio::Level::High,
        gpio::OutputConfig::default(),
    );

    let dma_channel = peripherals.DMA_CH0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus = Spi::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_dma(dma_channel)
    .with_buffers(dma_rx_buf, dma_tx_buf);

    info!("Spi Init 1/2");

    let mut spi =
        embedded_hal_bus::spi::ExclusiveDevice::new(spi_bus, cs, Delay).expect("spi error");

    info!("Spi Init 2/2");

    reset.set_low();
    Timer::after(Duration::from_millis(10)).await;
    reset.set_high();
    Timer::after(Duration::from_millis(200)).await;

    while busy.is_low() {
        Timer::after(Duration::from_millis(10)).await;
    }

    info!("Busy is High Proceding");

    info!(
        "Busy Pin State: {}",
        if busy.is_low() { "LOW" } else { "HIGH" }
    );

    let mut epd =
        epd_waveshare::epd2in13_v2::Epd2in13::new(&mut spi, busy, dc, reset, &mut Delay, None)
            .unwrap();

    info!("epd init 1/2");

    let mut display = epd_waveshare::epd2in13_v2::Display2in13::default();

    info!("epd init 2/2");

    display.set_rotation(epd_waveshare::graphics::DisplayRotation::Rotate0);
    epd.clear_frame(&mut spi, &mut Delay).unwrap();
    display.clear(Color::Black).unwrap();
    epd.update_and_display_frame(&mut spi, display.buffer(), &mut Delay)
        .unwrap();

    Timer::after(Duration::from_secs(5)).await;

    let style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(epd_waveshare::color::Color::Black)
        .build();

    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    embedded_graphics::text::Text::with_text_style(
        "Hello World",
        embedded_graphics::geometry::Point::new(5, 30),
        style,
        text_style,
    )
    .draw(&mut display)
    .unwrap();

    epd.update_and_display_frame(&mut spi, display.buffer(), &mut Delay)
        .unwrap();

    Timer::after(Duration::from_secs(5)).await;

    epd.sleep(&mut spi, &mut Delay).unwrap();

    log::info!("wow");
}
