use crate::config::Config;
use crate::db::Database;
use crate::AppResult;

/// 启动时不再自动创建管理员或默认应用。
/// 首次部署请在管理端完成账号初始化，登录后再创建应用。
pub async fn seed(_db: &Database, _config: &Config) -> AppResult<()> {
    Ok(())
}
