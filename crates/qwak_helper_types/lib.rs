use std::fmt::Debug;

use extism_pdk::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, FromBytes, ToBytes)]
#[encoding(Msgpack)]
pub struct MapInteraction(pub NetWorldPtr, pub String);

#[derive(Clone, Copy, Deserialize, Serialize, FromBytes, ToBytes)]
#[encoding(Msgpack)]
pub struct NetWorldPtr(u64);
impl Debug for NetWorldPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0:08x}", self.0)
    }
}
impl NetWorldPtr {
    pub fn new<T>(ptr: &T) -> Self {
        let ptr = unsafe { std::mem::transmute::<&T, usize>(ptr) };
        Self(ptr as u64)
    }
    pub fn into_inner<'l, T>(self) -> &'l T {
        unsafe { std::mem::transmute::<usize, &T>(self.0 as usize) }
    }
}
