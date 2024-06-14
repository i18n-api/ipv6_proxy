use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use http_body_util::Full;
use hyper::{
  body::{Body, Bytes},
  header,
  upgrade::Upgraded,
  Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use rand::Rng;
use tokio::net::TcpSocket;

async fn tunnel(
  ipv6: u128,
  prefix_len: u8,
  upgraded: &mut Upgraded,
  addr_str: String,
) -> std::io::Result<()> {
  if let Ok(addrs) = addr_str.to_socket_addrs() {
    for addr in addrs {
      let socket = TcpSocket::new_v6()?;
      let bind_addr = rand_ipv6_socket_addr(ipv6, prefix_len);
      if xerr::is_ok!(socket.bind(bind_addr)) {
        tracing::info!("{addr_str} → {bind_addr}");
        if let Ok(mut server) = xerr::ok!(socket.connect(addr).await) {
          let mut upgraded = TokioIo::new(upgraded);
          tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
          return Ok(());
        }
      }
    }
  } else {
    tracing::error!("ipv6 tunnel error: {addr_str}");
  }

  Ok(())
}

fn rand_ipv6_socket_addr(ipv6: u128, prefix_len: u8) -> SocketAddr {
  let mut rng = rand::thread_rng();
  SocketAddr::new(rand_ipv6(ipv6, prefix_len), rng.gen_range(1025..u16::MAX))
}

fn rand_ipv6(mut ipv6: u128, prefix_len: u8) -> IpAddr {
  // 避免第一个 IP 被绑定了端口
  let rand: u128 = rand::thread_rng().gen_range(2..u128::MAX);
  let net_part = (ipv6 >> (128 - prefix_len)) << (128 - prefix_len);
  let host_part = (rand << prefix_len) >> prefix_len;
  ipv6 = net_part | host_part;
  IpAddr::V6(ipv6.into())
}

async fn process_connect(ipv6: u128, prefix_len: u8, req: Request<impl Body>) {
  let remote_addr = req.uri();
  if let Some(remote_addr) = remote_addr.authority().map(|auth| auth.to_string()) {
    let upgrade = hyper::upgrade::on(req);
    tokio::task::spawn(async move {
      if let Ok(mut upgraded) = xerr::ok!(upgrade.await) {
        xerr::log!(tunnel(ipv6, prefix_len, &mut upgraded, remote_addr).await);
      }
    });
  }
}
pub async fn proxy(
  ipv6: u128,
  prefix_len: u8,
  user_passwd: &[u8],
  mut req: Request<impl Body>,
) -> aok::Result<Response<Full<Bytes>>> {
  if let Some(auth) = req.headers_mut().remove(header::PROXY_AUTHORIZATION) {
    let auth = auth.to_str()?;
    if let Some(p) = auth.find(' ') {
      if let Ok(auth) = xerr::ok!(B64.decode(&auth[p + 1..])) {
        if user_passwd == &auth[..] {
          let method = req.method();
          if method != Method::CONNECT {
            return resp(StatusCode::METHOD_NOT_ALLOWED);
          } else {
            process_connect(ipv6, prefix_len, req).await;
            return resp(StatusCode::OK);
          }
        }
      }
    }
  }

  resp(StatusCode::UNAUTHORIZED)

  // } else {
  // self.process_request(req).await
  // } {
  //   Ok(resp) => Ok(resp),
  //   Err(e) => Err(e),
  // }
}
pub fn resp(status_code: StatusCode) -> aok::Result<Response<Full<Bytes>>> {
  Ok(
    Response::builder()
      .status(status_code)
      .body(Full::new(Bytes::new()))?,
  )
}

// genv::s!(IPV6_PROXY_USER, IPV6_PROXY_PASSWD);
//
// pub async fn start_proxy(
//   listen_addr: SocketAddr,
//   (ipv6, prefix_len): (Ipv6Addr, u8),
// ) -> Result<(), Box<dyn std::error::Error>> {
//   let make_service = make_service_fn(move |_: &AddrStream| async move {
//     Ok::<_, hyper::Error>(service_fn(move |req| {
//       let auth: Box<[u8]> = format!("{}:{}", &*IPV6_PROXY_USER, &*IPV6_PROXY_PASSWD)
//         .as_bytes()
//         .into();
//       Proxy {
//         ipv6: ipv6.into(),
//         prefix_len,
//         auth,
//       }
//       .proxy(req)
//     }))
//   });
//
//   Server::bind(&listen_addr)
//     .http1_preserve_header_case(true)
//     .http1_title_case_headers(true)
//     .serve(make_service)
//     .await
//     .map_err(|err| err.into())
// }

// #[derive(Clone)]
// pub(crate) struct Proxy {
//   pub ipv6: u128,
//   pub prefix_len: u8,
//   pub auth: Box<[u8]>,
// }

// impl Proxy {

// async fn process_request(self, mut req: Request<impl Body>) -> Result<Response<impl Body>, hyper::Error> {
//   let bind_addr = rand_ipv6(self.ipv6, self.prefix_len);
//   let mut http = HttpConnector::new();
//   http.set_local_address(Some(bind_addr));
//   let uri = req.uri().to_string();
//
//   dbg!(&req);
//   if uri.starts_with('/') {
//     let host = req
//       .headers()
//       .get(hyper::header::HOST)
//       .and_then(|value| value.to_str().ok())
//       .unwrap_or_default();
//
//     let uri = Uri::builder()
//       .scheme("http")
//       .authority(host)
//       .path_and_query(uri)
//       .build()
//       .unwrap();
//
//     *req.uri_mut() = uri;
//   }
//
//   tracing::info!("{} {bind_addr}", req.uri());
//   let client = Client::builder()
//     .http1_title_case_headers(true)
//     .http1_preserve_header_case(true)
//     .build(http);
//   let res = client.request(req).await?;
//   Ok(res)
// }
// }
