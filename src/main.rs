mod structures;
mod error;
mod db;
mod endpoints;
mod hash;
mod layers;

use std::sync::Arc;
use axum::{middleware, Router};
use axum::routing::{get, post, put};
use mongodb::{Client};
use mongodb::options::{ClientOptions, Credential};
use dotenvy::dotenv;
use structures::IdGenerator;
use tokio::net::TcpListener;
use crate::endpoints::posts::{create_post, get_posts, rate_post};
use crate::endpoints::user::{login, register, update_token};
use crate::layers::auth::auth;

struct AppState {
    client: Client,
    id_gen: IdGenerator,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;
    let uri = std::env::var("MONGODB_URI")?;
    let mut client_options = ClientOptions::parse(uri).await?;
    let credentials = Credential::builder()
        .username(std::env::var("MONGO_USERNAME")?.to_string())
        .password(std::env::var("PASSWORD")?.to_string())
        .source("alexandria".to_string())
        .build();
    client_options.credential = Some(credentials);

    let client = Client::with_options(client_options)?;

    let state = Arc::new(AppState { client: client.clone(), id_gen: IdGenerator::new(client.database("alexandria")).await });

    let with = Router::new()
        .route("/create_post", post(create_post))
        .route("/get_posts", get(get_posts))
        .route("/rate_post", post(rate_post))
        .route("/update_token", put(update_token))
        .layer(middleware::from_fn(auth))
        .with_state(state.clone());

    let without = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state);

    let app = Router::new()
        .merge(with)
        .merge(without);

    let addr = TcpListener::bind(std::env::var("SERVER_URL")?.to_string()).await?;

    axum::serve(addr, app).await?;

    println!("Hello, world!");
    Ok(())
}