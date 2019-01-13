pub mod source;
pub mod state;
pub mod error;

#[allow(clippy::module_inception)]
mod contract;
mod resolver;

pub use self::contract::{Contract, ContractValue};