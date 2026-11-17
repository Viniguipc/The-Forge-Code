pub mod driver_agent;
pub mod station_agent;
pub mod types;

use crate::jade::acl::AclMessage;
use crate::jade::ams::AgentManagementSystem;
use crate::jade::df::DirectoryFacilitator;
use crate::sim::driver_agent::{spawn_driver_agent, DriverAgentHandle, DriverConfig};
use crate::sim::station_agent::{spawn_station_agent, StationAgentHandle, StationConfig};
use crate::sim::types::{CartelStatus, MarketRegime, SimCommand, SimSnapshot};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{sleep, Duration};

struct SimWorld {
    tick: u64,
    paused: bool,
    speed_ms: u64,
    ams: AgentManagementSystem,
    df: DirectoryFacilitator,
    stations: Vec<StationAgentHandle>,
    drivers: Vec<DriverAgentHandle>,
    acl_rx: broadcast::Receiver<AclMessage>,
}

impl SimWorld {
    async fn new(acl_tx: broadcast::Sender<AclMessage>) -> Self {
        let acl_rx = acl_tx.subscribe();
        let ams = AgentManagementSystem::new(acl_tx);
        let df = DirectoryFacilitator::new();

        let station_specs = vec![
            ("Posto Alfa", "#38bdf8", 120.0, 210_000, 460, 555, 115_000, 695),
            ("Posto Beta", "#a78bfa", 310.0, 175_000, 520, 550, 120_000, 700),
            ("Posto Gama", "#34d399", 500.0, 230_000, 440, 560, 110_000, 690),
            ("Posto Delta", "#fbbf24", 690.0, 160_000, 540, 548, 125_000, 705),
            ("Posto Ômega", "#f472b6", 880.0, 195_000, 480, 552, 118_000, 695),
        ];

        let all_names: Vec<String> = station_specs.iter().map(|s| s.0.to_string()).collect();
        let mut stations = Vec::new();

        for (name, color, pos_x, cash, fixed, init_p, panic, greed) in station_specs {
            let mailbox = ams.register_agent(name).await;
            let peers: Vec<String> = all_names.iter().filter(|n| *n != name).cloned().collect();
            let handle = spawn_station_agent(
                StationConfig {
                    name: name.to_string(),
                    color: color.to_string(),
                    position_x: pos_x,
                    initial_cash_cents: cash,
                    fixed_cost_per_tick: fixed,
                    wholesale_cost_cents: 450,
                    initial_price_cents: init_p,
                    panic_threshold_cents: panic,
                    greed_price_cents: greed,
                },
                peers,
                ams.clone(),
                df.clone(),
                mailbox,
            );
            stations.push(handle);
        }

        let mut drivers = Vec::new();
        for i in 1..=20 {
            let name = format!("Motorista #{:02}", i);
            let mailbox = ams.register_agent(&name).await;
            let (start_x, direction, initial_fuel, speed, burn_rate) = {
                let mut rng = rand::thread_rng();
                (
                    rng.gen_range(40.0..960.0),
                    if rng.gen_bool(0.5) { 1 } else { -1 },
                    rng.gen_range(14.0..48.0),
                    rng.gen_range(26.0..44.0),
                    rng.gen_range(0.9..1.5),
                )
            };
            let handle = spawn_driver_agent(
                DriverConfig {
                    name,
                    start_x,
                    direction,
                    initial_fuel,
                    speed,
                    burn_rate,
                },
                ams.clone(),
                df.clone(),
                mailbox,
            );
            drivers.push(handle);
        }

        Self {
            tick: 0,
            paused: false,
            speed_ms: 450,
            ams,
            df,
            stations,
            drivers,
            acl_rx,
        }
    }

    async fn step_tick(&mut self) -> SimSnapshot {
        self.tick += 1;
        self.ams.set_tick(self.tick);

        // Night cycle: ticks 18..=27 out of every 30 ticks ("Calada da Noite")
        let cycle = self.tick % 30;
        let is_night = (18..=27).contains(&cycle);

        // 1. Tick all stations concurrently and wait for ack
        let mut acks = Vec::with_capacity(self.stations.len());
        for st in &self.stations {
            let (tx, rx) = mpsc::channel(1);
            let _ = st.tick_tx.send((self.tick, is_night, tx)).await;
            acks.push(rx);
        }
        for mut rx in acks {
            let _ = rx.recv().await;
        }

        // 2. Tick all drivers concurrently and wait for ack
        let mut d_acks = Vec::with_capacity(self.drivers.len());
        for dr in &self.drivers {
            let (tx, rx) = mpsc::channel(1);
            let _ = dr.tick_tx.send((self.tick, tx)).await;
            d_acks.push(rx);
        }
        for mut rx in d_acks {
            let _ = rx.recv().await;
        }

        // Small yield so FuelRequest -> FuelReceipt messages settle in the same tick
        tokio::task::yield_now().await;
        sleep(Duration::from_millis(8)).await;

        self.build_snapshot(is_night).await
    }

