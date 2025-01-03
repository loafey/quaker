use bevy::prelude::Component;
use faststr::FastStr;

#[derive(Debug, Component, Clone)]
pub struct Interactable {
    pub script: FastStr,
}
