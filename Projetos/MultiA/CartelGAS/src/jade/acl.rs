use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    pub name: String,
    pub kind: AgentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentKind {
    Station,
    Driver,
    DirectoryFacilitator,
    Ams,
}

impl AgentId {
    pub fn station(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: AgentKind::Station,
        }
    }

    pub fn driver(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: AgentKind::Driver,
        }
    }

    pub fn df() -> Self {
        Self {
            name: "DF (Páginas Amarelas)".to_string(),
            kind: AgentKind::DirectoryFacilitator,
        }
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// FIPA-ACL Performatives inspired by JADE's `ACLMessage`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Performative {
    Propose,
    AcceptProposal,
    RejectProposal,
    Inform,
    QueryRef,
    Request,
    Agree,
    Refuse,
}

impl Performative {
    pub fn as_str(&self) -> &'static str {
        match self {
            Performative::Propose => "PROPOSE",
            Performative::AcceptProposal => "ACCEPT-PROPOSAL",
            Performative::RejectProposal => "REJECT-PROPOSAL",
            Performative::Inform => "INFORM",
            Performative::QueryRef => "QUERY-REF",
            Performative::Request => "REQUEST",
            Performative::Agree => "AGREE",
            Performative::Refuse => "REFUSE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageContent {
    /// Private cartel proposal between stations
    CartelProposal {
        target_price_cents: i64,
        round: u64,
    },
    /// Station accepts cartel price
    CartelAccept {
        agreed_price_cents: i64,
    },
    /// Station rejects cartel price
    CartelReject {
        reason: String,
    },
    /// Cartel confirmed / active
    CartelFormed {
        agreed_price_cents: i64,
    },
    /// Angry message when a station detects betrayal
    CartelBetrayalAlert {
        traitor: String,
        undercut_price_cents: i64,
    },
    /// Driver queries DF for fuel-retail services
    DfQuery {
        service_type: String,
    },
    /// DF responds with listed stations
    DfResult {
        listings: Vec<DfListingSummary>,
    },
    /// Driver requests fuel at a station
    FuelRequest {
        liters: f64,
    },
    /// Station agrees and delivers fuel
    FuelReceipt {
        liters: f64,
        price_cents: i64,
        total_cents: i64,
    },
    /// Station refuses (bankrupt or out of stock)
    FuelRefused {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfListingSummary {
    pub agent_name: String,
    pub price_cents: i64,
    pub position_x: f64,
    pub stock_liters: f64,
    pub is_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclMessage {
    pub id: u64,
    pub tick: u64,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub performative: Performative,
    pub conversation_id: String,
    pub content: MessageContent,
    pub summary: String,
}

impl AclMessage {
    pub fn is_private_cartel(&self) -> bool {
        self.sender.kind == AgentKind::Station && self.receiver.kind == AgentKind::Station
    }

    pub fn is_df(&self) -> bool {
        self.sender.kind == AgentKind::DirectoryFacilitator
            || self.receiver.kind == AgentKind::DirectoryFacilitator
    }
}
