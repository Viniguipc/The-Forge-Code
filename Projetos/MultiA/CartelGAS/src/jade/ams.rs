use crate::jade::acl::{AclMessage, AgentId, MessageContent, Performative};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

pub type MailboxTx = mpsc::UnboundedSender<AclMessage>;
pub type MailboxRx = mpsc::UnboundedReceiver<AclMessage>;

/// JADE-inspired Agent Management System & Message Transport Service
#[derive(Clone)]
pub struct AgentManagementSystem {
    mailboxes: Arc<RwLock<HashMap<String, MailboxTx>>>,
    next_msg_id: Arc<AtomicU64>,
    current_tick: Arc<AtomicU64>,
    event_tx: broadcast::Sender<AclMessage>,
}

impl AgentManagementSystem {
    pub fn new(event_tx: broadcast::Sender<AclMessage>) -> Self {
        Self {
            mailboxes: Arc::new(RwLock::new(HashMap::new())),
            next_msg_id: Arc::new(AtomicU64::new(1)),
            current_tick: Arc::new(AtomicU64::new(0)),
            event_tx,
        }
    }

    pub fn set_tick(&self, tick: u64) {
        self.current_tick.store(tick, Ordering::Relaxed);
    }

    pub fn tick(&self) -> u64 {
        self.current_tick.load(Ordering::Relaxed)
    }

    pub async fn register_agent(&self, name: impl Into<String>) -> MailboxRx {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut map = self.mailboxes.write().await;
        map.insert(name.into(), tx);
        rx
    }

    pub async fn deregister_agent(&self, name: &str) {
        let mut map = self.mailboxes.write().await;
        map.remove(name);
    }

    pub async fn send(
        &self,
        sender: AgentId,
        receiver: AgentId,
        performative: Performative,
        conversation_id: impl Into<String>,
        content: MessageContent,
        summary: impl Into<String>,
    ) {
        let msg = AclMessage {
            id: self.next_msg_id.fetch_add(1, Ordering::Relaxed),
            tick: self.tick(),
            sender,
            receiver: receiver.clone(),
            performative,
            conversation_id: conversation_id.into(),
            content,
            summary: summary.into(),
        };

        // Emit to telemetry/UI observer
        let _ = self.event_tx.send(msg.clone());

        // Deliver to receiver's mailbox if registered
        let map = self.mailboxes.read().await;
        if let Some(tx) = map.get(&receiver.name) {
            let _ = tx.send(msg);
        }
    }

    /// Log a DF interaction as a FIPA-ACL message pair for full observability
    pub fn emit_telemetry_msg(
        &self,
        sender: AgentId,
        receiver: AgentId,
        performative: Performative,
        conversation_id: impl Into<String>,
        content: MessageContent,
        summary: impl Into<String>,
    ) {
        let msg = AclMessage {
            id: self.next_msg_id.fetch_add(1, Ordering::Relaxed),
            tick: self.tick(),
            sender,
            receiver,
            performative,
            conversation_id: conversation_id.into(),
            content,
            summary: summary.into(),
        };
        let _ = self.event_tx.send(msg);
    }
}
