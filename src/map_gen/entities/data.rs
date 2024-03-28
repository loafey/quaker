use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PickupType {
    Weapon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickupData {
    pub pickup_type: PickupType,
    pub classname: String,
    pub gives: String,
    pub pickup_model: String,
    pub pickup_material: String,
    pub texture_file: String,
    pub scale: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum SoundEffect {
    #[default]
    Silent,
    Single(String),
    Random(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponData {
    #[serde(default)]
    pub shoot_sfx: SoundEffect,
    pub id: String,
    #[serde(default)]
    pub slot: usize,
    #[serde(default)]
    pub texture_file: String,
    #[serde(default)]
    pub model_file: String,
    pub scale: f32,
    #[serde(default)]
    pub animations: WeaponAnimations,
    #[serde(default)]
    pub offset: [f32; 3],
    #[serde(default)]
    pub rotation: [f32; 3],
    pub pickup_sound: Option<String>,
    #[serde(default)]
    pub attack1: Attack,
    #[serde(default)]
    pub attack2: Attack,
    #[serde(default = "default_pickupmessage1")]
    pub pickup_message1: String,
    #[serde(default = "default_pickupmessage2")]
    pub pickup_message2: String,
    #[serde(default = "default_fancyname")]
    pub fancy_name: String,
}
impl WeaponData {
    fn default_firetime() -> f32 {
        1.0
    }
}

fn default_fancyname() -> String {
    "UNNAMNED_WEAPON".to_string()
}

fn default_pickupmessage1() -> String {
    "pICKED UP: ".to_string()
}

fn default_pickupmessage2() -> String {
    "!".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(tag = "type")]
pub enum Attack {
    #[default]
    None,
    RayCast {
        amount: usize,
        angle_mod: f32,
        damage: f32,
        damage_mod: f32,
        range: f32,
    },
    Projectile {
        projectile: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WeaponAnimations {
    pub idle: String,
    pub shoot1: String,
    pub shoot2: String,
    pub reload: Option<String>,

    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time1: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time1: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub fire_time2: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub anim_time2: f32,

    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time_skip: f32,
    #[serde(default = "WeaponData::default_firetime")]
    pub reload_time: f32,
}
