#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use rocket::State;
use rocket_contrib::Json;

extern crate rustdag_lib;

use rustdag_lib::util::{self, peer::Peer, types::{TransactionHashes,ProcessStatus}};
use rustdag_lib::dag::{self, transaction::Transaction};

mod dagmanager;
mod peermanager;

use dagmanager::DAGManager;

#[get("/tips/all")]
fn get_tips(dag: State<DAGManager>) -> Json<Vec<Transaction>> {
    Json(dag.inner().get_tips())
}

#[get("/tips")]
fn select_tips(dag: State<DAGManager>) -> Json<TransactionHashes> {
    Json(dag.inner().select_tips())
}

#[get("/transaction/get/<hash>")]
fn get_transaction(hash: u64, dag: State<DAGManager>) -> Option<Json<Transaction>> {
    dag.inner().get_transaction(hash).and_then(|x| Some(Json(x)))
}

#[post("/transaction", data = "<transaction>")]
fn add_transaction(transaction: Json<Transaction>, dag: State<DAGManager>) -> Json<ProcessStatus> {
    Json(ProcessStatus::new(dag.inner().add_transaction(transaction.into_inner())))
}

#[post("/peer/register", data = "<peer>")]
fn new_peer(peer: Json<Peer>, chain: State<DAGManager>) {
    chain.inner().add_peer(peer.into_inner());
}

fn main() {
    rocket::ignite()
        .mount("/", routes![select_tips, get_tips, get_transaction, add_transaction, new_peer])
        .manage(DAGManager::default())
        .launch();
}
