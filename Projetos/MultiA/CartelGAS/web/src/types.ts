export type CartelStatus = 'Independent' | 'CartelMember' | 'Traitor' | 'Bankrupt';
export type MarketRegime = 'FreeMarket' | 'CartelActive' | 'PriceWar' | 'Monopoly';
export type DriverActivity = 'Cruising' | 'SeekingFuel' | 'Refueling';
export type AgentKind = 'Station' | 'Driver' | 'DirectoryFacilitator' | 'Ams';

export type Performative =
  | 'Propose'
  | 'AcceptProposal'
  | 'RejectProposal'
  | 'Inform'
  | 'QueryRef'
  | 'Request'
  | 'Agree'
  | 'Refuse';

export interface AgentId {
  name: string;
  kind: AgentKind;
}

export interface AclMessage {
  id: number;
  tick: number;
  sender: AgentId;
  receiver: AgentId;
  performative: Performative;
  conversation_id: string;
  content: {
    type: string;
    data: unknown;
  };
  summary: string;
}

export interface StationSnapshot {
  name: string;
  color: string;
  position_x: number;
  price_cents: number;
  wholesale_cost_cents: number;
  cash_cents: number;
  initial_cash_cents: number;
  stock_liters: number;
  liters_sold_total: number;
  customers_served: number;
  cartel_status: CartelStatus;
  agreed_price_cents: number | null;
  is_open: boolean;
  last_action: string;
}

export interface DriverSnapshot {
  name: string;
  position_x: number;
  direction: number;
  fuel_liters: number;
  max_fuel_liters: number;
  reserve_threshold: number;
  activity: DriverActivity;
  target_station: string | null;
  total_spent_cents: number;
  refuels_count: number;
}

export interface ServiceDescription {
  service_type: string;
  agent_name: string;
  price_cents: number;
  position_x: number;
  stock_liters: number;
  is_open: boolean;
  last_updated_tick: number;
}

export interface SimSnapshot {
  tick: number;
  is_night: boolean;
  paused: boolean;
  speed_ms: number;
  market_regime: MarketRegime;
  stations: StationSnapshot[];
  drivers: DriverSnapshot[];
  df_listings: ServiceDescription[];
  new_messages: AclMessage[];
}

export type SimCommand =
  | { cmd: 'Play' }
  | { cmd: 'Pause' }
  | { cmd: 'Step' }
  | { cmd: 'Reset' }
  | { cmd: 'SetSpeed'; payload: { tick_ms: number } }
  | { cmd: 'InjectCashCrisis' };
