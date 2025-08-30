#![no_std]
#![no_main]

use core::net::Ipv4Addr;
use core::str::FromStr;

use alloc::string::ToString;
use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_config::{esp_config_bool, esp_config_int, esp_config_str};
use esp_hal::clock::CpuClock;

use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::{AccessPointConfiguration, AuthMethod, WifiDevice};
use esp_wifi::EspWifiController;
use log::info;
use picoserve::AppWithStateBuilder;

extern crate alloc;

const GATEWAY_IP: &str = esp_config_str!("GATEWAY_IP");
const WIFI_CHANNEL: u8 = esp_config_int!(u8, "WIFI_CHANNEL");
const WIFI_CHANNEL_SECONDARY: u8 = esp_config_int!(u8, "WIFI_CHANNEL_SECONDARY");
const WIFI_SSID: &str = esp_config_str!("WIFI_SSID");
const WIFI_HIDDEN: bool = esp_config_bool!("WIFI_HIDDEN");
const WIFI_PASSWORD: &str = esp_config_str!("WIFI_PASSWORD");

#[allow(unused)]
const MAX_PAYLOAD_SIZE: u16 = esp_config_int!(u16, "MAX_PAYLOAD_SIZE");
#[allow(unused)]
const STACK_SIZE: u16 = esp_config_int!(u16, "STACK_SIZE");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // INFO: chip initialisation

    esp_println::logger::init_logger_from_env(); // logger

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max()); // chip config
    let peripherals = esp_hal::init(config); // chip initialisation

    esp_alloc::heap_allocator!(size: 128 * 1024); // initialise heap/aloc

    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let mut rng: Rng = Rng::new(peripherals.RNG);

    info!("Embassy initialized!");

    // esp_wifi stuff...
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let esp_wifi: &'static mut EspWifiController = sonic_bt::mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer1.timer0, rng).unwrap()
    );

    let auth_method: AuthMethod = match WIFI_PASSWORD.to_string().len() {
        0..4 => {
            log::info!("no auth");
            AuthMethod::None
        }
        _ => AuthMethod::WPA3Personal,
    };

    // embassy_net setup...
    let gateway_ip: Ipv4Addr =
        Ipv4Addr::from_str(GATEWAY_IP).expect("ERROR: Couldn't parse [`GATEWAY_IP`] to IP");

    let (controller, stack, runner): (
        esp_wifi::wifi::WifiController<'static>,
        embassy_net::Stack<'static>,
        embassy_net::Runner<'static, WifiDevice<'static>>,
    ) = sonic_bt::wifi::setup(
        peripherals.WIFI,
        esp_wifi,
        gateway_ip,
        (rng.random() as u64) << 32 | rng.random() as u64,
        AccessPointConfiguration {
            ssid: WIFI_SSID.to_string(),
            ssid_hidden: WIFI_HIDDEN,
            channel: WIFI_CHANNEL,
            secondary_channel: Some(WIFI_CHANNEL_SECONDARY),
            auth_method,
            password: WIFI_PASSWORD.to_string(),
            max_connections: 8,
            ..Default::default()
        },
    );

    spawner.must_spawn(sonic_bt::wifi::captive_dns(stack, gateway_ip));

    spawner.must_spawn(sonic_bt::wifi::dhcp_server(stack, gateway_ip));

    // start up wifi device
    spawner.must_spawn(sonic_bt::wifi::connection_up(controller));

    spawner.must_spawn(sonic_bt::wifi::run_net_task(runner));

    // INFO: Task Spawning
    // spawner.must_spawn(sonic_bt::status::run());

    embassy_time::Timer::after_secs(15).await;
    log::info!("starting webserver");

    sonic_bt::web_server_2::spawn_web_tasks(
        spawner,
        stack,
        sonic_bt::mk_static!(
            picoserve::AppRouter<sonic_bt::web_server_2::AppProps>,
            sonic_bt::web_server_2::AppProps.build_app()
        ),
        sonic_bt::mk_static!(
            picoserve::Config<embassy_time::Duration>,
            picoserve::Config::new(picoserve::Timeouts {
                start_read_request: Some(embassy_time::Duration::from_secs(5)),
                persistent_start_read_request: Some(embassy_time::Duration::from_secs(1)),
                read_request: Some(embassy_time::Duration::from_secs(1)),
                write: Some(embassy_time::Duration::from_secs(1))
            })
        ),
    );

    // cfg_if::cfg_if! {
    //     // status led controller
    //     if #[cfg(feature = "smart-led")] {
    //         use esp_hal::{rmt::{Rmt, ChannelCreator}, time::Rate};
    //
    //         let rmt_channel: ChannelCreator<esp_hal::Async, 0> = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("Failed to initialize RMT0").into_async().channel0;
    //         spawner.spawn(sonic_bt::status::led::led_run(rmt_channel, peripherals.GPIO8.into(), 0u8)).unwrap();
    //     }
    // }
    //
    // INFO: System Status Publisher
    // let publisher = sonic_bt::status::STATUS.publisher().unwrap();
    //
    // loop {
    //     if stack.is_link_up() {
    //         log::info!("link up");
    //     }
    //     if stack.is_config_up() {
    //         log::info!("Config up");
    //     }
    //     if stack.is_config_up() && stack.is_link_up() {
    //         break;
    //     }
    //     embassy_time::Timer::after_millis(500).await;
    // }
    //
    // spawner
    //     .spawn(sonic_bt::web_server::web_server(stack, gateway_ip))
    //     .ok();

    // // INFO: Demo loop
    // loop {
    //     publisher
    //         .publish(sonic_bt::status::Status::Ok(Some(
    //             String::try_from("abc").unwrap(),
    //         )))
    //         .await;
    //
    //     Timer::after(Duration::from_millis(500)).await;
    //
    //     publisher
    //         .publish(sonic_bt::status::Status::Warn(None))
    //         .await;
    //
    //     Timer::after(Duration::from_millis(500)).await;
    //
    //     publisher.publish(sonic_bt::status::Status::Err(None)).await;
    //
    //     Timer::after(Duration::from_millis(500)).await;
    //
    //     publisher
    //         .publish(sonic_bt::status::Status::Ok(Some(
    //             String::try_from("abc11").unwrap(),
    //         )))
    //         .await;
    //
    //     Timer::after(Duration::from_millis(500)).await;
    //
    //     publisher
    //         .publish(sonic_bt::status::Status::Ok(Some(
    //             String::try_from("abc22").unwrap(),
    //         )))
    //         .await;
    // }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
