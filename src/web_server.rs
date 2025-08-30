use core::fmt::Debug;
use edge_nal::TcpBind;
use embassy_executor::task;
use embassy_net::Stack;

#[task(pool_size = 4)]
pub async fn web_server(stack: Stack<'static>, server_ip: core::net::Ipv4Addr) {
    use core::net::{SocketAddr, SocketAddrV4};
    use edge_http::io::server::Server;
    use edge_nal_embassy::{Tcp, TcpBuffers};

    let tcp_buf: TcpBuffers<1usize, 1024usize, 1024usize> = TcpBuffers::new();

    let undound_socket = Tcp::new(stack, &tcp_buf);
    let socket = undound_socket
        .bind(SocketAddr::V4(SocketAddrV4::new(server_ip, 80)))
        .await
        .unwrap();

    let mut http_server: Server<1usize, 1024usize, 64usize> = Server::new();

    http_server
        .run(Some(500u32), socket, HandleBars)
        .await
        .unwrap();
}

struct HandleBars;

impl edge_http::io::server::Handler for HandleBars {
    type Error<E>
        = edge_http::io::Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        task_id: impl core::fmt::Display + Copy,
        connection: &mut edge_http::io::server::Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: embedded_io_async::Read + embedded_io_async::Write + edge_nal::TcpSplit,
    {
        use edge_http::{Method, RequestHeaders};

        let headers: &RequestHeaders<'_, N> = connection.headers()?;

        if headers.method != Method::Get {
            connection
                .initiate_response(405, Some("Method Not Allowed"), &[])
                .await?;
        } else if headers.path != "/" {
            connection
                .initiate_response(404, Some("Page Not Found"), &[])
                .await?;
        } else {
            connection
                .initiate_response(200, Some("Hello World"), &[("content-type", "plain/text")])
                .await?;
        }

        Ok(())
    }
}

