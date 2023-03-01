#[macro_use]
extern crate rocket;

mod mdns_record;
mod store;

use std::path::Path;

use rocket::response::Redirect;
use rocket::State;
use store::NameStore;

#[derive(Debug)]
enum Err {
    Mdns(mdns_record::Error),
}

impl From<mdns_record::Error> for Err {
    fn from(value: mdns_record::Error) -> Self {
        Err::Mdns(value)
    }
}

#[get("/?<token>")]
async fn index(paths: &State<NameStore>, token: Option<&str>) -> String {
    if let Some(token) = token {
        format!("{} => {}", token, paths.get(token).await.unwrap_or("<unbound>".into()))
    } else {
        format!("Hello world!")
    }
}

#[get("/<token>")]
async fn resolve_token(paths: &State<NameStore>, token: &str) -> Redirect {
    if let Some(path) = paths.get(token).await {
        Redirect::to(path)
    } else {
        Redirect::to(uri!(index(Some(token))))
    }
}

#[post("/<token>", data = "<url>")]
async fn assign_token(paths: &State<NameStore>, token: &str, url: &str) -> Redirect {
    paths.set(token, url).await.unwrap();
    Redirect::to(uri!(resolve_token(token)))
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(NameStore::new(
            Path::new("names.json"),
            Path::new("names_cache.json"),
        ).await.unwrap())
        .mount("/", routes![index, resolve_token, assign_token])
}
