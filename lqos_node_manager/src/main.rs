use rocket::fairing::AdHoc;

#[macro_use] extern crate rocket;
mod static_pages;
mod tracker;

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

            // Our JS library
            static_pages::lqos_js,

            // API calls
            tracker::current_throughput,
            tracker::throughput_ring,
            tracker::cpu_usage,
            tracker::ram_usage,
            tracker::top_10_downloaders,

            // Supporting files
            static_pages::bootsrap_css,
            static_pages::plotly_js,
            static_pages::jquery_js,
            static_pages::bootsrap_js,
            static_pages::tinylogo,
            static_pages::favicon,
        ])
}