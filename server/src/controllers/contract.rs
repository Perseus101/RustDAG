use rocket::{Route, State};
use rocket_contrib::json::Json;

use rustdag_lib::dag::contract::Contract;

use dagmanager::DAGManager;

pub fn contract_routes() -> Vec<Route> {
    routes![get_contract]
}

#[get("/<hash>")]
fn get_contract(hash: u64, dag: State<DAGManager>) -> Option<Json<Contract>> {
    dag.inner().get_contract(hash).and_then(|x| Some(Json(x)))
}
