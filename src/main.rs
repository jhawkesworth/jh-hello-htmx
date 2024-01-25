#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use rocket_dyn_templates::Template;
use rocket_dyn_templates::context;

use rocket::response::Redirect;
use rocket::Request;
use rocket::fs::{FileServer, relative};
use rocket::serde::Serialize;
use rocket::State;
use rocket::form::Form;
use std::env;
use std::string::ToString;

use sqlx::{Executor, FromRow, PgPool};

use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;


use rustrict::CensorStr;

struct MyState {
    pool: PgPool,
}

#[derive(FromRow, Serialize, FromForm)]
struct Todo {
    pub id: i32,
    // #[field(default=false)]
    // pub done: bool,
    pub note: String,
}

#[derive(FromForm)]
struct TodoNew{
    pub note: String,
}

const DEFAULT_TODO: Todo = Todo {
    id: -1_i32,
    // created: DateTime::now_utc(),
    // done: false,
    note: String::new()
};

#[get("/todos")]
async fn get_todo_list(state: &State<MyState>) -> Template {
    // forget old things, but not the first 10 things.
    let mut transaction = state.pool.begin().await.unwrap();
    let _result = sqlx::query!("delete from TODOS where created < now() - interval '1.5' day and id not in (select id from todos order by id asc limit 10)")
        .execute(&mut *transaction)
        .await;
    transaction.commit().await.unwrap();

    let items = sqlx::query_as!(Todo, "SELECT id, note from todos order by id asc LIMIT 10")
        .fetch_all(&state.pool)
        .await.
        unwrap_or(vec![DEFAULT_TODO]);
    Template::render("todo", context! {
        title: "Todo List",
        items: items,
    })
}

#[post("/todos", data = "<data>")]
async fn add_todo(data: Form<TodoNew>, state: &State<MyState>) -> Template {
    let note = &data.note;

    let mut status = "Added..";

    if note.is_inappropriate() {
        status = "Use appropriate language and try again"
    } else if note.is_empty() {
        status = "Nothing to add"
    } else {
    let _todo_new = sqlx::query_as!(TodoNew,
        "INSERT INTO todos(note) VALUES ($1) RETURNING note",
        note.as_str())
        .fetch_one(&state.pool)
        .await.unwrap_or(TodoNew { note: "oh we failed".to_string() });
    }

    // it is possible above insert should be written using 'execute' instead of 'fetch_one'
    // TODO work out what htmx failure pattern should be.

    let items = sqlx::query_as!(Todo, "SELECT id, note from todos order by id asc LIMIT 10")
        .fetch_all(&state.pool)
        .await.
        unwrap_or(vec![DEFAULT_TODO]);

    Template::render("todo-edit-add", context! {
        status: status,
        title: "Todo List",
        items: items,
    })

}

#[get("/todos/<id>/edit")]
async fn edit_todo_form(id: i32, state: &State<MyState>) -> Template {
    let item = sqlx::query_as!(Todo, "SELECT id, note from todos where id= ($1)", id)
        .fetch_one(&state.pool)
        .await.
        unwrap_or(DEFAULT_TODO);
    Template::render("edit_todo", context! {
        title: "Edit todo",
        item: item
    })
}

#[put("/todos/<id>", data = "<data>")]
async fn update_todo(data: Form<Todo>, id: i32, state: &State<MyState>) -> Template {
    // TODO a boat load in tests regarding input.
    let _item = sqlx::query_as!(Todo, "update TODOS set note=($1) where id= ($2) returning id, note", data.note, id)
        .fetch_one(&state.pool)
        .await.
        unwrap_or(DEFAULT_TODO);

    // TODO check update was good
    let items = sqlx::query_as!(Todo, "SELECT id, note from todos order by id asc LIMIT 10")
        .fetch_all(&state.pool)
        .await.
        unwrap_or(vec![DEFAULT_TODO]);

    Template::render("todo-edit-add", context! {
        title: "Todo List",
        items: items,
    })
}

#[delete("/todos/<id>")]
async fn delete_todo(id: i32, state: &State<MyState>) -> Template {
    let _result = sqlx::query!("DELETE from todos where id= ($1)", id)
        .execute(&state.pool)
        .await;

    // TODO check the delete result and log success or failure?
    // either way want to refresh the list.
    // TODO work out what htmx failure pattern should be.
    let items = sqlx::query_as!(Todo, "SELECT id, note from todos order by id asc LIMIT 10")
        .fetch_all(&state.pool)
        .await.
        unwrap_or(vec![DEFAULT_TODO]);

    Template::render("todo-edit-add", context! {
        title: "Todo List",
        items: items,
    })

}
#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!("/todos"))
}

#[get("/favicon.ico")]
fn favicon() -> Redirect {
    Redirect::to(uri!("/static/static/favicon.ico"))
}


#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}

#[shuttle_runtime::main]
async fn rocket(#[shuttle_shared_db::Postgres] pool: PgPool,
                #[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_rocket::ShuttleRocket {

    let db_url = if let Some(db_url) = secret_store.get("DATABASE_URL") {
        db_url
    } else {
        "postgres://postgres:postgres@localhost:17209/postgres".to_string()
    };
    std::env::set_var("DATABASE_URL", db_url);
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };

    let template_dir = PathBuf::from("static");
    let figment = rocket::Config::figment().merge(("template_dir", template_dir));
    let rocket = rocket::custom(figment)
        .mount("/", routes![index,
            favicon,
            get_todo_list, add_todo, edit_todo_form, update_todo, delete_todo
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .register("/", catchers![not_found])
        .attach(Template::fairing())
        .manage(state)
        ;

    Ok(rocket.into())
}