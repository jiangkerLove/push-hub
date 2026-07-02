use std::sync::Arc;

use push_hub_server::config::Config;
use push_hub_server::db::{seed, Database};
use push_hub_server::push::{OnlinePushHub, PushHubManager, PushService};
use push_hub_server::routes;
use push_hub_server::state::AppState;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "push_hub_server=info,tower_http=info".into()),
        )
        .init();

    let config = Arc::new(Config::from_env()?);
    let db = Arc::new(Database::connect(&config.database_url).await?);
    seed::seed(&db, &config).await?;

    let online_hub = Arc::new(OnlinePushHub::new());
    let hub_manager = Arc::new(PushHubManager::new(db.clone(), online_hub.clone()));
    let push_service = Arc::new(PushService::new(config.clone()));

    push_hub_server::push::fallback_worker::spawn(db.clone(), hub_manager.clone());

    let state = AppState {
        db: db.clone(),
        push_service,
        hub_manager,
        config: config.clone(),
        online_hub,
    };

    let app = routes::create_router(state);

    let addr = config.listen_addr();
    info!("push-hub server listening on http://{addr}");
    let admin_count = db.admin_users().count().await?;
    if admin_count == 0 {
        info!("no admin user yet; open admin console to create the first account");
    } else {
        info!("admin users ready ({admin_count})");
    }
    info!(
        "database={:?}; vendor credentials are configured per app in admin console",
        db.kind()
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
