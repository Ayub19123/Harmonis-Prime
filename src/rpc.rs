use crate::errors::SovereignResult;
use crate::types::{
    AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse,
};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct RaftRpc {
    node_id: u64,
    peers: HashMap<u64, String>,
}

impl RaftRpc {
    pub fn new(node_id: u64, peers: HashMap<u64, String>) -> Self {
        Self { node_id, peers }
    }

    pub fn send_append_entries(
        &self,
        target: u64,
        req: &AppendEntriesRequest,
    ) -> SovereignResult<AppendEntriesResponse> {
        let _url = format!("{}/raft/append_entries", self.get_peer_url(target));
        let _ = req;
        Ok(AppendEntriesResponse {
            term: req.term,
            success: true,
            conflict_index: 0,
            conflict_term: 0,
        })
    }

    pub fn send_request_vote(
        &self,
        target: u64,
        req: &RequestVoteRequest,
    ) -> SovereignResult<RequestVoteResponse> {
        let _url = format!("{}/raft/request_vote", self.get_peer_url(target));
        let _ = req;
        Ok(RequestVoteResponse {
            term: req.term,
            vote_granted: true,
        })
    }

    fn get_peer_url(&self, target: u64) -> String {
        self.peers
            .get(&target)
            .cloned()
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", 9000 + target))
    }
}
