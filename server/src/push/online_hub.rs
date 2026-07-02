use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

/// 在线推送长连接 hub：按 online_token 维护 WebSocket 订阅。
#[derive(Clone, Default)]
pub struct OnlinePushHub {
    inner: Arc<OnlinePushHubInner>,
}

struct OnlinePushHubInner {
    next_id: AtomicU64,
    connections: Mutex<HashMap<String, HashMap<u64, mpsc::UnboundedSender<String>>>>,
}

impl Default for OnlinePushHubInner {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            connections: Mutex::new(HashMap::new()),
        }
    }
}

pub struct OnlineSubscription {
    pub conn_id: u64,
    pub rx: mpsc::UnboundedReceiver<String>,
}

impl OnlinePushHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn subscribe(&self, push_token: &str) -> OnlineSubscription {
        let token = push_token.trim().to_string();
        let conn_id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::unbounded_channel();

        let mut map = self.inner.connections.lock().await;
        map.entry(token).or_default().insert(conn_id, tx);

        OnlineSubscription { conn_id, rx }
    }

    pub async fn unsubscribe(&self, push_token: &str, conn_id: u64) {
        let token = push_token.trim();
        let mut map = self.inner.connections.lock().await;
        if let Some(conns) = map.get_mut(token) {
            conns.remove(&conn_id);
            if conns.is_empty() {
                map.remove(token);
            }
        }
    }

    pub async fn is_connected(&self, push_token: &str) -> bool {
        let token = push_token.trim();
        let map = self.inner.connections.lock().await;
        map.get(token).is_some_and(|c| !c.is_empty())
    }

    /// 向指定 token 的所有长连接推送 JSON 文本，返回送达连接数。
    pub async fn publish(&self, push_token: &str, payload: String) -> usize {
        let token = push_token.trim();
        let map = self.inner.connections.lock().await;
        let Some(conns) = map.get(token) else {
            return 0;
        };

        let mut delivered = 0usize;
        for tx in conns.values() {
            if tx.send(payload.clone()).is_ok() {
                delivered += 1;
            }
        }
        delivered
    }
}
