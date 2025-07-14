#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_config::{esp_config_int, esp_config_str};
use esp_hal::clock::CpuClock;

use esp_hal::timer::{systimer::SystemTimer, timg::TimerGroup};
use heapless::String;

use log::info;

extern crate alloc;

use sonic_bt;

#[allow(unused)]
const WIFI_CHANNEL: u8 = esp_config_int!(u8, "WIFI_CHANNEL");
#[allow(unused)]
const WIFI_SSID: &str = esp_config_str!("WIFI_SSID");
#[allow(unused)]
const WIFI_PASSWORD: &str = esp_config_str!("WIFI_PASSWORD");
#[allow(unused)]
const MAX_PAYLOAD_SIZE: u16 = esp_config_int!(u16, "MAX_PAYLOAD_SIZE");
#[allow(unused)]
const STACK_SIZE: u16 = esp_config_int!(u16, "STACK_SIZE");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    spawner.spawn(sonic_bt::status::run()).unwrap();

    cfg_if::cfg_if! {
        if #[cfg(feature = "smart-led")] {
            use esp_hal::{rmt::{Rmt, ChannelCreator}, time::Rate};

            let rmt_channel: ChannelCreator<esp_hal::Async, 0> = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("Failed to initialize RMT0").into_async().channel0;
            spawner.spawn(sonic_bt::status::led::led_run(rmt_channel, peripherals.GPIO8.into(), 128u8)).unwrap();
        }
    }

    let publisher = sonic_bt::status::STATUS.publisher().unwrap();

    loop {
        publisher
            .publish(sonic_bt::status::Status::Ok(Some(
                String::try_from("abc").unwrap(),
            )))
            .await;

        Timer::after(Duration::from_millis(500)).await;

        publisher
            .publish(sonic_bt::status::Status::Warn(None))
            .await;

        Timer::after(Duration::from_millis(500)).await;

        publisher.publish(sonic_bt::status::Status::Err(None)).await;

        Timer::after(Duration::from_millis(500)).await;

        publisher
            .publish(sonic_bt::status::Status::Ok(Some(
                String::try_from("abc11").unwrap(),
            )))
            .await;

        Timer::after(Duration::from_millis(500)).await;

        publisher
            .publish(sonic_bt::status::Status::Ok(Some(
                String::try_from("abc22").unwrap(),
            )))
            .await;
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
