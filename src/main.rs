#[macro_use] extern crate serde_derive;

mod util;
mod security;

mod dag;
use dag::blockdag::BlockDAG;

fn main() {
    let mut dag = BlockDAG::default();
    let trunk_hash: String;
    let branch_hash: String;
    {
        let tips = dag.get_tips();
        trunk_hash = tips[0].get_hash();
        branch_hash = tips[1].get_hash();
    }
    dag.create_transaction(trunk_hash.clone(), branch_hash.clone());
    let tips = dag.get_tips();
    println!("{:?}", tips);
}
