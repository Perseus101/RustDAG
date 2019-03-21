use rocket::{Route, State};
use rocket_contrib::json::Json;

use rustdag_lib::dag::{contract::ContractValue, storage::mpt::node::Node};

use dagmanager::DAGManager;

pub fn node_routes() -> Vec<Route> {
    routes![get_mpt_node]
}

#[get("/<hash>")]
fn get_mpt_node(hash: u64, dag: State<DAGManager>) -> Option<Json<Node<ContractValue>>> {
    dag.inner().get_mpt_node(hash).and_then(|x| Some(Json(x)))
}
