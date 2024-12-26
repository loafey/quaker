use bevy::ecs::system::Resource;
use std::{
    fmt::Debug,
    fs::File,
    io::{self, Read},
    marker::PhantomData,
    path::Path,
};

pub struct EGame;
pub struct EMisc;

#[derive(Resource)]
pub struct Entropy<Type> {
    values: Vec<f32>,
    index: usize,
    _phantom: PhantomData<Type>,
}

impl<Type> Debug for Entropy<Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entropy<{}>", std::any::type_name::<Type>())
    }
}
pub fn entropy_game() -> Entropy<EGame> {
    Entropy::load("assets/game.entropy").unwrap()
}
pub fn entropy_misc() -> Entropy<EMisc> {
    Entropy::load("assets/game.entropy").unwrap()
}
impl<Type> Entropy<Type> {
    /// Returns a "random" float between 0.0 and 1.0
    #[must_use]
    pub fn get_f32(&mut self) -> f32 {
        let value = self.values[self.index];
        self.index += 1;
        if self.index >= self.values.len() {
            self.index = 0;
        }
        value
    }
    /// Chooses a random element in a slice and copies it.
    /// Panics if slice is empty.
    #[must_use]
    #[allow(dead_code)]
    pub fn choose_copy<T: Copy>(&mut self, slice: &[T]) -> T {
        let entropy = self.get_f32();
        let len = slice.len() as f32;
        let index = (entropy * len).floor() as usize;
        slice[index]
    }
    /// Chooses a random element in a slice.
    /// Panics if slice is empty.
    #[must_use]
    #[allow(dead_code)]
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        let entropy = self.get_f32();
        let len = slice.len() as f32;
        let index = (entropy * len).floor() as usize;
        &slice[index]
    }

    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut f = File::open(path)?;
        let mut amount = [0u8; 4];
        f.read_exact(&mut amount)?;
        let amount = u32::from_ne_bytes(amount);
        let mut values = Vec::new();
        for _ in 0..amount {
            let mut byte = [0u8];
            f.read_exact(&mut byte)?;
            values.push(byte[0] as f32 / 255.0);
        }

        Ok(Entropy {
            values,
            index: 0,
            _phantom: PhantomData,
        })
    }
}
