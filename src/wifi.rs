use core::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use core::str::FromStr;

//use crate::status::Status;
use edge_nal::UdpBind;
use embassy_executor::task;
use embassy_net::{Ipv4Cidr, StackResources};
use embassy_time::Timer;
use esp_hal::peripherals::WIFI;
use esp_wifi::wifi;
use esp_wifi::wifi::{WifiController, WifiDevice};
use esp_wifi::EspWifiController;
use heapless::{String, Vec};
use static_cell::StaticCell;

pub static ESP_WIFI: StaticCell<EspWifiController<'static>> = StaticCell::new();

/// Setup the network stack
///
/// giving all resourses/controllers to further the network stack.
pub fn setup(
    wifi_peripheral: WIFI<'static>,
    wifi_controller: &'static EspWifiController,
    gateway_ip: Ipv4Addr,
    seed: u64,
    access_point_config: esp_wifi::wifi::AccessPointConfiguration,
) -> (
    esp_wifi::wifi::WifiController<'static>,
    embassy_net::Stack<'static>,
    embassy_net::Runner<'static, WifiDevice<'static>>,
) {
    let (mut controller, interfaces) = wifi::new(wifi_controller, wifi_peripheral).unwrap();

    controller
        .set_configuration(&esp_wifi::wifi::Configuration::AccessPoint(
            access_point_config,
        ))
        .unwrap();

    let (stack, runner): (
        embassy_net::Stack<'static>,
        embassy_net::Runner<'static, WifiDevice<'static>>,
    ) = embassy_net::new(
        interfaces.ap,
        embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(gateway_ip, 24u8),
            gateway: Some(gateway_ip),
            dns_servers: Default::default(),
        }),
        crate::mk_static!(StackResources<4>, StackResources::<4>::new()),
        seed,
    );

    (controller, stack, runner)
}

#[task]
pub async fn connection_up(mut wifi_conftroler: WifiController<'static>) {
    // let publisher: embassy_sync::pubsub::Publisher<
    //     'static,
    //     embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    //     Status,
    //     2,
    //     2,
    //     4,
    // > = crate::status::STATUS.publisher().unwrap();

    // publisher
    //     .publish(Status::Ok(Some(
    //         String::from_str("Starting Up Wifi AccessPoint").unwrap(),
    //     )))
    //     .await;

    log::info!("Starting Up Wifi AccessPoint");

    loop {
        if !wifi_conftroler
            .is_started()
            .expect("Failed To Check wifi_conftroler status")
        {
            log::info!("starting wifi controller");
            _ = wifi_conftroler
                .start_async()
                .await
                .inspect_err(|e| log::error!("Error Starting wifi {e:?}"));
        }
        Timer::after_millis(500).await;
    }
}

#[task]
pub async fn captive_dns(stack: embassy_net::Stack<'static>, redirect_ip: Ipv4Addr) {
    let udp_buffers: edge_nal_embassy::UdpBuffers<1, 512, 512, 1> =
        edge_nal_embassy::UdpBuffers::new();
    let udp = edge_nal_embassy::Udp::new(stack, &udp_buffers);

    let mut tx_buf: &mut [u8] = &mut [0; 512];
    let mut rx_buf: &mut [u8] = &mut [0; 512];

    log::info!("captive Checkpoint");

    edge_captive::io::run(
        &udp,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 53),
        &mut tx_buf,
        &mut rx_buf,
        redirect_ip,
        core::time::Duration::from_secs(60),
    )
    .await
    .unwrap();
}

#[task]
pub async fn dhcp_server(stack: embassy_net::Stack<'static>, gateway_ip: Ipv4Addr) {
    let mut buf = [0u8; 1024];
    let mut gateway_buf = [Ipv4Addr::UNSPECIFIED];
    let buffers: edge_nal_embassy::UdpBuffers<3, 1024, 1024, 10> =
        edge_nal_embassy::UdpBuffers::new();
    let unbound_socket = edge_nal_embassy::Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            edge_dhcp::io::DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();
    let mut server: edge_dhcp::server::Server<_, 64> =
        edge_dhcp::server::Server::new_with_et(gateway_ip.clone());

    let dns = &[gateway_ip];

    let mut server_options =
        edge_dhcp::server::ServerOptions::new(gateway_ip.clone(), Some(&mut gateway_buf));
    server_options.dns = dns;

    log::info!("dhcp Checkpoint");

    loop {
        _ = edge_dhcp::io::server::run(&mut server, &server_options, &mut bound_socket, &mut buf)
            .await
            .inspect_err(|e| log::error!("DHCP Server Error: {e:?}"));
        embassy_time::Timer::after_millis(500).await;
    }
}

#[task]
pub async fn run_net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}
