use crate::jade::acl::AclMessage;
use crate::jade::df::ServiceDescription;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CartelStatus {
    Independent,
    CartelMember,
    Traitor,
    Bankrupt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    FreeMarket,
    CartelActive,
    PriceWar,
    Monopoly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverActivity {
    Cruising,
    SeekingFuel,
    Refueling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationSnapshot {
    pub name: String,
    pub color: String,
    pub position_x: f64,
    pub price_cents: i64,
    pub wholesale_cost_cents: i64,
    pub cash_cents: i64,
    pub initial_cash_cents: i64,
    pub stock_liters: f64,
    pub liters_sold_total: f64,
    pub customers_served: u32,
    pub cartel_status: CartelStatus,
    pub agreed_price_cents: Option<i64>,
    pub is_open: bool,
    pub last_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverSnapshot {
    pub name: String,
    pub position_x: f64,
    pub direction: i8, // 1 = right, -1 = left
    pub fuel_liters: f64,
    pub max_fuel_liters: f64,
    pub reserve_threshold: f64,
    pub activity: DriverActivity,
    pub target_station: Option<String>,
    pub total_spent_cents: i64,
    pub refuels_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSnapshot {
    pub tick: u64,
    pub is_night: bool,
    pub paused: bool,
    pub speed_ms: u64,
    pub market_regime: MarketRegime,
    pub stations: Vec<StationSnapshot>,
    pub drivers: Vec<DriverSnapshot>,
    pub df_listings: Vec<ServiceDescription>,
    pub new_messages: Vec<AclMessage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum SimCommand {
    Play,
    Pause,
    Step,
    Reset,
    SetSpeed { tick_ms: u64 },
    /// Inject a cash crisis on a random or specific cartel member to trigger betrayal!
    InjectCashCrisis,
}
