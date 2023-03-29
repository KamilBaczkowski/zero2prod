use std::net::TcpListener;

use actix_web::{dev::Server, HttpServer, App, web};
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Make the connection object Clone'able.
    let db_pool = web::Data::new(db_pool);
    // Construct the HTTP Server.
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}