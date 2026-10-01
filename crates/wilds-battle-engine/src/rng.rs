/// Random draws used by the battle simulation. Implementations can record or
/// replay draws to compare a battle with another engine. `fraction` returns a
/// value in `[0, 1)`, `range` includes both endpoints, and `index(count)`
/// returns a value below a nonzero `count`, and `damage_roll` returns a
/// multiplier in `[0.85, 1]`.
pub trait BattleRng {
    fn next_u64(&mut self) -> u64;
    fn fraction(&mut self) -> f32;

    fn range(&mut self, min: u8, max: u8) -> u8 {
        min + (self.next_u64() % (u64::from(max - min) + 1)) as u8
    }

    fn index(&mut self, count: usize) -> usize {
        (self.next_u64() as usize) % count
    }

    fn chance(&mut self, probability: f32) -> bool {
        self.fraction() <= probability
    }

    fn damage_roll(&mut self) -> f32 {
        0.85 + self.fraction() * 0.15
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SeededRng(u64);

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        })
    }
}

impl BattleRng for SeededRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn fraction(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }
}
