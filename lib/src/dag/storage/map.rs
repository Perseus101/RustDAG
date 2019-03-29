use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
use std::ops::Deref;

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
    fn get<'a>(&'a self, k: &K) -> MapResult<OOB<'a, V>>;
    fn set(&mut self, k: K, v: V) -> MapResult<()>;
}

impl<K: Eq + Hash, V> Map<K, V> for HashMap<K, V> {
    fn get<'a>(&'a self, k: &K) -> MapResult<OOB<'a, V>> {
        HashMap::get(self, k).map_or(Err(MapError::NotFound), |v| Ok(OOB::Borrowed(v)))
    }
    fn set(&mut self, k: K, v: V) -> MapResult<()> {
        HashMap::insert(self, k, v);
        Ok(())
    }
}

#[derive(PartialEq, Hash, Debug)]
pub enum OOB<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> OOB<'a, T> {
    pub fn inner_ref(&'a self) -> &'a T {
        match self {
            OOB::Owned(t) => &t,
            OOB::Borrowed(ref t) => t,
        }
    }
}

impl<'a, T> Borrow<T> for OOB<'a, T> {
    fn borrow(&self) -> &T {
        self.inner_ref()
    }
}

impl<'a, T> Deref for OOB<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner_ref()
    }
}
