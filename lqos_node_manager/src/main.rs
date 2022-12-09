#[macro_use] extern crate rocket;
use rocket::fairing::AdHoc;
mod static_pages;
mod tracker;
mod shaped_devices;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::on_liftoff("Poll lqosd", |_| {
            Box::pin(async move {
                rocket::tokio::spawn(tracker::update_tracking());
            })
        }))
        .mount("/", routes![
            static_pages::index,
            static_pages::shaped_devices_csv_page,

            // Our JS library
            static_pages::lqos_js,

            // API calls
            tracker::current_throughput,
            tracker::throughput_ring,
            tracker::cpu_usage,
            tracker::ram_usage,
            tracker::top_10_downloaders,
            tracker::worst_10_rtt,
            tracker::rtt_histogram,
            tracker::host_counts,
            shaped_devices::all_shaped_devices,

            // Supporting files
            static_pages::bootsrap_css,
            static_pages::plotly_js,
            static_pages::jquery_js,
            static_pages::bootsrap_js,
            static_pages::tinylogo,
            static_pages::favicon,
        ])
}