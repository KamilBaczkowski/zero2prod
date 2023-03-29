use std::{net::TcpListener};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{PgPool, PgConnection, Connection, Executor};
use uuid::Uuid;
use zero2prod::{startup::run, config::{get_config, DatabaseSettings}, telemetry::{get_subscriber, init_subscriber}};

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let response = client
    .get(format!("{}/health_check", address))
    .send()
    .await
    .expect("Failed to execute the request.");

    assert!(response.status().is_success());
    assert_ne!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let db_pool = app.db_pool;
    let address = app.address;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscription", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request.");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_missing_data() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name, and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscription", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute the request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 Bad Request when the payload was {}",
            error_message
        )
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create the DB.
    let mut connection = PgConnection::connect(
        config.db_conn_string().expose_secret()
    )
    .await
    .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create the database.");
    // Run DB migrations.
    let connection_pool = PgPool::connect(
        config.full_conn_string().expose_secret()
    )
    .await
    .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run database migrations.");

    connection_pool
}

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            "test".into(),
            "debug".into(),
            std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            "test".into(),
            "debug".into(),
            std::io::sink
        );
        init_subscriber(subscriber);
    }
});

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to the random port.");
    let port = listener.local_addr().unwrap().port();

    Lazy::force(&TRACING);

    let mut config = get_config().expect("Failed to read the config.");

    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.database).await;
    let server = run(listener, db_pool.clone()).expect("Failed to bind to the address.");
    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool,
    }
}
