use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;

use std::error::Error;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum MapError {
    NotFound,
    LookupError,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapError::NotFound => write!(f, "No value for key"),
            MapError::LookupError => write!(f, "Error while looking up value"),
        }
    }
}

impl Error for MapError {}

pub type MapResult<T> = Result<T, MapError>;

pub trait Map<K: Eq + Hash, V> {
    fn get(&self, k: &K) -> MapResult<&V>;
    fn set(&mut self, k: K, v: V) -> MapResult<()>;
}

impl<K: Eq + Hash, V> Map<K, V> for HashMap<K, V> {
    fn get(&self, k: &K) -> MapResult<&V> {
        HashMap::get(self, k).map_or(Err(MapError::NotFound), |v| { Ok(v) })
    }
    fn set(&mut self, k: K, v: V) -> MapResult<()> {
        HashMap::insert(self, k, v);
        Ok(())
    }
}