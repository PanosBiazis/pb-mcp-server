//! # Multi-Agent Consensus / Voting System
//!
//! Decision-making system where multiple AI agents can vote on proposals
//! and reach consensus using various voting strategies.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of a proposal throughout its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProposalStatus {
    /// Proposal is accepting votes.
    Open,
    /// Proposal has been closed and tallied.
    Closed,
    /// Proposal voting window has passed without a tally.
    Expired,
    /// Proposal was withdrawn before the deadline.
    Cancelled,
}

/// Supported consensus / voting strategies.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConsensusStrategy {
    /// Option with >50 % of cast votes wins.
    SimpleMajority,
    /// Option with >2/3 of cast votes wins.
    SuperMajority,
    /// All voters must agree on the same option.
    Unanimous,
    /// Like SimpleMajority but each voter carries an explicit weight
    /// (stored in [`ConsensusEngine::voter_weights`]).
    WeightedVoting,
    /// Instant-runoff voting – voters rank the options and the option
    /// with the fewest first-preference votes is eliminated each round
    /// until one option has a majority.
    RankedChoice,
    /// Voters may delegate their vote to another agent; once delegations
    /// are resolved, SimpleMajority is applied.
    Delegation,
}

/// A single vote cast by an agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vote {
    /// Agent ID of the voter.
    pub voter: String,
    /// The chosen option (or first preference for RankedChoice).
    pub choice: String,
    /// Confidence level 0.0 – 1.0.
    pub confidence: f64,
    /// Optional free-text rationale.
    pub rationale: Option<String>,
    /// Timestamp (ms since epoch).
    pub timestamp: u64,
    /// Ranked preferences (only used for RankedChoice).
    pub rankings: Vec<String>,
}

/// A proposal that agents may vote on.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Proposal {
    /// Unique identifier.
    pub id: String,
    /// Agent that proposed the vote.
    pub proposer: String,
    /// Short topic / title.
    pub topic: String,
    /// Longer description.
    pub description: String,
    /// Available choices.
    pub options: Vec<String>,
    /// Creation timestamp (ms since epoch).
    pub created_at: u64,
    /// Voting window in milliseconds.
    pub deadline_ms: u64,
    /// Strategy used to determine the winner.
    pub strategy: ConsensusStrategy,
    /// Current lifecycle status.
    pub status: ProposalStatus,
    /// Votes keyed by voter ID.
    pub votes: HashMap<String, Vote>,
    /// Computed result (set after tally / close).
    pub result: Option<ConsensusResult>,
}

impl Proposal {
    /// Absolute deadline as a millisecond timestamp.
    pub fn deadline_timestamp(&self) -> u64 {
        self.created_at + self.deadline_ms
    }

    /// Whether the voting deadline has passed relative to `now_ms`.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.deadline_timestamp()
    }
}

/// Outcome of a consensus tally.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsensusResult {
    /// The winning option, if one was determined.
    pub winning_option: Option<String>,
    /// Per-option tally (raw count for most strategies, weighted total
    /// for WeightedVoting).
    pub tally: HashMap<String, u64>,
    /// Fraction of eligible voters that cast a vote.
    pub participation_rate: f64,
    /// Whether consensus was reached under the chosen strategy.
    pub consensus_reached: bool,
    /// Agents whose votes decided the outcome.
    pub deciding_voters: Vec<String>,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The consensus engine manages proposals, votes, delegations and tallying.
pub struct ConsensusEngine {
    proposals: HashMap<String, Proposal>,
    /// Weights used by the `WeightedVoting` strategy (default 1.0).
    pub voter_weights: HashMap<String, f64>,
    /// Delegation map: from_voter → to_voter. Only consulted under the
    /// `Delegation` strategy.
    pub delegations: HashMap<String, String>,
    /// Clock function – defaults to a closure over
    /// `std::time::SystemTime::now()`.
    now_fn: Box<dyn Fn() -> u64 + Send + Sync>,
}

impl std::fmt::Debug for ConsensusEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsensusEngine")
            .field("proposals", &self.proposals)
            .field("voter_weights", &self.voter_weights)
            .field("delegations", &self.delegations)
            .field("now_fn", &"<closure>")
            .finish()
    }
}

