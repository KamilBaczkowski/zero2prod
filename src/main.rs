use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::{PgPool};
use zero2prod::{startup::run, config::get_config, telemetry::{get_subscriber, init_subscriber}};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get the config from a file.
    println!("kocham amelke");
    let config = get_config().expect("Failed to read the configuration.");

    let subscriber = get_subscriber(
        "zero2prod".into(),
        config.log_level.as_str().into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    // Set up the DB connection
    let db_pool = PgPool::connect_lazy(
        config.database.full_conn_string().expose_secret()
    )
    .expect("Failed to connect to Postgress.");

    // Get address information to attach to.
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address).expect("Failed to bind to the address.");

    // Run the application.
    run(listener, db_pool)?.await
}
