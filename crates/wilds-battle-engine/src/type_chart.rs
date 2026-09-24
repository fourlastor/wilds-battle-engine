use crate::model::{PokemonType, Status};
use PokemonType::*;

// Mirrors data/PType.cs. Resistance takes precedence over weakness there,
// including Grass versus Ice, which appears in both lists.
pub fn multiplier(attack: PokemonType, defenders: &[PokemonType]) -> f32 {
    defenders.iter().fold(1.0, |value, defender| {
        let (weak, resist, immune): (&[PokemonType], &[PokemonType], &[PokemonType]) =
            match defender {
                Normal => (&[Fighting], &[], &[Ghost]),
                Fighting => (&[Flying, Psychic, Fairy], &[Bug, Rock, Dark], &[]),
                Flying => (&[Rock, Electric, Ice], &[Grass, Fighting, Bug], &[Ground]),
                Poison => (
                    &[Ground, Psychic],
                    &[Grass, Fighting, Poison, Bug, Fairy],
                    &[],
                ),
                Ground => (&[Water, Grass, Ice], &[Poison, Rock], &[Electric]),
                Rock => (
                    &[Water, Grass, Fighting, Ground, Steel],
                    &[Normal, Fire, Poison, Flying],
                    &[],
                ),
                Bug => (&[Fire, Flying, Rock], &[Grass, Fighting, Ground], &[]),
                Ghost => (&[Ghost, Dark], &[Poison, Bug], &[Normal, Fighting]),
                Steel => (
                    &[Fire, Fighting, Ground],
                    &[
                        Normal, Grass, Ice, Flying, Psychic, Bug, Rock, Dragon, Steel, Fairy,
                    ],
                    &[Poison],
                ),
                Fire => (
                    &[Water, Ground, Rock],
                    &[Fire, Grass, Ice, Bug, Steel, Fairy],
                    &[],
                ),
                Water => (&[Grass, Electric], &[Fire, Water, Ice, Steel], &[]),
                Grass => (
                    &[Fire, Ice, Poison, Flying, Bug],
                    &[Water, Ice, Electric, Ground],
                    &[],
                ),
                Electric => (&[Ground], &[Electric, Flying, Steel], &[]),
                Psychic => (&[Bug, Ghost, Dark], &[Fighting, Psychic], &[]),
                Ice => (&[Fire, Fighting, Rock, Steel], &[Ice], &[]),
                Dragon => (&[Ice, Dragon, Fairy], &[Fire, Water, Grass, Electric], &[]),
                Dark => (&[Fighting, Bug, Fairy], &[Ghost, Dark], &[Psychic]),
                Fairy => (&[Poison, Steel], &[Fighting, Bug, Dark], &[Dragon]),
                None => (&[], &[], &[]),
            };
        if immune.contains(&attack) {
            0.0
        } else if resist.contains(&attack) {
            value * 0.5
        } else if weak.contains(&attack) {
            value * 2.0
        } else {
            value
        }
    })
}

pub fn status_immune(status: Status, move_type: PokemonType, defenders: &[PokemonType]) -> bool {
    defenders.iter().any(|kind| match status {
        Status::Poisoned | Status::BadlyPoisoned => matches!(kind, Poison | Steel),
        Status::Burned => *kind == Fire,
        Status::Frozen => *kind == Ice,
        Status::Paralyzed => *kind == Electric || (*kind == Ground && move_type == Electric),
        Status::Asleep => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csharp_resistance_precedes_weakness() {
        assert_eq!(multiplier(Ice, &[Grass]), 0.5);
    }
}
