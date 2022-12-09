use lqos_config::ShapedDevice;
use rocket::serde::json::Json;
use crate::tracker::SHAPED_DEVICES;

#[get("/api/all_shaped_devices")]
pub fn all_shaped_devices() -> Json<Vec<ShapedDevice>> {
    Json(SHAPED_DEVICES.read().devices.clone())
}