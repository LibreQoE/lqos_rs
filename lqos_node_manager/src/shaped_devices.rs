use lqos_config::ShapedDevice;
use rocket::serde::json::Json;
use crate::cache_control::NoCache;
use crate::tracker::SHAPED_DEVICES;

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