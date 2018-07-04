use server::peer::Peer;

pub struct PeerManager {
    pub peers: Vec<Peer>
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
}