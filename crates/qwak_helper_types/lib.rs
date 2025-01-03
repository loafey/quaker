use extism_pdk::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, FromBytes, ToBytes, Deserialize, Serialize)]
#[encoding(Msgpack)]
pub struct MapInteraction(pub String, pub u64);
