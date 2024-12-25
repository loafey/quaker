use faststr::FastStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PickupType {
    Weapon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PickupData {
    pub pickup_type: PickupType,
    pub classname: FastStr,
    pub gives: FastStr,
    pub pickup_model: FastStr,
    pub pickup_material: FastStr,
    pub texture_file: FastStr,
    pub scale: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum SoundEffect {
    #[default]
    Silent,
    Single(FastStr),
    Random(Vec<FastStr>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponData {
    #[serde(default)]
    pub shoot_sfx: SoundEffect,
    pub id: FastStr,
    #[serde(default)]
    pub slot: usize,
    #[serde(default)]
    pub texture_file: FastStr,
    #[serde(default)]
    pub model_file: FastStr,
    pub scale: f32,
    #[serde(default)]
    pub animations: WeaponAnimations,
    #[serde(default)]
    pub offset: [f32; 3],
    #[serde(default)]
    pub rotation: [f32; 3],
    pub pickup_sound: Option<FastStr>,
    #[serde(default)]
    pub attack1: Attack,
    #[serde(default)]
    pub attack2: Attack,
    #[serde(default = "default_pickupmessage1")]
    pub pickup_message1: FastStr,
    #[serde(default = "default_pickupmessage2")]
    pub pickup_message2: FastStr,
    #[serde(default = "default_fancyname")]
    pub fancy_name: FastStr,
}
impl WeaponData {
    fn default_firetime() -> f32 {
        1.0
    }
}

fn default_fancyname() -> FastStr {
    FastStr::from("UNNAMNED_WEAPON")
}

fn default_pickupmessage1() -> FastStr {
    FastStr::from("PICKED UP: ")
}

fn default_pickupmessage2() -> FastStr {
    FastStr::from("!")
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
    pub idle: usize,
    pub shoot1: usize,
    pub shoot2: usize,
    pub reload: Option<usize>,

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
