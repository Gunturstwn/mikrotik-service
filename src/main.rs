use mikrotik_service::AppState;
use mikrotik_service::config;
use mikrotik_service::routes;
use sea_orm_migration::MigratorTrait;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        mikrotik_service::handlers::health_handler::health_check,
        mikrotik_service::handlers::auth_handler::register,
        mikrotik_service::handlers::auth_handler::login,
        mikrotik_service::handlers::auth_handler::verify_token_handler,
        mikrotik_service::handlers::auth_handler::verify_email_handler,
        mikrotik_service::handlers::auth_handler::forgot_password,
        mikrotik_service::handlers::auth_handler::reset_password,
        mikrotik_service::handlers::user_handler::get_me,
        mikrotik_service::handlers::user_handler::update_me,
        mikrotik_service::handlers::user_handler::upload_photo,
        mikrotik_service::handlers::user_handler::get_users,
        mikrotik_service::handlers::user_handler::get_user,
        mikrotik_service::handlers::user_handler::delete_user,
        mikrotik_service::handlers::export_handler::export_users_csv,
        mikrotik_service::handlers::export_handler::export_users_xlsx,
    ),
    components(
        schemas(
            mikrotik_service::handlers::health_handler::HealthResponse,
            mikrotik_service::dto::auth::RegisterRequest,
            mikrotik_service::dto::auth::LoginRequest,
            mikrotik_service::dto::auth::AuthResponse,
            mikrotik_service::dto::auth::ForgotPasswordRequest,
            mikrotik_service::dto::auth::ResetPasswordRequest,
            mikrotik_service::dto::auth::VerifyTokenResponse,
            mikrotik_service::dto::user::UserProfileResponse,
            mikrotik_service::dto::user::UpdateUserRequest,
            mikrotik_service::dto::user::UserListResponse,
            mikrotik_service::dto::user::UploadPhotoRequest,
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "mikrotik_service=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing connection pools...");
    let db = config::database::connect().await;
    let redis_pool = config::redis::connect();
    let rabbit_conn = std::sync::Arc::new(config::rabbitmq::connect().await);

    let redis = mikrotik_service::cache::RedisClient::new(redis_pool);
    let rabbit = mikrotik_service::queue::RabbitMQClient::new(rabbit_conn.clone());

    // Initialize EmailWorker
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let smtp_port = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "1025".to_string())
        .parse()
        .unwrap_or(1025);
    let smtp_user = std::env::var("SMTP_USER").ok();
    let smtp_pass = std::env::var("SMTP_PASS").ok();

    let mut mailer_builder = if smtp_host == "localhost" || smtp_host == "127.0.0.1" {
        lettre::transport::smtp::SmtpTransport::builder_dangerous(smtp_host).port(smtp_port)
    } else if smtp_port == 465 {
        // Port 465 uses Implicit TLS (SSL)
        lettre::transport::smtp::SmtpTransport::relay(&smtp_host)
            .expect("Invalid SMTP host")
            .port(smtp_port)
            .tls(lettre::transport::smtp::client::Tls::Wrapper(
                lettre::transport::smtp::client::TlsParameters::new(smtp_host.clone()).unwrap()
            ))
    } else {
        // Port 587 uses STARTTLS
        lettre::transport::smtp::SmtpTransport::relay(&smtp_host)
            .expect("Invalid SMTP host")
            .port(smtp_port)
    };

    if let (Some(u), Some(p)) = (smtp_user.clone(), smtp_pass.clone()) {
        tracing::info!("SMTP: Loading credentials for user: {}", u);
        mailer_builder = mailer_builder
            .credentials(lettre::transport::smtp::authentication::Credentials::new(u, p))
            .authentication(vec![lettre::transport::smtp::authentication::Mechanism::Login]);
    } else {
        if smtp_user.is_none() {
            tracing::error!("SMTP ERROR: 'SMTP_USER' not found in .env!");
        }
        if smtp_pass.is_none() {
            tracing::error!("SMTP ERROR: 'SMTP_PASS' not found in .env!");
        }
        tracing::warn!("SMTP: Credentials incomplete, email delivery will likely fail.");
    }

    let mailer = mailer_builder.build();
    let email_worker = mikrotik_service::workers::EmailWorker::new(rabbit_conn.clone(), mailer);

    // Run EmailWorker in a background task
    tokio::spawn(async move {
        if let Err(e) = email_worker.run().await {
            tracing::error!("EmailWorker error: {}", e);
        }
    });

    let storage = config::storage::connect().await;
    let state = AppState::new(db.clone(), redis, rabbit, storage);

    // Run migrations automatically on startup (None type specified to fix inference)
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let app = routes::create_router(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let port = std::env::var("APP_PORT")
        .unwrap_or("3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server starting at http://{}", addr);
    info!("Swagger UI at http://{}/swagger-ui", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
