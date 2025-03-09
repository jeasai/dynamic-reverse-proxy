use lazy_static::lazy_static;
use std::{collections::HashMap, net::SocketAddr};

use tokio::sync::RwLock;

lazy_static! {
    static ref SNI_MAP: RwLock<HashMap<String, SocketAddr>> = RwLock::new(HashMap::new());
}

pub async fn add_sni_endpoint(sni: String, addr: SocketAddr) {
    let mut sni_map = SNI_MAP.write().await;
    sni_map.insert(sni, addr);
}

pub async fn remove_sni_endpoint(sni: &str) {
    let mut sni_map = SNI_MAP.write().await;
    sni_map.remove(sni);
}

pub async fn get_sni_endpoint(sni: &str) -> Option<SocketAddr> {
    let sni_map = SNI_MAP.read().await;
    sni_map.get(sni).copied()
}

pub async fn get_sni_endpoints() -> HashMap<String, SocketAddr> {
    let sni_map = SNI_MAP.read().await;
    sni_map.clone()
}
