#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket::State;
use rocket_contrib::json::Json;

extern crate rustdag_lib;

use rustdag_lib::util::{self, peer::Peer, types::TransactionHashes};
use rustdag_lib::dag;

mod dagmanager;
mod peermanager;
mod controllers;

use dagmanager::DAGManager;

#[get("/tips")]
fn get_tips(dag: State<DAGManager>) -> Json<TransactionHashes> {
    Json(dag.inner().get_tips())
}

#[post("/peer/register", data = "<peer>")]
fn new_peer(peer: Json<Peer>, chain: State<DAGManager>) {
    chain.inner().add_peer(peer.into_inner());
}

fn main() {
    rocket::ignite()
        .mount("/", routes![get_tips, new_peer])
        .mount("/transaction", controllers::transaction::transaction_routes())
        .mount("/contract", controllers::contract::contract_routes())
        .mount("/node", controllers::node::node_routes())
        .manage(DAGManager::default())
        .launch();
}
