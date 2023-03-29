use actix_web::{HttpResponse, web};
use chrono::Utc;
use tracing::{info, error};
use sqlx::{PgPool};

use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(data, pool),
    fields(
        email = %data.email,
        name = %data.name,
    ),
)]
pub async fn subscribe(
    data: web::Form<Subscription>,
    pool: web::Data<PgPool>
) -> HttpResponse {
    match insert_subscriber(&pool, &data).await {
        Ok(_) => {
            info!("New subscriber has been added.");
            HttpResponse::Ok().body(format!("Hello {}, your email {} was added to the newsletter", data.name, data.email))
        },
        Err(e) => {
            error!("Failed to execute query: {:?}.", e);
            HttpResponse::InternalServerError().body("There was an error while processing your request. Please try again later.")
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(data, pool),
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    data: &Subscription,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
