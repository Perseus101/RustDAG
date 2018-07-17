use util::peer::Peer;

pub struct PeerManager {
    peers: Vec<Peer>
}

impl PeerManager {
    pub fn new() -> PeerManager {
        PeerManager {
            peers: Vec::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.push(peer);
    }

    pub fn map_peers<U, F>(&self, f: F) -> Vec<U>
        where F: Fn(&Peer) -> U {
        self.peers.iter().map(f).collect()
    }
}
