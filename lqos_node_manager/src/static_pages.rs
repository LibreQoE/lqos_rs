use rocket::fs::NamedFile;

#[get("/")]
pub async fn index<'a>() -> Option<NamedFile> {
    NamedFile::open("static/main.html").await.ok()
}

#[get("/vendor/bootstrap.min.css")]
pub async fn bootsrap_css<'a>() -> Option<NamedFile> {
    NamedFile::open("static/vendor/bootstrap.min.css").await.ok()
}

#[get("/lqos.js")]
pub async fn lqos_js<'a>() -> Option<NamedFile> {
    NamedFile::open("static/lqos.js").await.ok()
}

#[get("/vendor/plotly-2.16.1.min.js")]
pub async fn plotly_js<'a>() -> Option<NamedFile> {
    NamedFile::open("static/vendor/plotly-2.16.1.min.js").await.ok()
}

#[get("/vendor/jquery.min.js")]
pub async fn jquery_js<'a>() -> Option<NamedFile> {
    NamedFile::open("static/vendor/jquery.min.js").await.ok()
}

#[get("/vendor/bootstrap.bundle.min.js")]
pub async fn bootsrap_js<'a>() -> Option<NamedFile> {
    NamedFile::open("static/vendor/bootstrap.bundle.min.js").await.ok()
}

#[get("/vendor/tinylogo.svg")]
pub async fn tinylogo<'a>() -> Option<NamedFile> {
    NamedFile::open("static/tinylogo.svg").await.ok()
}

#[get("/favicon.ico")]
pub async fn favicon<'a>() -> Option<NamedFile> {
    NamedFile::open("static/favicon.ico").await.ok()
}