use lqos_config::ShapedDevice;
use rocket::serde::json::Json;
use crate::cache_control::NoCache;
use crate::tracker::SHAPED_DEVICES;
use lazy_static::*;
use parking_lot::RwLock;

lazy_static! {
    static ref RELOAD_REQUIRED : RwLock<bool> = RwLock::new(false);
}

#[get("/api/all_shaped_devices")]
pub fn all_shaped_devices() -> NoCache<Json<Vec<ShapedDevice>>> {
    NoCache::new(Json(SHAPED_DEVICES.read().devices.clone()))
}

#[get("/api/shaped_devices_count")]
pub fn shaped_devices_count() -> NoCache<Json<usize>> {
    NoCache::new(Json(SHAPED_DEVICES.read().devices.len()))
}

#[get("/api/shaped_devices_range/<start>/<end>")]
pub fn shaped_devices_range(start: usize, end: usize) -> NoCache<Json<Vec<ShapedDevice>>> {
    let reader = SHAPED_DEVICES.read();
    let result: Vec<ShapedDevice> = reader.devices.iter().skip(start).take(end).cloned().collect();
    NoCache::new(Json(result))
}

#[get("/api/shaped_devices_search/<term>")]
pub fn shaped_devices_search(term: String) -> NoCache<Json<Vec<ShapedDevice>>> {
    let term = term.trim().to_lowercase();
    let reader = SHAPED_DEVICES.read();
    let result: Vec<ShapedDevice> = reader
        .devices
        .iter()
        .filter(|s| 
            s.circuit_name.trim().to_lowercase().contains(&term) ||
            s.device_name.trim().to_lowercase().contains(&term)
        )
        .cloned()
        .collect();
    NoCache::new(Json(result))
}

#[get("/api/reload_required")]
pub fn reload_required() -> NoCache<Json<bool>> {
    NoCache::new(Json(*RELOAD_REQUIRED.read()))
}

#[get("/api/reload_libreqos")]
pub fn reload_libreqos() -> Json<String> {
    let result = lqos_config::load_libreqos().unwrap();
    println!("Reload result: {}", result);
    *RELOAD_REQUIRED.write() = false;
    Json(result)
}