impl Clone for ConsensusEngine {
    fn clone(&self) -> Self {
        Self {
            proposals: self.proposals.clone(),
            voter_weights: self.voter_weights.clone(),
            delegations: self.delegations.clone(),
            now_fn: Box::new(real_now_ms),
        }
    }
}

impl ConsensusEngine {
    /// Create a new engine using the real system clock.
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            voter_weights: HashMap::new(),
            delegations: HashMap::new(),
            now_fn: Box::new(real_now_ms),
        }
    }

    /// Create an engine with a deterministic clock (useful for tests).
    pub fn new_with_clock(now_fn: Box<dyn Fn() -> u64 + Send + Sync>) -> Self {
        Self {
            proposals: HashMap::new(),
            voter_weights: HashMap::new(),
            delegations: HashMap::new(),
            now_fn,
        }
    }

    // -- Proposal management ------------------------------------------------

    /// Create a new proposal and register it in the engine.
    pub fn create_proposal(
        &mut self,
        proposer: impl Into<String>,
        topic: impl Into<String>,
        description: impl Into<String>,
        options: Vec<String>,
        deadline_ms: u64,
        strategy: ConsensusStrategy,
    ) -> Proposal {
        let now = (self.now_fn)();
        let id = format!("prop-{}", simple_id());
        let proposal = Proposal {
            id: id.clone(),
            proposer: proposer.into(),
            topic: topic.into(),
            description: description.into(),
            options,
            created_at: now,
            deadline_ms,
            strategy,
            status: ProposalStatus::Open,
            votes: HashMap::new(),
            result: None,
        };
        self.proposals.insert(id, proposal.clone());
        proposal
    }

    /// Cast a vote on an open proposal.
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter_id: impl Into<String>,
        choice: impl Into<String>,
        confidence: f64,
        rationale: Option<String>,
    ) -> Result<(), String> {
        let voter = voter_id.into();
        let choice_str = choice.into();
        self.cast_vote_inner(
            proposal_id,
            voter,
            choice_str,
            confidence,
            rationale,
            vec![],
        )
    }

    /// Cast a ranked vote (used by the RankedChoice strategy).
    pub fn cast_ranked_vote(
        &mut self,
        proposal_id: &str,
        voter_id: impl Into<String>,
        rankings: Vec<String>,
        confidence: f64,
        rationale: Option<String>,
    ) -> Result<(), String> {
        let voter = voter_id.into();
        let first_choice = rankings
            .first()
            .ok_or_else(|| "rankings must contain at least one option".to_string())?
            .clone();
        self.cast_vote_inner(
            proposal_id,
            voter,
            first_choice,
            confidence,
            rationale,
            rankings,
        )
    }

    fn cast_vote_inner(
        &mut self,
        proposal_id: &str,
        voter: String,
        choice: String,
        confidence: f64,
        rationale: Option<String>,
        rankings: Vec<String>,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;

        if proposal.status != ProposalStatus::Open {
            return Err(format!(
                "proposal '{}' is not open for voting (status: {:?})",
                proposal_id, proposal.status
            ));
        }

        let now = (self.now_fn)();
        if proposal.is_expired(now) {
            proposal.status = ProposalStatus::Expired;
            return Err(format!(
                "proposal '{}' has expired (deadline {} ms, now {} ms)",
                proposal_id,
                proposal.deadline_timestamp(),
                now
            ));
        }

        if proposal.votes.contains_key(&voter) {
            return Err(format!(
                "voter '{}' has already voted on proposal '{}'",
                voter, proposal_id
            ));
        }

        if !proposal.options.contains(&choice) {
            return Err(format!(
                "choice '{}' is not a valid option for proposal '{}'",
                choice, proposal_id
            ));
        }

        // Validate rankings if present
        if !rankings.is_empty() {
            for r in &rankings {
                if !proposal.options.contains(r) {
                    return Err(format!(
                        "ranked option '{}' is not a valid option for proposal '{}'",
                        r, proposal_id
                    ));
                }
            }
        }

        let vote = Vote {
            voter: voter.clone(),
            choice,
            confidence: confidence.clamp(0.0, 1.0),
            rationale,
            timestamp: now,
            rankings,
        };

        proposal.votes.insert(voter, vote);
        Ok(())
    }

    /// Delegate a vote from one agent to another (Delegation strategy only).
    pub fn delegate_vote(
        &mut self,
        proposal_id: &str,
        from_voter: impl Into<String>,
        to_voter: impl Into<String>,
    ) -> Result<(), String> {
        let from = from_voter.into();
        let to = to_voter.into();

        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;

        if proposal.strategy != ConsensusStrategy::Delegation {
            return Err(format!(
                "delegation is only allowed under the Delegation strategy (proposal '{}' uses {:?})",
                proposal_id, proposal.strategy
            ));
        }

        if proposal.status != ProposalStatus::Open {
            return Err(format!(
                "proposal '{}' is not open for delegation",
                proposal_id
            ));
        }

        if from == to {
            return Err("cannot delegate a vote to oneself".to_string());
        }

        // Prevent circular delegation chains
        let mut visited = vec![from.clone()];
        let mut current = to.clone();
        while let Some(next) = self.delegations.get(&current) {
            if visited.contains(next) {
                return Err("circular delegation chain detected".to_string());
            }
            visited.push(current.clone());
            current = next.clone();
        }

        self.delegations.insert(from, to);
        Ok(())
    }

    // -- Tallying -----------------------------------------------------------

    /// Compute (or re-compute) the consensus result for a proposal.
    pub fn tally(&self, proposal_id: &str) -> Result<ConsensusResult, String> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;

        if proposal.votes.is_empty() {
            return Ok(ConsensusResult {
                winning_option: None,
                tally: HashMap::new(),
                participation_rate: 0.0,
                consensus_reached: false,
                deciding_voters: vec![],
            });
        }

        let result = match &proposal.strategy {
            ConsensusStrategy::SimpleMajority => self.tally_simple_majority(proposal),
            ConsensusStrategy::SuperMajority => self.tally_super_majority(proposal),
            ConsensusStrategy::Unanimous => self.tally_unanimous(proposal),
            ConsensusStrategy::WeightedVoting => self.tally_weighted(proposal),
            ConsensusStrategy::RankedChoice => self.tally_ranked_choice(proposal),
            ConsensusStrategy::Delegation => self.tally_delegation(proposal),
        };

        Ok(result)
    }

    /// Force-close a proposal, tally it, and persist the result.
    pub fn close_proposal(&mut self, proposal_id: &str) -> Result<ConsensusResult, String> {
        // First, compute the result (borrows self immutably).
        let result = self.tally(proposal_id)?;
        // Then mutably update the proposal.
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| format!("proposal '{}' not found", proposal_id))?;
        proposal.status = ProposalStatus::Closed;
        proposal.result = Some(result.clone());
        Ok(result)
    }

    /// Expire all open proposals whose deadline has passed.
    pub fn expire_proposals(&mut self) -> Vec<Proposal> {
        let now = (self.now_fn)();
        let mut expired = Vec::new();

        for proposal in self.proposals.values_mut() {
            if proposal.status == ProposalStatus::Open && proposal.is_expired(now) {
                proposal.status = ProposalStatus::Expired;
                expired.push(proposal.clone());
            }
        }

        expired
    }

    /// Return references to all open proposals.
    pub fn list_open(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Open)
            .collect()
    }

    /// Look up a single proposal by ID.
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    // -- Strategy implementations -------------------------------------------

    fn tally_simple_majority(&self, proposal: &Proposal) -> ConsensusResult {
        let mut counts: HashMap<String, u64> = HashMap::new();
        let mut deciding: Vec<String> = Vec::new();
        let total_votes: u64 = proposal.votes.len() as u64;

        for vote in proposal.votes.values() {
            *counts.entry(vote.choice.clone()).or_insert(0) += 1;
            deciding.push(vote.voter.clone());
        }

        let threshold = (total_votes as f64) / 2.0;
        let (winning, consensus) = find_winner_above_threshold(&counts, threshold);

        ConsensusResult {
            winning_option: winning,
            tally: counts,
            participation_rate: 1.0, // everyone who voted
            consensus_reached: consensus,
            deciding_voters: deciding,
        }
    }

    fn tally_super_majority(&self, proposal: &Proposal) -> ConsensusResult {
        let mut counts: HashMap<String, u64> = HashMap::new();
        let mut deciding: Vec<String> = Vec::new();
        let total_votes: u64 = proposal.votes.len() as u64;

        for vote in proposal.votes.values() {
            *counts.entry(vote.choice.clone()).or_insert(0) += 1;
            deciding.push(vote.voter.clone());
        }

        let threshold = (2.0 / 3.0) * (total_votes as f64);
        let (winning, consensus) = find_winner_above_threshold(&counts, threshold);

        ConsensusResult {
            winning_option: winning,
            tally: counts,
            participation_rate: 1.0,
            consensus_reached: consensus,
            deciding_voters: deciding,
        }
    }

    fn tally_unanimous(&self, proposal: &Proposal) -> ConsensusResult {
        let mut counts: HashMap<String, u64> = HashMap::new();
        let mut deciding: Vec<String> = Vec::new();
        let total_votes: u64 = proposal.votes.len() as u64;

        for vote in proposal.votes.values() {
            *counts.entry(vote.choice.clone()).or_insert(0) += 1;
            deciding.push(vote.voter.clone());
        }

        // Unanimous: all votes must be for the same option.
        let winning = if counts.len() == 1 {
            counts.keys().next().cloned()
        } else {
            None
        };

        ConsensusResult {
            winning_option: winning.clone(),
            tally: counts,
            participation_rate: 1.0,
            consensus_reached: winning.is_some() && total_votes > 0,
            deciding_voters: deciding,
        }
    }

    fn tally_weighted(&self, proposal: &Proposal) -> ConsensusResult {
        let mut weighted_counts: HashMap<String, u64> = HashMap::new();
        let mut deciding: Vec<String> = Vec::new();
        let mut total_weight: f64 = 0.0;

        for vote in proposal.votes.values() {
            let weight = self.voter_weights.get(&vote.voter).copied().unwrap_or(1.0);
            total_weight += weight;
            *weighted_counts.entry(vote.choice.clone()).or_insert(0u64) += (weight * 1000.0) as u64; // store as fixed-point
            deciding.push(vote.voter.clone());
        }

        // Convert back for the threshold check.
        let threshold = total_weight / 2.0;
        let threshold_fp = (threshold * 1000.0) as u64;

        let (winning, consensus) = find_winner_above_threshold_fp(&weighted_counts, threshold_fp);

        ConsensusResult {
            winning_option: winning,
            tally: weighted_counts,
            participation_rate: 1.0,
            consensus_reached: consensus,
            deciding_voters: deciding,
        }
    }

    fn tally_ranked_choice(&self, proposal: &Proposal) -> ConsensusResult {
        let options = &proposal.options;
        let voters: Vec<&Vote> = proposal.votes.values().collect();
        let total_voters = voters.len();

        if total_voters == 0 {
            return ConsensusResult {
                winning_option: None,
                tally: HashMap::new(),
                participation_rate: 0.0,
                consensus_reached: false,
                deciding_voters: vec![],
            };
        }

        // Build per-voter ranking lists (fall back to [choice] if no
        // explicit rankings were supplied).
        let voter_rankings: Vec<Vec<String>> = voters
            .iter()
            .map(|v| {
                if v.rankings.is_empty() {
                    vec![v.choice.clone()]
                } else {
                    v.rankings.clone()
                }
            })
            .collect();

        // Remaining options still in the running.
        let mut remaining: Vec<String> = options.clone();

        loop {
            // Count first preferences among remaining options.
            let mut round_counts: HashMap<String, u64> = HashMap::new();
            for opt in &remaining {
                round_counts.insert(opt.clone(), 0);
            }

            for rankings in &voter_rankings {
                // Find the highest-ranked option still in the running.
                for pref in rankings {
                    if remaining.contains(pref) {
                        if let Some(count) = round_counts.get_mut(pref) {
                            *count += 1;
                        }
                        break;
                    }
                }
            }

            let total: u64 = round_counts.values().sum();
            let majority_threshold = (total as f64) / 2.0;

            // Check if any option has >50 %.
            let mut best_opt: Option<String> = None;
            let mut best_count: u64 = 0;
            for (opt, &count) in &round_counts {
                if count > best_count {
                    best_count = count;
                    best_opt = Some(opt.clone());
                }
            }

            if (best_count as f64) > majority_threshold {
                // Winner found.
                let deciding: Vec<String> = voters.iter().map(|v| v.voter.clone()).collect();
                return ConsensusResult {
                    winning_option: best_opt,
                    tally: round_counts,
                    participation_rate: 1.0,
                    consensus_reached: true,
                    deciding_voters: deciding,
                };
            }

            // No majority – eliminate the option with the fewest votes.
            let mut min_count = u64::MAX;
            let mut to_eliminate: Option<String> = None;
            for (opt, &count) in &round_counts {
                if count < min_count {
                    min_count = count;
                    to_eliminate = Some(opt.clone());
                }
            }

            match to_eliminate {
                Some(ref opt) => {
                    remaining.retain(|o| o != opt);
                }
                None => break,
            }

            // If only one option remains it wins by elimination.
            if remaining.len() == 1 {
                let winner = remaining[0].clone();
                let deciding: Vec<String> = voters.iter().map(|v| v.voter.clone()).collect();
                return ConsensusResult {
                    winning_option: Some(winner.clone()),
                    tally: round_counts,
                    participation_rate: 1.0,
                    consensus_reached: true,
                    deciding_voters: deciding,
                };
            }

            if remaining.is_empty() {
                break;
            }
        }

        // No winner could be determined (should be rare).
        let deciding: Vec<String> = voters.iter().map(|v| v.voter.clone()).collect();
        ConsensusResult {
            winning_option: None,
            tally: HashMap::new(),
            participation_rate: 1.0,
            consensus_reached: false,
            deciding_voters: deciding,
        }
    }

    fn tally_delegation(&self, proposal: &Proposal) -> ConsensusResult {
        // Resolve delegation chains: for each voter, follow the chain
        // until we reach a voter who actually cast a vote (or the
        // original voter if they voted themselves).
        let mut effective_votes: HashMap<String, &Vote> = HashMap::new();

        for (voter_id, vote) in &proposal.votes {
            effective_votes.insert(voter_id.clone(), vote);
        }

        // For voters who delegated, resolve to the final voter who
        // actually cast a vote.
        let mut resolved: HashMap<String, String> = HashMap::new();
        for vote in proposal.votes.values() {
            let voter = &vote.voter;
            if !self.delegations.contains_key(voter) {
                resolved.insert(voter.clone(), voter.clone());
            }
        }

        // Resolve delegations for voters who are in the delegations map.
        for from in self.delegations.keys() {
            let final_voter = Self::resolve_delegation(from, &self.delegations, &resolved);
            resolved.insert(from.clone(), final_voter);
        }

        // Count effective votes (one per resolved chain).
        let mut counts: HashMap<String, u64> = HashMap::new();
        let mut deciding: Vec<String> = Vec::new();
        let mut counted_voters: Vec<String> = Vec::new();

        for vote in proposal.votes.values() {
            // This voter's vote may count for multiple delegators.
            let voter = &vote.voter;

            // Count 1 for the voter themselves.
            *counts.entry(vote.choice.clone()).or_insert(0) += 1;
            deciding.push(voter.clone());

            // Count 1 for each delegator that resolves to this voter.
            for (from, to) in &resolved {
                if to == voter && from != voter {
                    *counts.entry(vote.choice.clone()).or_insert(0) += 1;
                    deciding.push(from.clone());
                }
            }

            counted_voters.push(voter.clone());
        }

        // Also count delegators who didn't vote but whose delegate did.
        let total_effective: u64 = counts.values().sum();
        let threshold = (total_effective as f64) / 2.0;
        let (winning, consensus) = find_winner_above_threshold(&counts, threshold);

        let participation_rate = if !self.delegations.is_empty() || !proposal.votes.is_empty() {
            // Approximate: ratio of voters who participated (directly or
            // via delegation) out of all known agents.
            let direct = proposal.votes.len() as f64;
            let delegated: usize = self
                .delegations
                .keys()
                .filter(|k| !proposal.votes.contains_key(*k))
                .count();
            (direct + delegated as f64) / (direct + delegated as f64 + 1.0) // avoid div-by-zero
        } else {
            0.0
        };

        ConsensusResult {
            winning_option: winning,
            tally: counts,
            participation_rate,
            consensus_reached: consensus,
            deciding_voters: deciding,
        }
    }

    /// Follow the delegation chain until we find a voter who cast a
    /// vote (or the chain ends).
    fn resolve_delegation(
        start: &str,
        delegations: &HashMap<String, String>,
        _resolved: &HashMap<String, String>,
    ) -> String {
        let mut current = start.to_string();
        let mut visited = vec![start.to_string()];

        while let Some(next) = delegations.get(&current) {
            if visited.contains(next) {
                // Cycle – break it by returning the current voter.
                return current;
            }
            visited.push(next.clone());
            current = next.clone();
        }

        current
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn real_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// Generate a simple pseudo-unique identifier.
fn simple_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    // Combine a timestamp with the counter for more uniqueness.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    format!("{:x}-{:x}", ts, n)
}

