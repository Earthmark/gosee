#[macro_use]
extern crate rocket;

mod client;
mod mdns_record;
mod store;

use client::StaticClientFiles;
use rocket::{
    http::Status,
    response::{Redirect, Responder},
    Request, State,
};
use std::path::Path;
use store::NameStore;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("mDNS: {0}")]
    Mdns(#[from] mdns_record::Error),
    #[error("Store: {0}")]
    Store(#[from] store::Error),
    #[error("Rocket: {0}")]
    Rocket(#[from] rocket::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'o> {
        error!("Internal error: {}", self);
        Err(Status::InternalServerError)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[get("/?<token>")]
fn index(paths: &State<NameStore>, token: Option<&str>) -> Result<String> {
    let res = if let Some(token) = token {
        format!(
            "{} => {}",
            token,
            paths.get(token)?.unwrap_or("<unbound>".into())
        )
    } else {
        format!("Hello world!")
    };
    Ok(res)
}

#[get("/<token>")]
fn resolve_token(paths: &State<NameStore>, token: &str) -> Result<Redirect> {
    let res = if let Some(path) = paths.get(token)? {
        Redirect::to(path)
    } else {
        Redirect::to(uri!(index(Some(token))))
    };
    Ok(res)
}

#[post("/<token>", data = "<url>")]
fn assign_token(paths: &State<NameStore>, token: &str, url: &str) -> Result<Redirect> {
    paths.set(token, url)?;
    Ok(Redirect::to(uri!(resolve_token(token))))
}

#[rocket::main]
async fn main() -> Result<()> {
    let _rocket = rocket::build()
        .manage(NameStore::new(Path::new("store"))?)
        .mount("/", routes![index, resolve_token, assign_token])
        .mount("/", StaticClientFiles::new())
        .launch()
        .await?;

    Ok(())
}
