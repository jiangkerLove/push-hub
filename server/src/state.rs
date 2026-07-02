use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;
use crate::push::hub_manager::PushHubManager;
use crate::push::online_hub::OnlinePushHub;
use crate::push::PushService;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub push_service: Arc<PushService>,
    pub hub_manager: Arc<PushHubManager>,
    pub config: Arc<Config>,
    pub online_hub: Arc<OnlinePushHub>,
}