/// Find the option with the highest count that exceeds the threshold
/// (fraction-based).
fn find_winner_above_threshold(
    counts: &HashMap<String, u64>,
    threshold: f64,
) -> (Option<String>, bool) {
    let mut best_opt: Option<String> = None;
    let mut best_count: u64 = 0;

    for (opt, &count) in counts {
        if count > best_count {
            best_count = count;
            best_opt = Some(opt.clone());
        }
    }

    let consensus = (best_count as f64) > threshold;
    (best_opt, consensus)
}

/// Same as above but works with fixed-point (×1000) values.
fn find_winner_above_threshold_fp(
    counts: &HashMap<String, u64>,
    threshold_fp: u64,
) -> (Option<String>, bool) {
    let mut best_opt: Option<String> = None;
    let mut best_count: u64 = 0;

    for (opt, &count) in counts {
        if count > best_count {
            best_count = count;
            best_opt = Some(opt.clone());
        }
    }

    let consensus = best_count > threshold_fp;
    (best_opt, consensus)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create an engine with a controllable clock.
    fn engine_at(ms: u64) -> ConsensusEngine {
        let ms_cell = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(ms));
        let ms_clone = ms_cell.clone();
        ConsensusEngine::new_with_clock(Box::new(move || {
            ms_clone.load(std::sync::atomic::Ordering::Relaxed)
        }))
    }

    // -- SimpleMajority -----------------------------------------------------

    #[test]
    fn test_simple_majority_consensus() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Language",
            "Which language to use?",
            vec!["Rust".into(), "Go".into(), "Python".into()],
            60_000,
            ConsensusStrategy::SimpleMajority,
        );

        engine.cast_vote(&p.id, "a1", "Rust", 0.9, None).unwrap();
        engine.cast_vote(&p.id, "a2", "Rust", 0.8, None).unwrap();
        engine.cast_vote(&p.id, "a3", "Go", 0.7, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("Rust"));
        assert_eq!(result.tally.get("Rust"), Some(&2));
        assert_eq!(result.tally.get("Go"), Some(&1));
    }

    // -- SuperMajority ------------------------------------------------------

    #[test]
    fn test_super_majority_consensus() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Deploy",
            "Should we deploy?",
            vec!["Yes".into(), "No".into()],
            60_000,
            ConsensusStrategy::SuperMajority,
        );

        // 2 out of 3 → not > 2/3
        engine.cast_vote(&p.id, "a1", "Yes", 0.9, None).unwrap();
        engine.cast_vote(&p.id, "a2", "Yes", 0.8, None).unwrap();
        engine.cast_vote(&p.id, "a3", "No", 0.7, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        assert!(!result.consensus_reached, "2/3 is not > 2/3");

        // Add a 4th voter → 3/4 > 2/3
        engine.cast_vote(&p.id, "a4", "Yes", 0.9, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        assert!(result.consensus_reached, "3/4 > 2/3");
        assert_eq!(result.winning_option.as_deref(), Some("Yes"));
    }

    // -- Unanimous ----------------------------------------------------------

    #[test]
    fn test_unanimous_consensus() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Approve",
            "Unanimous approval needed",
            vec!["Approve".into(), "Reject".into()],
            60_000,
            ConsensusStrategy::Unanimous,
        );

        engine.cast_vote(&p.id, "a1", "Approve", 1.0, None).unwrap();
        engine.cast_vote(&p.id, "a2", "Approve", 0.9, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("Approve"));

        // Add dissenting vote
        engine.cast_vote(&p.id, "a3", "Reject", 0.6, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        assert!(!result.consensus_reached, "not unanimous any more");
    }

    // -- WeightedVoting -----------------------------------------------------

    #[test]
    fn test_weighted_voting() {
        let mut engine = engine_at(0);
        engine.voter_weights.insert("heavy".into(), 3.0);
        engine.voter_weights.insert("light".into(), 1.0);
        engine.voter_weights.insert("medium".into(), 2.0);

        let p = engine.create_proposal(
            "agent_a",
            "Budget",
            "Budget allocation",
            vec!["A".into(), "B".into()],
            60_000,
            ConsensusStrategy::WeightedVoting,
        );

        engine.cast_vote(&p.id, "heavy", "A", 0.9, None).unwrap();
        engine.cast_vote(&p.id, "light", "B", 0.8, None).unwrap();
        engine.cast_vote(&p.id, "medium", "B", 0.7, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        // A has weight 3, B has weight 1+2=3 → tie, no strict majority
        // But let's check: weighted A = 3000, B = 3000, total = 6000,
        // threshold_fp = 3000.  Neither > 3000.
        assert!(!result.consensus_reached, "tie → no majority");

        // Add another light voter for A.
        engine.voter_weights.insert("light2".into(), 1.0);
        engine.cast_vote(&p.id, "light2", "A", 0.9, None).unwrap();

        let result = engine.tally(&p.id).unwrap();
        // A: 3 + 1 = 4, B: 1 + 2 = 3, total = 7, threshold = 3.5
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("A"));
    }

    // -- Delegation ---------------------------------------------------------

    #[test]
    fn test_delegation() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Framework",
            "Which framework?",
            vec!["React".into(), "Vue".into()],
            60_000,
            ConsensusStrategy::Delegation,
        );

        // a1 votes React, a2 delegates to a1, a3 votes Vue
        engine.cast_vote(&p.id, "a1", "React", 0.9, None).unwrap();
        engine.cast_vote(&p.id, "a3", "Vue", 0.8, None).unwrap();
        engine.delegate_vote(&p.id, "a2", "a1").unwrap();

        let result = engine.tally(&p.id).unwrap();
        // React: a1 (1) + a2 delegation (1) = 2, Vue: a3 (1) = 1
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("React"));
    }

    // -- Expired proposals --------------------------------------------------

    #[test]
    fn test_expired_proposal() {
        let mut engine = engine_at(1000);
        let p = engine.create_proposal(
            "agent_a",
            "Lunch",
            "Where to eat?",
            vec!["Pizza".into(), "Sushi".into()],
            5000, // 5 seconds
            ConsensusStrategy::SimpleMajority,
        );

        engine.cast_vote(&p.id, "a1", "Pizza", 0.9, None).unwrap();

        // Still open
        assert_eq!(
            engine.get_proposal(&p.id).unwrap().status,
            ProposalStatus::Open
        );

        // Advance clock past deadline
        engine.now_fn = Box::new(|| 7000);
        let expired = engine.expire_proposals();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].status, ProposalStatus::Expired);

        // Voting should now be rejected
        let res = engine.cast_vote(&p.id, "a2", "Sushi", 0.8, None);
        assert!(res.is_err());
    }

    // -- Double vote rejected -----------------------------------------------

    #[test]
    fn test_double_vote_rejected() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Color",
            "Favourite colour?",
            vec!["Red".into(), "Blue".into()],
            60_000,
            ConsensusStrategy::SimpleMajority,
        );

        engine.cast_vote(&p.id, "a1", "Red", 0.9, None).unwrap();

        let res = engine.cast_vote(&p.id, "a1", "Blue", 0.8, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("already voted"));
    }

    // -- Vote after deadline rejected ---------------------------------------

    #[test]
    fn test_vote_after_deadline_rejected() {
        let mut engine = engine_at(1000);
        let p = engine.create_proposal(
            "agent_a",
            "Deploy",
            "Deploy now?",
            vec!["Yes".into(), "No".into()],
            3000, // 3 s
            ConsensusStrategy::SimpleMajority,
        );

        // Advance clock past deadline
        engine.now_fn = Box::new(|| 5000);

        let res = engine.cast_vote(&p.id, "a1", "Yes", 0.9, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("expired"));
    }

    // -- Additional edge-case tests -----------------------------------------

    #[test]
    fn test_close_proposal() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Close",
            "Force close test",
            vec!["A".into(), "B".into()],
            60_000,
            ConsensusStrategy::SimpleMajority,
        );

        engine.cast_vote(&p.id, "v1", "A", 1.0, None).unwrap();
        engine.cast_vote(&p.id, "v2", "A", 0.9, None).unwrap();
        engine.cast_vote(&p.id, "v3", "B", 0.5, None).unwrap();

        let result = engine.close_proposal(&p.id).unwrap();
        assert_eq!(
            engine.get_proposal(&p.id).unwrap().status,
            ProposalStatus::Closed
        );
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("A"));
    }

    #[test]
    fn test_ranked_choice() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "Dinner",
            "Dinner choice",
            vec!["Pizza".into(), "Sushi".into(), "Tacos".into()],
            60_000,
            ConsensusStrategy::RankedChoice,
        );

        // 3 voters: 2 prefer Pizza > Tacos > Sushi, 1 prefers Sushi > Tacos > Pizza
        engine
            .cast_ranked_vote(
                &p.id,
                "v1",
                vec!["Pizza".into(), "Tacos".into(), "Sushi".into()],
                0.9,
                None,
            )
            .unwrap();
        engine
            .cast_ranked_vote(
                &p.id,
                "v2",
                vec!["Pizza".into(), "Tacos".into(), "Sushi".into()],
                0.8,
                None,
            )
            .unwrap();
        engine
            .cast_ranked_vote(
                &p.id,
                "v3",
                vec!["Sushi".into(), "Tacos".into(), "Pizza".into()],
                0.7,
                None,
            )
            .unwrap();

        let result = engine.tally(&p.id).unwrap();
        // Pizza has 2 first-preference votes out of 3 → majority
        assert!(result.consensus_reached);
        assert_eq!(result.winning_option.as_deref(), Some("Pizza"));
    }

    #[test]
    fn test_ranked_choice_runoff() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "agent_a",
            "IRV",
            "Instant-runoff test",
            vec!["A".into(), "B".into(), "C".into()],
            60_000,
            ConsensusStrategy::RankedChoice,
        );

        // v1: A > B > C
        // v2: B > C > A
        // v3: B > C > A
        // First round: A=1, B=2, C=0 → no majority (need >1.5).
        // Eliminate C (0 votes). A still has 1, B still has 2.
        // Second round: B=2 out of 3 → majority → B wins.
        engine
            .cast_ranked_vote(
                &p.id,
                "v1",
                vec!["A".into(), "B".into(), "C".into()],
                0.9,
                None,
            )
            .unwrap();
        engine
            .cast_ranked_vote(
                &p.id,
                "v2",
                vec!["B".into(), "C".into(), "A".into()],
                0.8,
                None,
            )
            .unwrap();
        engine
            .cast_ranked_vote(
                &p.id,
                "v3",
                vec!["B".into(), "C".into(), "A".into()],
                0.7,
                None,
            )
            .unwrap();
        let result = engine.tally(&p.id).unwrap();
        assert!(result.consensus_reached);
        // B has 2 first-preference votes out of 3 (>50%), so B wins.
        assert_eq!(result.winning_option.as_deref(), Some("B"));
    }

    #[test]
    fn test_list_open() {
        let mut engine = engine_at(0);
        let p1 = engine.create_proposal(
            "a",
            "t1",
            "d1",
            vec!["X".into()],
            1000,
            ConsensusStrategy::SimpleMajority,
        );
        let p2 = engine.create_proposal(
            "a",
            "t2",
            "d2",
            vec!["Y".into()],
            2000,
            ConsensusStrategy::SimpleMajority,
        );

        assert_eq!(engine.list_open().len(), 2);
        engine.close_proposal(&p1.id).unwrap();
        assert_eq!(engine.list_open().len(), 1);
        // p2 should still be open
        assert!(engine.list_open().iter().any(|p| p.id == p2.id));
    }

    #[test]
    fn test_delegation_circular_rejected() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "a",
            "Del",
            "Delegation test",
            vec!["A".into(), "B".into()],
            60_000,
            ConsensusStrategy::Delegation,
        );

        engine.delegate_vote(&p.id, "v1", "v2").unwrap();
        engine.delegate_vote(&p.id, "v2", "v3").unwrap();
        engine.delegate_vote(&p.id, "v3", "v1").unwrap_err();
    }

    #[test]
    fn test_delegation_non_delegation_strategy_rejected() {
        let mut engine = engine_at(0);
        let p = engine.create_proposal(
            "a",
            "Del",
            "Wrong strategy",
            vec!["A".into(), "B".into()],
            60_000,
            ConsensusStrategy::SimpleMajority,
        );

        let res = engine.delegate_vote(&p.id, "v1", "v2");
        assert!(res.is_err());
    }

    #[test]
    fn test_empty_proposal_tally() {
        let engine = engine_at(0);
        // No proposals – should error
        let res = engine.tally("nonexistent");
        assert!(res.is_err());
    }
}
