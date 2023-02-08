#[macro_use]
extern crate rocket;

use std::path::PathBuf;
use rocket::{Build, Rocket};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[shuttle_service::main]
//async fn rocket(#[shuttle_static_folder::StaticFolder(folder = "templates")] _static_folder: PathBuf) -> shuttle_service::ShuttleRocket {
async fn rocket(#[shuttle_static_folder::StaticFolder(folder = "templates")] _static_folder: PathBuf) -> Result<Rocket<Build>, shuttle_service::Error> {
    let rocket = rocket::build().mount("/hello", routes![index]);

    Ok(rocket)
}