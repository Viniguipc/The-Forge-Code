use crate::jade::acl::{AclMessage, AgentId, DfListingSummary, MessageContent, Performative};
use crate::jade::ams::{AgentManagementSystem, MailboxRx};
use crate::jade::df::DirectoryFacilitator;
use crate::sim::types::{DriverActivity, DriverSnapshot};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct DriverConfig {
    pub name: String,
    pub start_x: f64,
    pub direction: i8,
    pub initial_fuel: f64,
    pub speed: f64,
    pub burn_rate: f64,
}

pub struct DriverAgentHandle {
    pub state: Arc<RwLock<DriverSnapshot>>,
    pub tick_tx: mpsc::Sender<(u64, mpsc::Sender<()>)>,
}

pub fn spawn_driver_agent(
    cfg: DriverConfig,
    ams: AgentManagementSystem,
    df: DirectoryFacilitator,
    mut mailbox: MailboxRx,
) -> DriverAgentHandle {
    let initial_state = DriverSnapshot {
        name: cfg.name.clone(),
        position_x: cfg.start_x,
        direction: cfg.direction,
        fuel_liters: cfg.initial_fuel,
        max_fuel_liters: 50.0,
        reserve_threshold: 12.0,
        activity: DriverActivity::Cruising,
        target_station: None,
        total_spent_cents: 0,
        refuels_count: 0,
    };

    let state = Arc::new(RwLock::new(initial_state));
    let state_clone = Arc::clone(&state);
    let (tick_tx, mut tick_rx) = mpsc::channel::<(u64, mpsc::Sender<()>)>(4);

    let my_name = cfg.name.clone();
    tokio::spawn(async move {
        let my_id = AgentId::driver(&my_name);
        let mut target_x: Option<f64> = None;
        let mut waiting_receipt = false;

        loop {
            tokio::select! {
                Some(msg) = mailbox.recv() => {
                    handle_driver_msg(&state_clone, msg, &mut target_x, &mut waiting_receipt).await;
                }
                Some((tick, done_tx)) = tick_rx.recv() => {
                    while let Ok(msg) = mailbox.try_recv() {
                        handle_driver_msg(&state_clone, msg, &mut target_x, &mut waiting_receipt).await;
                    }

                    let mut st = state_clone.write().await;

                    if st.activity == DriverActivity::Refueling && waiting_receipt {
                        // Still waiting or just finished
                        let _ = done_tx.send(()).await;
                        continue;
                    }

                    match st.activity {
                        DriverActivity::Cruising => {
                            st.position_x += st.direction as f64 * cfg.speed;
                            if st.position_x >= 980.0 {
                                st.position_x = 980.0;
                                st.direction = -1;
                            } else if st.position_x <= 20.0 {
                                st.position_x = 20.0;
                                st.direction = 1;
                            }
                            st.fuel_liters = (st.fuel_liters - cfg.burn_rate).max(0.5);

                            if st.fuel_liters <= st.reserve_threshold {
                                st.activity = DriverActivity::SeekingFuel;
                                let my_x = st.position_x;
                                drop(st);

                                // Query Directory Facilitator (Páginas Amarelas)
                                query_df_and_pick(&my_id, my_x, tick, &ams, &df, &state_clone, &mut target_x).await;
                                let _ = done_tx.send(()).await;
                                continue;
                            }
                        }
                        DriverActivity::SeekingFuel => {
                            let my_x = st.position_x;
                            let target_name = st.target_station.clone();
                            drop(st);

                            if target_name.is_none() || target_x.is_none() {
                                query_df_and_pick(&my_id, my_x, tick, &ams, &df, &state_clone, &mut target_x).await;
                                let _ = done_tx.send(()).await;
                                continue;
                            }

                            let tx = target_x.unwrap();
                            let tname = target_name.unwrap();
                            let diff = tx - my_x;

                            if diff.abs() <= 38.0 {
                                // Arrived at station! Send ACL Request for fuel
                                let mut st = state_clone.write().await;
                                st.position_x = tx;
                                st.activity = DriverActivity::Refueling;
                                let needed = (st.max_fuel_liters - st.fuel_liters).max(10.0);
                                drop(st);

                                waiting_receipt = true;
                                ams.send(
                                    my_id.clone(),
                                    AgentId::station(&tname),
                                    Performative::Request,
                                    format!("refuel-{}-{}", my_name, tick),
                                    MessageContent::FuelRequest { liters: needed },
                                    format!("Abastecer {:.1}L em {}", needed, tname),
                                )
                                .await;
                            } else {
                                let step = 62.0_f64.min(diff.abs()) * diff.signum();
                                let mut st = state_clone.write().await;
                                st.position_x += step;
                                st.direction = if diff >= 0.0 { 1 } else { -1 };
                                st.fuel_liters = (st.fuel_liters - cfg.burn_rate * 0.5).max(0.2);
                            }
                        }
                        DriverActivity::Refueling => {
                            waiting_receipt = false;
                            st.activity = DriverActivity::Cruising;
                        }
                    }

                    let _ = done_tx.send(()).await;
                }
            }
        }
    });

    DriverAgentHandle {
        state,
        tick_tx,
    }
}

