use crate::brick42::quantum::qpu_engine::{QPUEngine, QuantumBackend};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SwarmAgent {
    pub agent_id: String,
    pub domain: String,
    pub liquidity_score: f64,
    pub risk_threshold: f64,
    pub trade_history: VecDeque<TradeExecution>,
    pub consensus_votes: HashMap<String, bool>,
    pub qpu_engine: QPUEngine,
}

#[derive(Debug, Clone)]
pub struct TradeExecution {
    pub trade_id: String,
    pub instrument: String,
    pub quantity: f64,
    pub price: f64,
    pub exchange: String,
    pub timestamp_ns: u128,
    pub consensus_achieved: bool,
}

#[derive(Debug, Clone)]
pub struct LiquiditySignal {
    pub exchange: String,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub spread: f64,
    pub timestamp_ns: u128,
}

#[derive(Debug, Clone)]
pub struct SwarmConsensus {
    pub proposal_id: String,
    pub trade: TradeExecution,
    pub votes: HashMap<String, bool>,
    pub quorum_size: usize,
    pub status: ConsensusStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusStatus {
    Proposed,
    Prepared,
    Committed,
    Rejected,
}

impl SwarmAgent {
    pub fn new(agent_id: &str, domain: &str, risk_threshold: f64) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            domain: domain.to_string(),
            liquidity_score: 0.0,
            risk_threshold,
            trade_history: VecDeque::with_capacity(1000),
            consensus_votes: HashMap::new(),
            qpu_engine: QPUEngine::new(QuantumBackend::Simulated, 32),
        }
    }

    pub fn read_liquidity(&mut self, signal: &LiquiditySignal) -> f64 {
        let depth = signal.bid_volume + signal.ask_volume;
        let efficiency = 1.0 - (signal.spread / signal.ask_volume.max(1.0));
        self.liquidity_score = depth * efficiency;
        self.liquidity_score
    }

    pub fn assess_risk(&self, trade: &TradeExecution) -> bool {
        let exposure = trade.quantity * trade.price;
        let volatility = self
            .trade_history
            .iter()
            .filter(|t| t.instrument == trade.instrument)
            .map(|t| (t.price - trade.price).abs())
            .sum::<f64>()
            / self.trade_history.len().max(1) as f64;
        let risk = exposure * volatility / self.liquidity_score.max(1.0);
        risk <= self.risk_threshold
    }

    pub fn propose_trade(
        &mut self,
        instrument: &str,
        quantity: f64,
        price: f64,
        exchange: &str,
    ) -> SwarmConsensus {
        let trade = TradeExecution {
            trade_id: format!("trade_{}_{}", self.agent_id, now_ns()),
            instrument: instrument.to_string(),
            quantity,
            price,
            exchange: exchange.to_string(),
            timestamp_ns: now_ns(),
            consensus_achieved: false,
        };
        SwarmConsensus {
            proposal_id: trade.trade_id.clone(),
            trade,
            votes: HashMap::new(),
            quorum_size: 3,
            status: ConsensusStatus::Proposed,
        }
    }

    pub fn vote_on_proposal(&mut self, proposal: &mut SwarmConsensus) -> bool {
        let my_vote = self.assess_risk(&proposal.trade);
        proposal.votes.insert(self.agent_id.clone(), my_vote);
        self.consensus_votes
            .insert(proposal.proposal_id.clone(), my_vote);
        my_vote
    }

    pub fn check_consensus(&self, proposal: &SwarmConsensus) -> bool {
        let yes_votes = proposal.votes.values().filter(|&&v| v).count();
        let total = proposal.votes.len();
        let f = (total - yes_votes).min(yes_votes);
        yes_votes >= (2 * f + 1) && total >= proposal.quorum_size
    }
}

impl SwarmConsensus {
    pub fn commit(&mut self) -> bool {
        if self.status == ConsensusStatus::Prepared {
            self.status = ConsensusStatus::Committed;
            self.trade.consensus_achieved = true;
            true
        } else {
            false
        }
    }

    pub fn prepare(&mut self) -> bool {
        if self.status == ConsensusStatus::Proposed {
            self.status = ConsensusStatus::Prepared;
            true
        } else {
            false
        }
    }
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
