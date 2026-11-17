use crate::jade::acl::DfListingSummary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub service_type: String,
    pub agent_name: String,
    pub price_cents: i64,
    pub position_x: f64,
    pub stock_liters: f64,
    pub is_open: bool,
    pub last_updated_tick: u64,
}

impl From<&ServiceDescription> for DfListingSummary {
    fn from(s: &ServiceDescription) -> Self {
        Self {
            agent_name: s.agent_name.clone(),
            price_cents: s.price_cents,
            position_x: s.position_x,
            stock_liters: s.stock_liters,
            is_open: s.is_open,
        }
    }
}

/// JADE-inspired Directory Facilitator (Yellow Pages)
#[derive(Clone, Default)]
pub struct DirectoryFacilitator {
    services: Arc<RwLock<HashMap<String, ServiceDescription>>>,
}

impl DirectoryFacilitator {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, desc: ServiceDescription) {
        let mut map = self.services.write().await;
        map.insert(desc.agent_name.clone(), desc);
    }

    pub async fn modify(&self, agent_name: &str, price_cents: i64, stock_liters: f64, is_open: bool, tick: u64) {
        let mut map = self.services.write().await;
        if let Some(entry) = map.get_mut(agent_name) {
            entry.price_cents = price_cents;
            entry.stock_liters = stock_liters;
            entry.is_open = is_open;
            entry.last_updated_tick = tick;
        }
    }

    pub async fn deregister(&self, agent_name: &str) {
        let mut map = self.services.write().await;
        map.remove(agent_name);
    }

    pub async fn search(&self, service_type: &str) -> Vec<ServiceDescription> {
        let map = self.services.read().await;
        let mut results: Vec<ServiceDescription> = map
            .values()
            .filter(|s| s.service_type == service_type && s.is_open && s.stock_liters > 1.0)
            .cloned()
            .collect();
        results.sort_by_key(|s| s.price_cents);
        results
    }

    pub async fn all_listings(&self) -> Vec<ServiceDescription> {
        let map = self.services.read().await;
        let mut results: Vec<ServiceDescription> = map.values().cloned().collect();
        results.sort_by(|a, b| a.position_x.partial_cmp(&b.position_x).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}
