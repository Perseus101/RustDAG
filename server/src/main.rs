#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket::State;
use rocket_contrib::json::Json;

extern crate rustdag_lib;

use rustdag_lib::util::{self, peer::Peer, types::{TransactionHashes,TransactionStatus}};
use rustdag_lib::dag::{self, transaction::Transaction};

mod dagmanager;
mod peermanager;

use dagmanager::DAGManager;

#[get("/tips")]
fn get_tips(dag: State<DAGManager>) -> Json<TransactionHashes> {
    Json(dag.inner().get_tips())
}

#[get("/transaction/get/<hash>")]
fn get_transaction(hash: u64, dag: State<DAGManager>) -> Option<Json<Transaction>> {
    dag.inner().get_transaction(hash).and_then(|x| Some(Json(x)))
}

#[get("/transaction/get/<hash>/status")]
fn get_transaction_status(hash: u64, dag: State<DAGManager>) -> Json<TransactionStatus> {
    Json(dag.inner().get_transaction_status(hash))
}

#[post("/transaction", data = "<transaction>")]
fn add_transaction(transaction: Json<Transaction>, dag: State<DAGManager>) -> Json<TransactionStatus> {
    Json(dag.inner().add_transaction(transaction.into_inner()))
}

#[post("/peer/register", data = "<peer>")]
fn new_peer(peer: Json<Peer>, chain: State<DAGManager>) {
    chain.inner().add_peer(peer.into_inner());
}

fn main() {
    rocket::ignite()
        .mount("/", routes![get_tips, get_transaction, get_transaction_status, add_transaction, new_peer])
        .manage(DAGManager::default())
        .launch();
}
