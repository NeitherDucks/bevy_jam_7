use bevy::ecs::resource::Resource;
use bevy_prng::ChaCha20Rng;
use rand::seq::SliceRandom;

#[derive(Resource)]
pub struct Shuffle<T: Clone> {
    default: Vec<T>,
    remaining: Vec<T>,
}

impl<T: Clone> Shuffle<T> {
    pub fn new(list: &[T]) -> Self {
        debug_assert!(!list.is_empty(), "Levels must not be empty");

        Self {
            default: Vec::from(list),
            remaining: Vec::from(list),
        }
    }

    pub fn next(&mut self, rng: &mut ChaCha20Rng) -> T {
        if self.remaining.is_empty() {
            self.remaining = self.default.clone();
        }

        self.remaining.shuffle(rng);
        self.remaining.pop().unwrap()
    }
}
