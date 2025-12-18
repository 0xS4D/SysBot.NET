//! Application state management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use sysbot_base::{ConnectionConfig, SwitchConnection, WifiConnection};

/// Unique identifier for a bot connection
pub type BotId = String;

/// Information about a connected bot
#[derive(Debug, Clone, serde::Serialize)]
pub struct BotInfo {
    pub id: BotId,
    pub name: String,
    pub address: String,
    pub connected: bool,
}

/// A bot instance with its connection
pub struct Bot {
    pub info: BotInfo,
    pub connection: Box<dyn SwitchConnection>,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    /// Active bot connections
    bots: RwLock<HashMap<BotId, Arc<RwLock<Bot>>>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                bots: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Add a new bot connection
    pub async fn add_bot(&self, id: BotId, ip: &str, port: u16, name: Option<String>) -> sysbot_base::Result<BotInfo> {
        let config = ConnectionConfig::wifi(ip, port)?;
        let config = if let Some(n) = name {
            config.with_name(n)
        } else {
            config
        };

        let connection = WifiConnection::new(config.clone());
        let info = BotInfo {
            id: id.clone(),
            name: config.name.clone(),
            address: config.address.to_string(),
            connected: false,
        };

        let bot = Bot {
            info: info.clone(),
            connection: Box::new(connection),
        };

        let mut bots = self.inner.bots.write().await;
        bots.insert(id, Arc::new(RwLock::new(bot)));

        Ok(info)
    }

    /// Remove a bot connection
    pub async fn remove_bot(&self, id: &BotId) -> Option<BotInfo> {
        let mut bots = self.inner.bots.write().await;
        if let Some(bot) = bots.remove(id) {
            let bot = bot.read().await;
            Some(bot.info.clone())
        } else {
            None
        }
    }

    /// Get all bot infos
    pub async fn list_bots(&self) -> Vec<BotInfo> {
        let bots = self.inner.bots.read().await;
        let mut infos = Vec::with_capacity(bots.len());

        for bot in bots.values() {
            let bot = bot.read().await;
            infos.push(bot.info.clone());
        }

        infos
    }

    /// Get a specific bot
    pub async fn get_bot(&self, id: &BotId) -> Option<Arc<RwLock<Bot>>> {
        let bots = self.inner.bots.read().await;
        bots.get(id).cloned()
    }

    /// Connect a bot to its Switch
    pub async fn connect_bot(&self, id: &BotId) -> sysbot_base::Result<BotInfo> {
        let bot_arc = self.get_bot(id).await
            .ok_or(sysbot_base::Error::NotConnected)?;

        let mut bot = bot_arc.write().await;
        bot.connection.connect().await?;
        bot.info.connected = true;

        Ok(bot.info.clone())
    }

    /// Disconnect a bot from its Switch
    pub async fn disconnect_bot(&self, id: &BotId) -> sysbot_base::Result<BotInfo> {
        let bot_arc = self.get_bot(id).await
            .ok_or(sysbot_base::Error::NotConnected)?;

        let mut bot = bot_arc.write().await;
        bot.connection.disconnect().await?;
        bot.info.connected = false;

        Ok(bot.info.clone())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_list_bots() {
        let state = AppState::new();

        let info = state.add_bot(
            "bot1".to_string(),
            "192.168.1.100",
            6000,
            Some("TestBot".to_string()),
        ).await.unwrap();

        assert_eq!(info.id, "bot1");
        assert_eq!(info.name, "TestBot");
        assert!(!info.connected);

        let bots = state.list_bots().await;
        assert_eq!(bots.len(), 1);
        assert_eq!(bots[0].id, "bot1");
    }

    #[tokio::test]
    async fn test_remove_bot() {
        let state = AppState::new();

        state.add_bot("bot1".to_string(), "192.168.1.100", 6000, None).await.unwrap();

        let removed = state.remove_bot(&"bot1".to_string()).await;
        assert!(removed.is_some());

        let bots = state.list_bots().await;
        assert!(bots.is_empty());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_bot() {
        let state = AppState::new();
        let removed = state.remove_bot(&"nonexistent".to_string()).await;
        assert!(removed.is_none());
    }
}
