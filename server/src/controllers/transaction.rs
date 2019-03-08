use rocket::{Route, State};
use rocket_contrib::json::Json;

use rustdag_lib::util::{HexEncodedTransaction, types::TransactionStatus};
use rustdag_lib::dag::transaction::Transaction;

use dagmanager::DAGManager;

pub fn transaction_routes() -> Vec<Route> {
    routes![get_transaction, get_transaction_status, get_transaction_hex,
            post_transaction, post_hex_transaction]
}

#[get("/<hash>")]
fn get_transaction(hash: u64, dag: State<DAGManager>) -> Option<Json<Transaction>> {
    dag.inner().get_transaction(hash).and_then(|x| Some(Json(x)))
}

#[get("/<hash>/status")]
fn get_transaction_status(hash: u64, dag: State<DAGManager>) -> Json<TransactionStatus> {
    Json(dag.inner().get_transaction_status(hash))
}

#[get("/<hash>/hex")]
fn get_transaction_hex(hash: u64, dag: State<DAGManager>) -> Option<Json<HexEncodedTransaction>> {
    dag.inner().get_transaction(hash).and_then(|x| Some(Json(x.into())))
}

#[post("/", data = "<transaction>")]
fn post_transaction(transaction: Json<Transaction>, dag: State<DAGManager>) -> Json<TransactionStatus> {
    Json(dag.inner().add_transaction(transaction.into_inner()))
}

#[post("/hex", data = "<transaction>")]
fn post_hex_transaction( transaction: Json<HexEncodedTransaction>, dag: State<DAGManager>) -> Json<TransactionStatus> {
    Json(dag.inner().add_transaction(transaction.into_inner().into()))
}
