#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_config::{esp_config_int, esp_config_str};
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::{systimer::SystemTimer, timg::TimerGroup};
use log::info;

extern crate alloc;

use sonic_bt;

const WIFI_CHANNEL: u8 = esp_config_int!(u8, "WIFI_CHANNEL");
const WIFI_SSID: &str = esp_config_str!("WIFI_SSID");
const WIFI_PASSWORD: &str = esp_config_str!("WIFI_PASSWORD");
const MAX_PAYLOAD_SIZE: u16 = esp_config_int!(u16, "MAX_PAYLOAD_SIZE");
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

    // TODO: Spawn some tasks
    let spawn = spawner;

    spawner.spawn(sonic_bt::led::run()).unwrap();

    sonic_bt::led::LED_STATUS.signal(sonic_bt::led::LEDStatus::Ok());

    Timer::after(Duration::from_millis(500)).await;

    sonic_bt::led::LED_STATUS.signal(sonic_bt::led::LEDStatus::Progressing());

    Timer::after(Duration::from_millis(500)).await;

    sonic_bt::led::LED_STATUS.signal(sonic_bt::led::LEDStatus::Panic());

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
