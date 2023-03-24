#[macro_use]
extern crate rocket;
use std::path::PathBuf;
use rocket_dyn_templates::Template;
use rocket_dyn_templates::context;
use rocket::response::Redirect;
use rocket::Request;
//use rocket::fs::{FileServer};

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!("/", hello(name = "Your Name")))
}

#[get("/favicon.ico")]
fn favicon() -> Redirect {
    Redirect::to(uri!("/static/static/favicon.ico"))
}


#[get("/hello/<name>")]
pub fn hello(name: &str) -> Template {
    Template::render("index", context! {
        title: "Hello",
        name: Some(name),
        items: vec!["One", "Two", "Three"],
    })
}
#[post("/clicked")]
pub fn button_clicked() -> Template {
    Template::render("no_button", context! {
        message: "You Replaced the Button"
    })
}

#[post("/button")]
pub fn a_clicked() -> Template {
    Template::render("button", context! {
        message: "You Got the button back"
    })
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}

#[shuttle_runtime::main]
async fn rocket(#[shuttle_static_folder::StaticFolder] _static_folder: PathBuf) -> shuttle_rocket::ShuttleRocket {
    //let static_path = static_folder.as_path();
    //std::thread::sleep(std::time::Duration::from_millis(3000));
    //    static_subdir.push("static");
    //let static = static_folder.clone();
    let rocket = rocket::build()
        .mount("/", routes![index, hello, button_clicked, a_clicked, favicon])
//        .mount("/static", FileServer::from(static.as_path()) )
        //.mount("/hello/static", FileServer::from(relative!("static")) )
        // .mount("/static", FileServer::from(relative!(&static_subdir)) )
        // .mount("/hello/static", FileServer::from(relative!(&static_subdir)) )
        .register("/", catchers![not_found])
        .attach(Template::fairing())
        ;

    Ok(rocket.into())
}