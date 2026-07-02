use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::db::Database;
use crate::models::PushApp;
use crate::push::online_hub::OnlinePushHub;
use crate::push::{HonorPushProvider, HuaweiPushProvider, MeizuPushProvider, OnlinePushProvider, OppoPushProvider, PushHub, PushProvider, VivoPushProvider, XiaomiPushProvider};
use crate::AppResult;

pub struct PushHubManager {
    online_hub: Arc<OnlinePushHub>,
    db: Arc<Database>,
    cache: RwLock<HashMap<String, Arc<PushHub>>>,
}

impl PushHubManager {
    pub fn new(db: Arc<Database>, online_hub: Arc<OnlinePushHub>) -> Self {
        Self {
            online_hub,
            db,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn invalidate(&self, app_id: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(app_id);
        }
    }

    pub fn hub_for_app(&self, app: &PushApp) -> AppResult<Arc<PushHub>> {
        if let Ok(cache) = self.cache.read() {
            if let Some(hub) = cache.get(&app.id) {
                return Ok(hub.clone());
            }
        }

        let hub = Arc::new(build_hub(app, self.db.clone(), self.online_hub.clone())?);
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(app.id.clone(), hub.clone());
        }
        Ok(hub)
    }
}

fn build_hub(
    app: &PushApp,
    db: Arc<Database>,
    online_hub: Arc<OnlinePushHub>,
) -> AppResult<PushHub> {
    let mut providers: Vec<Arc<dyn PushProvider>> = Vec::new();

    if let Some(secret) = app
        .xiaomi_app_secret
        .as_ref()
        .filter(|v| !v.trim().is_empty())
    {
        providers.push(Arc::new(XiaomiPushProvider::new(
            secret.clone(),
            app.package_name.clone(),
        )));
    }

    if let (Some(app_id), Some(secret)) = (
        app.huawei_app_id.as_ref().filter(|v| !v.trim().is_empty()),
        app.huawei_app_secret.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        let oauth_client_id = app
            .huawei_oauth_client_id
            .clone()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| app_id.clone());
        providers.push(Arc::new(HuaweiPushProvider::new(
            app_id.clone(),
            oauth_client_id,
            secret.clone(),
            app.package_name.clone(),
        )));
    }

    if let (Some(app_key), Some(master_secret)) = (
        app.oppo_app_key.as_ref().filter(|v| !v.trim().is_empty()),
        app.oppo_master_secret.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        providers.push(Arc::new(OppoPushProvider::new(
            app_key.clone(),
            master_secret.clone(),
        )));
    }

    if let (Some(app_id), Some(app_key), Some(app_secret)) = (
        app.vivo_app_id.as_ref().filter(|v| !v.trim().is_empty()),
        app.vivo_app_key.as_ref().filter(|v| !v.trim().is_empty()),
        app.vivo_app_secret.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        if let Ok(provider) = VivoPushProvider::new(
            app_id.clone(),
            app_key.clone(),
            app_secret.clone(),
        ) {
            providers.push(Arc::new(provider));
        }
    }

    if let (Some(app_id), Some(oauth_client_id), Some(client_secret)) = (
        app.honor_app_id.as_ref().filter(|v| !v.trim().is_empty()),
        app.honor_oauth_client_id
            .as_ref()
            .filter(|v| !v.trim().is_empty()),
        app.honor_app_secret.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        if let Ok(credentials) = crate::push::vendors::honor::resolve_honor_credentials(
            Some(app_id.clone()),
            Some(oauth_client_id.clone()),
            Some(client_secret.clone()),
        ) {
            providers.push(Arc::new(HonorPushProvider::from_credentials(
                credentials,
                app.package_name.clone(),
            )));
        }
    }

    if let (Some(app_id), Some(secret)) = (
        app.meizu_app_id.as_ref().filter(|v| !v.trim().is_empty()),
        app.meizu_app_secret.as_ref().filter(|v| !v.trim().is_empty()),
    ) {
        providers.push(Arc::new(MeizuPushProvider::new(
            app_id.clone(),
            secret.clone(),
        )));
    }

    providers.push(Arc::new(OnlinePushProvider::new(
        db.outbox(),
        online_hub,
    )));

    Ok(PushHub::new(providers))
}