    async fn build_snapshot(&mut self, is_night: bool) -> SimSnapshot {
        let mut station_snaps = Vec::with_capacity(self.stations.len());
        for st in &self.stations {
            station_snaps.push(st.state.read().await.clone());
        }

        let mut driver_snaps = Vec::with_capacity(self.drivers.len());
        for dr in &self.drivers {
            driver_snaps.push(dr.state.read().await.clone());
        }

        let df_listings = self.df.all_listings().await;

        let mut new_messages = Vec::new();
        while let Ok(msg) = self.acl_rx.try_recv() {
            new_messages.push(msg);
        }

        let open_count = station_snaps.iter().filter(|s| s.is_open).count();
        let cartel_count = station_snaps
            .iter()
            .filter(|s| s.cartel_status == CartelStatus::CartelMember)
            .count();
        let traitor_count = station_snaps
            .iter()
            .filter(|s| s.cartel_status == CartelStatus::Traitor)
            .count();

        let market_regime = if open_count <= 1 {
            MarketRegime::Monopoly
        } else if traitor_count > 0 {
            MarketRegime::PriceWar
        } else if cartel_count >= 2 {
            MarketRegime::CartelActive
        } else {
            MarketRegime::FreeMarket
        };

        SimSnapshot {
            tick: self.tick,
            is_night,
            paused: self.paused,
            speed_ms: self.speed_ms,
            market_regime,
            stations: station_snaps,
            drivers: driver_snaps,
            df_listings,
            new_messages,
        }
    }
}

pub struct SimulationController {
    pub snapshot_tx: broadcast::Sender<SimSnapshot>,
    pub latest_snapshot: Arc<RwLock<Option<SimSnapshot>>>,
    pub cmd_tx: mpsc::Sender<SimCommand>,
}

pub fn start_simulation_engine() -> Arc<SimulationController> {
    let (snapshot_tx, _) = broadcast::channel::<SimSnapshot>(64);
    let latest_snapshot = Arc::new(RwLock::new(None));
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<SimCommand>(32);

    let snap_tx_clone = snapshot_tx.clone();
    let latest_clone = Arc::clone(&latest_snapshot);

    tokio::spawn(async move {
        let (acl_tx, _) = broadcast::channel::<AclMessage>(512);
        let mut world = SimWorld::new(acl_tx.clone()).await;

        let init_snap = world.build_snapshot(false).await;
        *latest_clone.write().await = Some(init_snap.clone());
        let _ = snap_tx_clone.send(init_snap);

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    SimCommand::Play => world.paused = false,
                    SimCommand::Pause => world.paused = true,
                    SimCommand::SetSpeed { tick_ms } => {
                        world.speed_ms = tick_ms.clamp(60, 2000);
                    }
                    SimCommand::Step => {
                        world.paused = true;
                        let snap = world.step_tick().await;
                        *latest_clone.write().await = Some(snap.clone());
                        let _ = snap_tx_clone.send(snap);
                    }
                    SimCommand::Reset => {
                        let was_paused = world.paused;
                        let speed = world.speed_ms;
                        world = SimWorld::new(acl_tx.clone()).await;
                        world.paused = was_paused;
                        world.speed_ms = speed;
                        let snap = world.build_snapshot(false).await;
                        *latest_clone.write().await = Some(snap.clone());
                        let _ = snap_tx_clone.send(snap);
                    }
                    SimCommand::InjectCashCrisis => {
                        // Pick a random open cartel member (or open station) and trigger a cash crisis
                        let mut candidates = Vec::new();
                        for st in &world.stations {
                            let s = st.state.read().await;
                            if s.is_open {
                                candidates.push((s.cartel_status == CartelStatus::CartelMember, st.crisis_tx.clone()));
                            }
                        }
                        if let Some((_, tx)) = candidates
                            .iter()
                            .find(|(is_cartel, _)| *is_cartel)
                            .or_else(|| candidates.first())
                        {
                            let _ = tx.send(()).await;
                        }
                    }
                }
            }

            if !world.paused {
                let snap = world.step_tick().await;
                *latest_clone.write().await = Some(snap.clone());
                let _ = snap_tx_clone.send(snap);
                sleep(Duration::from_millis(world.speed_ms)).await;
            } else {
                // While paused, send state heartbeat on command or sleep briefly
                sleep(Duration::from_millis(80)).await;
            }
        }
    });

    Arc::new(SimulationController {
        snapshot_tx,
        latest_snapshot,
        cmd_tx,
    })
}
