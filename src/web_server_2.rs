use picoserve::{routing::get, AppBuilder, AppRouter, Router};

pub struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter<()>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        Router::new().route("/", get(|| async move { "hello_world" }))
    }
}

const WEB_TASK_POOL_SIZE: usize = 1;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<embassy_time::Duration>,
) {
    let port: u16 = 8080;
    let mut tcp_rx_buffer = [0; 512];
    let mut tcp_tx_buffer = [0; 512];
    let mut http_buffer = [0; 1024];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await;
}

pub fn spawn_web_tasks(
    spawner: embassy_executor::Spawner,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<embassy_time::Duration>,
) {
    //for i in 0..WEB_TASK_POOL_SIZE {
    spawner.must_spawn(web_task(1, stack, app, config));
    //}
}
