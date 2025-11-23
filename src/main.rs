use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use std::env;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;

mod user;
mod product;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Database URL must be set!!");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database!");

    println!("Connected to database successfully.");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .configure(user::init)
                    .configure(product::init)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
