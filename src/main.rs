#![feature(async_closure)]
mod proxy;

use std::{env, process::exit};

use cidr::Ipv6Cidr;
use getopts::Options;
mod srv;

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  print!("{}", opts.usage(&brief));
}

fn main() {
  loginit::init();
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optopt("b", "bind", "http proxy bind address", "BIND");
  opts.optopt(
    "i",
    "ipv6-subnet",
    "IPv6 Subnet: 2001:19f0:6001:48e4::/64",
    "IPv6_SUBNET",
  );
  opts.optflag("h", "help", "print this help menu");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => {
      panic!("{}", f.to_string())
    }
  };
  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  }

  let bind_addr = matches.opt_str("b").unwrap_or("0.0.0.0:51080".to_string());
  let ipve_subnet = matches
    .opt_str("i")
    .unwrap_or("2001:19f0:6001:48e4::/64".to_string());

  tracing::info!("Proxy on {} with IPv6 subnet {}", bind_addr, ipve_subnet);
  run(bind_addr, ipve_subnet)
}

#[tokio::main]
async fn run(bind_addr: String, ipv6_subnet: String) {
  let ipv6 = match ipv6_subnet.parse::<Ipv6Cidr>() {
    Ok(cidr) => {
      let a = cidr.first_address();
      let b = cidr.network_length();
      (a, b)
    }
    Err(_) => {
      tracing::error!("invalid ipv6 subnet");
      exit(1);
    }
  };

  let bind_addr = match bind_addr.parse() {
    Ok(b) => b,
    Err(e) => {
      tracing::error!("bind address not valid: {}", e);
      return;
    }
  };

  xerr::log!(srv::srv(bind_addr, ipv6).await);
}
