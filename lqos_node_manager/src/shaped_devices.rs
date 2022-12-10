use lqos_config::ShapedDevice;
use rocket::serde::json::Json;
use crate::cache_control::NoCache;
use crate::tracker::SHAPED_DEVICES;

#[get("/api/all_shaped_devices")]
pub fn all_shaped_devices() -> NoCache<Json<Vec<ShapedDevice>>> {
    NoCache::new(Json(SHAPED_DEVICES.read().devices.clone()))
}