async fn query_df_and_pick(
    my_id: &AgentId,
    my_x: f64,
    tick: u64,
    ams: &AgentManagementSystem,
    df: &DirectoryFacilitator,
    state: &Arc<RwLock<DriverSnapshot>>,
    target_x: &mut Option<f64>,
) {
    let conv_id = format!("df-{}-{}", my_id.name, tick);
    ams.emit_telemetry_msg(
        my_id.clone(),
        AgentId::df(),
        Performative::QueryRef,
        conv_id.clone(),
        MessageContent::DfQuery {
            service_type: "fuel-retail".to_string(),
        },
        "Tanque na reserva! Buscando menor preço nas Páginas Amarelas".to_string(),
    );

    let listings = df.search("fuel-retail").await;
    let summaries: Vec<DfListingSummary> = listings.iter().map(DfListingSummary::from).collect();

    // Pick lowest price; break ties by closest distance
    let best = listings.iter().min_by(|a, b| {
        a.price_cents.cmp(&b.price_cents).then_with(|| {
            let da = (a.position_x - my_x).abs();
            let db = (b.position_x - my_x).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    if let Some(chosen) = best {
        let mut st = state.write().await;
        st.target_station = Some(chosen.agent_name.clone());
        *target_x = Some(chosen.position_x);
        drop(st);

        ams.emit_telemetry_msg(
            AgentId::df(),
            my_id.clone(),
            Performative::Inform,
            conv_id,
            MessageContent::DfResult { listings: summaries },
            format!(
                "Menor preço: {} (R$ {:.2})",
                chosen.agent_name,
                chosen.price_cents as f64 / 100.0
            ),
        );
    } else {
        let mut st = state.write().await;
        st.target_station = None;
        *target_x = None;
    }
}

async fn handle_driver_msg(
    state: &Arc<RwLock<DriverSnapshot>>,
    msg: AclMessage,
    target_x: &mut Option<f64>,
    waiting_receipt: &mut bool,
) {
    match msg.content {
        MessageContent::FuelReceipt {
            liters,
            total_cents,
            ..
        } => {
            let mut st = state.write().await;
            st.fuel_liters = (st.fuel_liters + liters).min(st.max_fuel_liters);
            st.total_spent_cents += total_cents;
            st.refuels_count += 1;
            st.activity = DriverActivity::Cruising;
            st.target_station = None;
            st.direction = if rand::thread_rng().gen_bool(0.5) { 1 } else { -1 };
            *target_x = None;
            *waiting_receipt = false;
        }
        MessageContent::FuelRefused { .. } => {
            let mut st = state.write().await;
            st.activity = DriverActivity::SeekingFuel;
            st.target_station = None;
            *target_x = None;
            *waiting_receipt = false;
        }
        _ => {}
    }
}
