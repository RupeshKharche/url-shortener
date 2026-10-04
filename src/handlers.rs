use crate::AppState;
use crate::errors::AppError;
use crate::models::{CreateUrlRequest, CreateUrlResponse, Url as UrlModel};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::Redirect;
use rand::Rng;
use sqlx::Error;
use url::Url;
use uuid::Uuid;

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn generate_code(state: &AppState) -> String {
    let app_config = &state.config;

    let mut rng = rand::rng();
    let mut code = String::with_capacity(app_config.code_length);

    while code.len() < app_config.code_length {
        let mut byte = [0u8; 1];
        rng.fill_bytes(&mut byte);

        let value = byte[0];

        // 1 byte = 256 values, but we are using base 62,
        // so we can use max of floor(256/62) * 62 = 248,
        // to prevent mapping the remaining 8 values to 0..7 again (adds bias to those chars from our constant)
        if value >= 248 {
            continue;
        }

        code.push(BASE62[(value % 62) as usize] as char);
    }

    code
}

pub async fn shorten_url(
    State(state): State<AppState>,
    Json(payload): Json<CreateUrlRequest>,
) -> Result<Json<CreateUrlResponse>, AppError> {
    let app_config = &state.config;

    // Parse the incoming url into a Url instance
    let parsed_url =
        Url::parse(&payload.url).map_err(|_| AppError::BadRequest("Invalid URL".to_string()))?;

    // Only allow http and https urls
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(AppError::BadRequest(
            "Only HTTP and HTTPS urls are allowed".to_string(),
        ));
    };

    let original_url = parsed_url.to_string();

    let (id, code) = loop {
        // Generate id and code for the url
        let id = Uuid::new_v4().to_string();
        let code = generate_code(&state);

        // Insert into the db, keep on looping until the generated code is unique
        let result = sqlx::query(
            r#"
                insert into urls (id, code, original_url)
                values (?, ?, ?)
                "#,
        )
        .bind(&id)
        .bind(&code)
        .bind(&original_url)
        .execute(&state.db)
        .await;

        match result {
            Ok(_) => break (id, code),

            Err(Error::Database(err)) => {
                if err.is_unique_violation() {
                    continue;
                }
            }

            Err(error) => return Err(AppError::Database(error)),
        }
    };

    let base_url = &app_config.base_url;
    let short_url = format!("{base_url}/{code}");

    println!("Created {id} -> {original_url}");

    Ok(Json(CreateUrlResponse { short_url }))
}

pub async fn redirect_to_url(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    // Fetch the original url against the given code
    let url = sqlx::query_as::<_, UrlModel>(
        r#"
            select
                id,
                original_url,
                code,
                created_at
            from
                urls
            where code = ?
            "#,
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await?;

    let url = url.ok_or(AppError::NotFound)?;

    Ok(Redirect::temporary(&url.original_url))
}

pub async fn health(State(state): State<AppState>) -> &'static str {
    match sqlx::query("select 1").execute(&state.db).await {
        Ok(_) => "OK",
        Err(_) => "Database Unavailable",
    }
}
