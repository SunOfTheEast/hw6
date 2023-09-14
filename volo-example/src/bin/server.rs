#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use std::fs;
//use volo_example::FilterLayer;
use volo_example::{FilterLayer, S};
use log::{error, warn, info, debug, trace};
#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
    trace!("跟踪服务端");
    let addr: SocketAddr = "[::]:33333".parse().unwrap();
    let addr = volo::net::Address::from(addr);
    let db = S::new();
    volo_gen::volo::example::ItemServiceServer::new(db)
        .layer_front(FilterLayer)
        .run(addr)
        .await
        .unwrap();
}
