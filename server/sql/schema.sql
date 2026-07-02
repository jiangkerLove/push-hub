-- Push Hub PostgreSQL 完整建库脚本（纯 SQL，任意客户端可用）
--
-- 步骤：
--   1. 连到维护库 postgres，执行下方「建库」一段（库已存在则跳过）
--   2. 切换连接到 push_hub，再执行下方「表结构」全部语句
--
-- psql 示例：
--   psql -U postgres -c "CREATE DATABASE push_hub WITH ENCODING 'UTF8' TEMPLATE template0;"
--   psql -U postgres -d push_hub -f server/sql/schema.sql
--   （第二次执行时本文件顶部建库语句已注释，直接跑表结构即可）
--
-- 连接串：
--   DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/push_hub

-- ========== 建库（连 postgres 时执行；库已存在可整段注释掉）==========
-- CREATE DATABASE push_hub
--     WITH
--     ENCODING = 'UTF8'
--     TEMPLATE = template0;

-- ========== 表结构（必须已连接到 push_hub）==========

CREATE TABLE IF NOT EXISTS apps (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    package_name            TEXT NOT NULL DEFAULT '',
    ios_bundle_id           TEXT,
    harmony_bundle_name     TEXT,
    description             TEXT,
    server_base_url         TEXT,
    push_api_key            TEXT NOT NULL,
    xiaomi_app_id           TEXT,
    xiaomi_app_key          TEXT,
    xiaomi_channel_id       TEXT,
    xiaomi_app_secret       TEXT,
    huawei_app_id           TEXT,
    huawei_oauth_client_id  TEXT,
    huawei_app_secret       TEXT,
    oppo_app_key            TEXT,
    oppo_app_secret         TEXT,
    oppo_master_secret      TEXT,
    vivo_app_id             TEXT,
    vivo_app_key            TEXT,
    vivo_app_secret         TEXT,
    honor_app_id            TEXT,
    honor_oauth_client_id   TEXT,
    honor_app_secret        TEXT,
    meizu_app_id            TEXT,
    meizu_app_key           TEXT,
    meizu_app_secret        TEXT,
    online_push_fallback_secs BIGINT NOT NULL DEFAULT 90,
    online_message_cache_secs BIGINT NOT NULL DEFAULT 86400,
    is_default              BOOLEAN NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_users (
    id                  TEXT PRIMARY KEY,
    username            TEXT NOT NULL UNIQUE,
    password_hash       TEXT NOT NULL,
    is_owner            BOOLEAN NOT NULL DEFAULT FALSE,
    display_time_zone   TEXT,
    password_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS devices (
    id              TEXT PRIMARY KEY,
    app_id          TEXT NOT NULL DEFAULT '',
    package_name    TEXT NOT NULL,
    platform        TEXT NOT NULL,
    push_token      TEXT NOT NULL,
    online_token    TEXT,
    last_online_at  TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (platform, push_token)
);

CREATE INDEX IF NOT EXISTS idx_devices_app_id ON devices (app_id);
CREATE INDEX IF NOT EXISTS idx_devices_online_token ON devices (online_token);
CREATE INDEX IF NOT EXISTS idx_devices_package_name ON devices (package_name);

CREATE TABLE IF NOT EXISTS push_templates (
    id                  TEXT PRIMARY KEY,
    app_id              TEXT NOT NULL DEFAULT '',
    name                TEXT NOT NULL,
    kind                TEXT NOT NULL DEFAULT 'private',
    content_mode        TEXT NOT NULL DEFAULT 'compose',
    title               TEXT NOT NULL,
    body                TEXT NOT NULL,
    channels            JSONB NOT NULL DEFAULT '{}'::jsonb,
    click_action        JSONB NOT NULL DEFAULT '{}'::jsonb,
    message_cache_days  BIGINT NOT NULL DEFAULT 7,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_templates_app_id ON push_templates (app_id);

CREATE TABLE IF NOT EXISTS push_channels (
    id              TEXT PRIMARY KEY,
    app_id          TEXT NOT NULL,
    platform        TEXT NOT NULL,
    name            TEXT NOT NULL,
    code            TEXT NOT NULL,
    description     TEXT,
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_channels_app_platform
    ON push_channels (app_id, platform);

CREATE TABLE IF NOT EXISTS push_outbox (
    id                  TEXT PRIMARY KEY,
    batch_id            TEXT NOT NULL,
    push_token          TEXT NOT NULL,
    package_name        TEXT NOT NULL,
    title               TEXT NOT NULL,
    body                TEXT NOT NULL,
    payload             JSONB NOT NULL DEFAULT '{}'::jsonb,
    delivery_mode       TEXT NOT NULL DEFAULT 'notification',
    channels_json       JSONB,
    click_action_json   JSONB,
    template_vars_json  JSONB,
    notify_id           INTEGER,
    fallback_platform   TEXT,
    fallback_token      TEXT,
    fallback_sent_at    TIMESTAMPTZ,
    expires_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_push_outbox_token_pending
    ON push_outbox (push_token, created_at)
    WHERE delivered_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_push_outbox_batch_id ON push_outbox (batch_id);

CREATE INDEX IF NOT EXISTS idx_push_outbox_fallback
    ON push_outbox (created_at)
    WHERE delivered_at IS NULL
      AND fallback_sent_at IS NULL
      AND fallback_platform IS NOT NULL;

CREATE TABLE IF NOT EXISTS push_jobs (
    id              TEXT PRIMARY KEY,
    app_id          TEXT NOT NULL,
    template_id     TEXT NOT NULL,
    template_name   TEXT NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL,
    total_targets   BIGINT NOT NULL DEFAULT 0,
    success_count   BIGINT NOT NULL DEFAULT 0,
    failed_count    BIGINT NOT NULL DEFAULT 0,
    batch_id        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_jobs_app_created
    ON push_jobs (app_id, created_at DESC);

CREATE TABLE IF NOT EXISTS push_job_targets (
    id                  TEXT PRIMARY KEY,
    job_id              TEXT NOT NULL,
    device_id           TEXT,
    platform            TEXT NOT NULL,
    push_token          TEXT NOT NULL,
    route_decision      TEXT NOT NULL DEFAULT 'pending',
    final_status        TEXT NOT NULL DEFAULT 'pending',
    final_channel       TEXT,
    outbox_id           TEXT,
    vendor_message_id   TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_job_targets_job_id ON push_job_targets (job_id);

CREATE TABLE IF NOT EXISTS push_job_events (
    id              TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL,
    target_id       TEXT,
    stage           TEXT NOT NULL,
    status          TEXT NOT NULL,
    platform        TEXT,
    detail          TEXT NOT NULL,
    metadata        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_push_job_events_job_id ON push_job_events (job_id);

CREATE TABLE IF NOT EXISTS push_outbox_trace (
    outbox_id   TEXT PRIMARY KEY,
    job_id      TEXT NOT NULL,
    target_id   TEXT
);

CREATE INDEX IF NOT EXISTS idx_push_outbox_trace_job_id ON push_outbox_trace (job_id);

-- 增量迁移：已有库重复执行 schema.sql 时补齐新字段（索引须在 ALTER 之后）
ALTER TABLE push_job_targets ADD COLUMN IF NOT EXISTS outbox_id TEXT;
ALTER TABLE push_job_targets ADD COLUMN IF NOT EXISTS vendor_message_id TEXT;
ALTER TABLE push_outbox_trace ADD COLUMN IF NOT EXISTS target_id TEXT;
CREATE INDEX IF NOT EXISTS idx_push_job_targets_vendor_message_id
    ON push_job_targets (vendor_message_id)
    WHERE vendor_message_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_push_outbox_trace_target_id ON push_outbox_trace (target_id);

ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS is_owner BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS display_time_zone TEXT;
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS password_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
UPDATE admin_users
SET is_owner = TRUE
WHERE id = (
    SELECT id FROM admin_users ORDER BY created_at ASC LIMIT 1
)
AND NOT EXISTS (SELECT 1 FROM admin_users WHERE is_owner = TRUE);

ALTER TABLE apps ADD COLUMN IF NOT EXISTS push_api_key TEXT;
UPDATE apps
SET push_api_key = 'phk_' || replace(gen_random_uuid()::text, '-', '')
WHERE push_api_key IS NULL OR push_api_key = '';
