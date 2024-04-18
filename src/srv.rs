use std::net::{Ipv6Addr, SocketAddr};

use aok::Null;
use axum::extract::Request;
use hyper::body::Incoming;
use hyper_util::{
  rt::{TokioExecutor, TokioIo},
  server,
};
use tokio::net::TcpListener;

use crate::proxy::proxy;
// use tower::{Service, ServiceExt};

genv::s!(IPV6_PROXY_USER, IPV6_PROXY_PASSWD);

#[static_init::dynamic]
static AUTH: Box<[u8]> = [
  IPV6_PROXY_USER.as_bytes(),
  b":",
  IPV6_PROXY_PASSWD.as_bytes(),
]
.concat()
.into();

pub async fn srv(bind: SocketAddr, (ipv6, prefix_len): (Ipv6Addr, u8)) -> Null {
  let listener = TcpListener::bind(bind).await?;

  let ipv6: u128 = ipv6.into();

  loop {
    let (socket, _remote_addr) = listener.accept().await?;

    tokio::spawn(async move {
      let socket = TokioIo::new(socket);

      let hyper_service = hyper::service::service_fn(|req: Request<Incoming>| async move {
        proxy(ipv6, prefix_len, &AUTH, req).await
      });

      xerr::log!(
        server::conn::auto::Builder::new(TokioExecutor::new())
          .serve_connection_with_upgrades(socket, hyper_service)
          .await,
      )
    });
  }
}
