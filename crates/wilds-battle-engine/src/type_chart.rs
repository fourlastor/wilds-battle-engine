use crate::model::{PokemonType, Status};
use PokemonType::*;

// Generation 8 type effectiveness; a defending type has disjoint weakness,
// resistance and immunity lists. Multiple defending types multiply together.
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
                    &[Water, Grass, Electric, Ground],
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
    fn all_gen8_matchups_match_the_reference_chart() {
        // Independent fixture extracted from PokeAPI data/v2/csv/type_efficacy.csv.
        let types = [
            Normal, Fighting, Flying, Poison, Ground, Rock, Bug, Ghost, Steel, Fire, Water, Grass,
            Electric, Psychic, Ice, Dragon, Dark, Fairy,
        ];
        let rows: Vec<_> = include_str!("../tests/fixtures/gen8-type-chart.csv")
            .lines()
            .collect();
        assert_eq!(rows.len(), 19);
        for (attack, row) in types.into_iter().zip(&rows[1..]) {
            let fields: Vec<_> = row.split(',').collect();
            assert_eq!(fields.len(), 19);
            assert_eq!(fields[0], format!("{attack:?}").to_lowercase());
            for (defender, expected) in types.into_iter().zip(&fields[1..]) {
                let expected = expected.parse::<f32>().unwrap() / 100.0;
                assert_eq!(
                    multiplier(attack, &[defender]),
                    expected,
                    "{attack:?} -> {defender:?}"
                );
            }
        }
    }

    #[test]
    fn dual_types_multiply_and_immunities_win_in_either_order() {
        assert_eq!(multiplier(Ice, &[Grass, Dragon]), 4.0);
        assert_eq!(multiplier(Grass, &[Grass, Dragon]), 0.25);
        assert_eq!(multiplier(Ice, &[Grass, Fire]), 1.0);
        for pair in [[Water, Ground], [Ground, Water]] {
            assert_eq!(multiplier(Electric, &pair), 0.0);
        }
        assert_eq!(multiplier(None, &[Grass]), 1.0);
        assert_eq!(multiplier(Ice, &[None]), 1.0);
    }
}
