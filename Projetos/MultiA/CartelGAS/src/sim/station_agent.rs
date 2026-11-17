use crate::jade::acl::{AclMessage, AgentId, MessageContent, Performative};
use crate::jade::ams::{AgentManagementSystem, MailboxRx};
use crate::jade::df::{DirectoryFacilitator, ServiceDescription};
use crate::sim::types::{CartelStatus, StationSnapshot};
use rand::Rng;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct StationConfig {
    pub name: String,
    pub color: String,
    pub position_x: f64,
    pub initial_cash_cents: i64,
    pub fixed_cost_per_tick: i64,
    pub wholesale_cost_cents: i64,
    pub initial_price_cents: i64,
    pub panic_threshold_cents: i64,
    pub greed_price_cents: i64,
}

pub struct StationAgentHandle {
    pub name: String,
    pub state: Arc<RwLock<StationSnapshot>>,
    pub tick_tx: mpsc::Sender<(u64, bool, mpsc::Sender<()>)>,
    pub crisis_tx: mpsc::Sender<()>,
}

pub fn spawn_station_agent(
    cfg: StationConfig,
    peer_names: Vec<String>,
    ams: AgentManagementSystem,
    df: DirectoryFacilitator,
    mut mailbox: MailboxRx,
) -> StationAgentHandle {
    let initial_state = StationSnapshot {
        name: cfg.name.clone(),
        color: cfg.color.clone(),
        position_x: cfg.position_x,
        price_cents: cfg.initial_price_cents,
        wholesale_cost_cents: cfg.wholesale_cost_cents,
        cash_cents: cfg.initial_cash_cents,
        initial_cash_cents: cfg.initial_cash_cents,
        stock_liters: 1200.0,
        liters_sold_total: 0.0,
        customers_served: 0,
        cartel_status: CartelStatus::Independent,
        agreed_price_cents: None,
        is_open: true,
        last_action: "Aberto no mercado livre".to_string(),
    };

    let state = Arc::new(RwLock::new(initial_state));
    let state_clone = Arc::clone(&state);

    let (tick_tx, mut tick_rx) = mpsc::channel::<(u64, bool, mpsc::Sender<()>)>(4);
    let (crisis_tx, mut crisis_rx) = mpsc::channel::<()>(4);

    let my_name = cfg.name.clone();
    let handle_name = my_name.clone();
    tokio::spawn(async move {
        let my_id = AgentId::station(&my_name);
        df.register(ServiceDescription {
            service_type: "fuel-retail".to_string(),
            agent_name: my_name.clone(),
            price_cents: cfg.initial_price_cents,
            position_x: cfg.position_x,
            stock_liters: 1200.0,
            is_open: true,
            last_updated_tick: 0,
        })
        .await;

        let mut cartel_round: u64 = 0;
        let mut pending_accepts: HashSet<String> = HashSet::new();
        let mut proposal_sent_tick: Option<u64> = None;
        let mut ticks_without_sale: u64 = 0;
        let mut price_war_cooldown: u64 = 0;
        let mut traitor_cooldown: u64 = 0;

        loop {
            tokio::select! {
                Some(msg) = mailbox.recv() => {
                    handle_acl_message(
                        &my_id,
                        &cfg,
                        &peer_names,
                        &ams,
                        &df,
                        &state_clone,
                        msg,
                        &mut pending_accepts,
                        &mut ticks_without_sale,
                        &mut price_war_cooldown,
                    ).await;
                }
                Some(_) = crisis_rx.recv() => {
                    let mut st = state_clone.write().await;
                    if st.is_open {
                        st.cash_cents = (cfg.panic_threshold_cents - 15_000).max(15_000);
                        st.last_action = "CRISE DE CAIXA! Sem reserva!".to_string();
                    }
                }
                Some((tick, is_night, done_tx)) = tick_rx.recv() => {
                    // Drain any queued ACL messages before running tick logic
                    while let Ok(msg) = mailbox.try_recv() {
                        handle_acl_message(
                            &my_id,
                            &cfg,
                            &peer_names,
                            &ams,
                            &df,
                            &state_clone,
                            msg,
                            &mut pending_accepts,
                            &mut ticks_without_sale,
                            &mut price_war_cooldown,
                        ).await;
                    }

                    let mut st = state_clone.write().await;
                    if !st.is_open {
                        let _ = done_tx.send(()).await;
                        continue;
                    }

                    // 1. Fixed operating costs (rent, employees, electricity)
                    st.cash_cents -= cfg.fixed_cost_per_tick;

                    // 2. Check bankruptcy
                    if st.cash_cents <= 0 {
                        st.cash_cents = 0;
                        st.is_open = false;
                        st.cartel_status = CartelStatus::Bankrupt;
                        st.agreed_price_cents = None;
                        st.last_action = "FALÊNCIA! Bomba lacrada.".to_string();
                        drop(st);
                        df.deregister(&my_name).await;
                        let _ = done_tx.send(()).await;
                        continue;
                    }

                    // 3. Auto-restock fuel if low and has cash
                    if st.stock_liters < 250.0 {
                        let buy_liters = 800.0;
                        let cost = (buy_liters * cfg.wholesale_cost_cents as f64) as i64;
                        if st.cash_cents > cost + 10_000 {
                            st.cash_cents -= cost;
                            st.stock_liters += buy_liters;
                        } else if st.cash_cents > cfg.wholesale_cost_cents * 100 {
                            let affordable = ((st.cash_cents - 5_000) / cfg.wholesale_cost_cents).max(0) as f64;
                            if affordable >= 50.0 {
                                st.cash_cents -= (affordable * cfg.wholesale_cost_cents as f64) as i64;
                                st.stock_liters += affordable;
                            }
                        }
                    }

                    if price_war_cooldown > 0 {
                        price_war_cooldown -= 1;
                    }
                    if traitor_cooldown > 0 {
                        traitor_cooldown -= 1;
                    }

                    let current_status = st.cartel_status;
                    let current_cash = st.cash_cents;
                    let current_price = st.price_cents;
                    let agreed_price = st.agreed_price_cents;
                    drop(st);

                    // Query DF to observe competitors
                    let market = df.search("fuel-retail").await;
                    let min_competitor_price = market
                        .iter()
                        .filter(|s| s.agent_name != my_name)
                        .map(|s| s.price_cents)
                        .min();
                    let active_competitors: Vec<String> = market
                        .iter()
                        .filter(|s| s.agent_name != my_name)
                        .map(|s| s.agent_name.clone())
                        .collect();

                    match current_status {
                        CartelStatus::Independent => {
                            // If in price war cooldown, undercut competitors to survive
                            if price_war_cooldown > 0 {
                                if let Some(min_p) = min_competitor_price {
                                    let floor = cfg.wholesale_cost_cents + 5;
                                    if current_price > min_p && min_p > floor {
                                        let new_p = (min_p - 1).max(floor);
                                        let mut st = state_clone.write().await;
                                        st.price_cents = new_p;
                                        st.last_action = format!("Guerra de preços! Baixou para R$ {:.2}", new_p as f64 / 100.0);
                                        let stock = st.stock_liters;
                                        drop(st);
                                        df.modify(&my_name, new_p, stock, true, tick).await;
                                    }
                                }
                            } else if tick >= 6 && active_competitors.len() >= 1 {
                                // Propose a cartel if nobody proposed recently
                                let should_propose = match proposal_sent_tick {
                                    None => rand::thread_rng().gen_bool(0.28),
                                    Some(t) => tick.saturating_sub(t) > 10 && rand::thread_rng().gen_bool(0.25),
                                };
                                if should_propose {
                                    cartel_round += 1;
                                    proposal_sent_tick = Some(tick);
                                    pending_accepts.clear();
                                    pending_accepts.insert(my_name.clone());

                                    let target = cfg.greed_price_cents;
                                    for peer in &active_competitors {
                                        ams.send(
                                            my_id.clone(),
                                            AgentId::station(peer),
                                            Performative::Propose,
                                            format!("cartel-r{}", cartel_round),
                                            MessageContent::CartelProposal {
                                                target_price_cents: target,
                                                round: cartel_round,
                                            },
                                            format!("Bora fechar o cartel em R$ {:.2}?", target as f64 / 100.0),
                                        )
                                        .await;
                                    }
                                    let mut st = state_clone.write().await;
                                    st.last_action = format!("Propôs cartel a R$ {:.2}", target as f64 / 100.0);
                                }
                            }
                        }
                        CartelStatus::CartelMember => {
                            let agreed = agreed_price.unwrap_or(cfg.greed_price_cents);

                            // Check if someone already betrayed in the DF!
                            if let Some(min_p) = min_competitor_price {
                                if min_p < agreed {
                                    // Betrayal detected!
                                    let traitor_name = market
                                        .iter()
                                        .find(|s| s.agent_name != my_name && s.price_cents == min_p)
                                        .map(|s| s.agent_name.clone())
                                        .unwrap_or_else(|| "Desconhecido".to_string());

                                    for peer in &active_competitors {
                                        ams.send(
                                            my_id.clone(),
                                            AgentId::station(peer),
                                            Performative::Inform,
                                            format!("betrayal-{}", tick),
                                            MessageContent::CartelBetrayalAlert {
                                                traitor: traitor_name.clone(),
                                                undercut_price_cents: min_p,
                                            },
                                            format!("TRAIÇÃO! {} baixou para R$ {:.2}! Cartel desfeito!", traitor_name, min_p as f64 / 100.0),
                                        )
                                        .await;
                                    }

                                    let new_p = (min_p - 1).max(cfg.wholesale_cost_cents + 5);
                                    let mut st = state_clone.write().await;
                                    st.cartel_status = CartelStatus::Independent;
                                    st.agreed_price_cents = None;
                                    st.price_cents = new_p;
                                    st.last_action = format!("Descobriu traição de {}! Guerra!", traitor_name);
                                    let stock = st.stock_liters;
                                    drop(st);
                                    price_war_cooldown = 14;
                                    df.modify(&my_name, new_p, stock, true, tick).await;
                                    let _ = done_tx.send(()).await;
                                    continue;
                                }
                            }

                            // Should THIS station betray the cartel?
                            // Betrayal happens when cash drops below panic_threshold (especially na calada da noite)
                            // or if starved of customers for too long
                            let desperate = current_cash < cfg.panic_threshold_cents
                                || (ticks_without_sale >= 5 && is_night);
                            let night_bonus = if is_night { 0.85 } else { 0.25 };

                            if desperate && traitor_cooldown == 0 && rand::thread_rng().gen_bool(night_bonus) {
                                let undercut = agreed - 1; // 1 centavo cheaper!
                                let mut st = state_clone.write().await;
                                st.cartel_status = CartelStatus::Traitor;
                                st.agreed_price_cents = None;
                                st.price_cents = undercut;
                                st.last_action = format!(
                                    "TRAIU O CARTEL{}! Baixou 1¢ -> R$ {:.2}",
                                    if is_night { " na calada da noite" } else { "" },
                                    undercut as f64 / 100.0
                                );
                                let stock = st.stock_liters;
                                drop(st);
                                traitor_cooldown = 18;
                                df.modify(&my_name, undercut, stock, true, tick).await;
                            }
                        }
                        CartelStatus::Traitor => {
                            // Keep undercutting by 1 centavo if rivals respond
                            if let Some(min_p) = min_competitor_price {
                                let floor = cfg.wholesale_cost_cents + 5;
                                if min_p <= current_price && min_p > floor {
                                    let new_p = (min_p - 1).max(floor);
                                    let mut st = state_clone.write().await;
                                    st.price_cents = new_p;
                                    st.last_action = format!("Cortou mais 1¢ -> R$ {:.2}", new_p as f64 / 100.0);
                                    let stock = st.stock_liters;
                                    drop(st);
                                    df.modify(&my_name, new_p, stock, true, tick).await;
                                } else if price_war_cooldown == 0 && traitor_cooldown == 0 {
                                    // Reset back to Independent after price war cools
                                    let mut st = state_clone.write().await;
                                    st.cartel_status = CartelStatus::Independent;
                                    st.last_action = "Tentando recompor margem".to_string();
                                }
                            }
                        }
                        CartelStatus::Bankrupt => {}
                    }

                    ticks_without_sale += 1;
                    let _ = done_tx.send(()).await;
                }
            }
        }
    });

    StationAgentHandle {
        name: handle_name,
        state,
        tick_tx,
        crisis_tx,
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_acl_message(
    my_id: &AgentId,
    cfg: &StationConfig,
    peer_names: &[String],
    ams: &AgentManagementSystem,
    df: &DirectoryFacilitator,
    state: &Arc<RwLock<StationSnapshot>>,
    msg: AclMessage,
    pending_accepts: &mut HashSet<String>,
    ticks_without_sale: &mut u64,
    price_war_cooldown: &mut u64,
) {
    match msg.content {
        MessageContent::FuelRequest { liters } => {
            let mut st = state.write().await;
            if !st.is_open || st.stock_liters < 1.0 {
                let reason = if !st.is_open {
                    "Posto falido".to_string()
                } else {
                    "Sem combustível no tanque".to_string()
                };
                drop(st);
                ams.send(
                    my_id.clone(),
                    msg.sender,
                    Performative::Refuse,
                    msg.conversation_id,
                    MessageContent::FuelRefused { reason: reason.clone() },
                    reason,
                )
                .await;
                return;
            }

            let sold = liters.min(st.stock_liters);
            let total_cents = (sold * st.price_cents as f64).round() as i64;
            let price = st.price_cents;
            st.stock_liters -= sold;
            st.cash_cents += total_cents;
            st.liters_sold_total += sold;
            st.customers_served += 1;
            *ticks_without_sale = 0;
            let stock = st.stock_liters;
            let tick = ams.tick();
            drop(st);

            df.modify(&my_id.name, price, stock, true, tick).await;

            ams.send(
                my_id.clone(),
                msg.sender,
                Performative::Inform,
                msg.conversation_id,
                MessageContent::FuelReceipt {
                    liters: sold,
                    price_cents: price,
                    total_cents,
                },
                format!("Abasteceu {:.1}L por R$ {:.2}", sold, total_cents as f64 / 100.0),
            )
            .await;
        }
        MessageContent::CartelProposal { target_price_cents, .. } => {
            let st = state.read().await;
            let can_join = st.is_open
                && st.cartel_status == CartelStatus::Independent
                && *price_war_cooldown == 0
                && st.cash_cents >= cfg.panic_threshold_cents;
            drop(st);

            if can_join {
                let mut st = state.write().await;
                st.cartel_status = CartelStatus::CartelMember;
                st.agreed_price_cents = Some(target_price_cents);
                st.price_cents = target_price_cents;
                st.last_action = format!("Aceitou cartel! Preço R$ {:.2}", target_price_cents as f64 / 100.0);
                let stock = st.stock_liters;
                drop(st);

                df.modify(&my_id.name, target_price_cents, stock, true, ams.tick()).await;

                ams.send(
                    my_id.clone(),
                    msg.sender,
                    Performative::AcceptProposal,
                    msg.conversation_id,
                    MessageContent::CartelAccept {
                        agreed_price_cents: target_price_cents,
                    },
                    format!("Fechado! Subindo para R$ {:.2}", target_price_cents as f64 / 100.0),
                )
                .await;
            } else {
                ams.send(
                    my_id.clone(),
                    msg.sender,
                    Performative::RejectProposal,
                    msg.conversation_id,
                    MessageContent::CartelReject {
                        reason: "Sem condição agora".to_string(),
                    },
                    "Recusou proposta de cartel".to_string(),
                )
                .await;
            }
        }
        MessageContent::CartelAccept { agreed_price_cents } => {
            pending_accepts.insert(msg.sender.name.clone());
            if pending_accepts.len() >= 2 {
                let mut st = state.write().await;
                if st.is_open && st.cartel_status == CartelStatus::Independent {
                    st.cartel_status = CartelStatus::CartelMember;
                    st.agreed_price_cents = Some(agreed_price_cents);
                    st.price_cents = agreed_price_cents;
                    st.last_action = format!("Cartel ativo! R$ {:.2}", agreed_price_cents as f64 / 100.0);
                    let stock = st.stock_liters;
                    drop(st);

                    df.modify(&my_id.name, agreed_price_cents, stock, true, ams.tick()).await;

                    for peer in peer_names {
                        if peer != &my_id.name && pending_accepts.contains(peer) {
                            ams.send(
                                my_id.clone(),
                                AgentId::station(peer),
                                Performative::Inform,
                                msg.conversation_id.clone(),
                                MessageContent::CartelFormed { agreed_price_cents },
                                format!("Cartel confirmado em R$ {:.2}!", agreed_price_cents as f64 / 100.0),
                            )
                            .await;
                        }
                    }
                }
            }
        }
        MessageContent::CartelFormed { agreed_price_cents } => {
            let mut st = state.write().await;
            if st.is_open && st.cartel_status == CartelStatus::CartelMember {
                st.agreed_price_cents = Some(agreed_price_cents);
                st.price_cents = agreed_price_cents;
            }
        }
        MessageContent::CartelBetrayalAlert { traitor, undercut_price_cents } => {
            let mut st = state.write().await;
            if st.is_open && st.cartel_status == CartelStatus::CartelMember {
                let new_p = (undercut_price_cents - 1).max(cfg.wholesale_cost_cents + 5);
                st.cartel_status = CartelStatus::Independent;
                st.agreed_price_cents = None;
                st.price_cents = new_p;
                st.last_action = format!("Guerra contra {}! R$ {:.2}", traitor, new_p as f64 / 100.0);
                let stock = st.stock_liters;
                drop(st);
                *price_war_cooldown = 14;
                df.modify(&my_id.name, new_p, stock, true, ams.tick()).await;
            }
        }
        MessageContent::CartelReject { .. } => {}
        _ => {}
    }
}
