//! Gen 8 integer damage arithmetic. Modifiers use a denominator of 4096.
//! Reference: Pokemon Showdown 942cda71, sim/battle-actions.ts and sim/battle.ts.

fn fixed(factor: f32) -> u64 {
    (f64::from(factor) * 4096.0).round().max(0.0) as u64
}

/// Modifier chains round halves up; applying the chain to a value rounds halves down.
pub(crate) fn chain(a: f32, b: f32) -> f32 {
    ((fixed(a) * fixed(b) + 2048) / 4096) as f32 / 4096.0
}

pub(crate) fn chain_all(factors: impl IntoIterator<Item = f32>) -> f32 {
    factors.into_iter().fold(1.0, chain)
}

pub(crate) fn modify(value: u32, factor: f32) -> u32 {
    ((u64::from(value) * fixed(factor) + 2047) / 4096) as u32
}

pub(crate) fn base(level: u8, power: u32, attack: u32, defense: u32) -> u32 {
    (2 * u32::from(level) / 5 + 2)
        .wrapping_mul(power)
        .wrapping_mul(attack.max(1))
        / defense.max(1)
        / 50
        + 2
}

pub(crate) fn randomize(value: u32, roll: f32) -> u32 {
    // The public replay hook remains a multiplier; normalize it to a Gen 8 percentage.
    let percent = (roll * 100.0).round().clamp(85.0, 100.0) as u32;
    value.wrapping_mul(percent) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_application_and_chaining_have_different_half_rounding() {
        assert_eq!(modify(3, 1.5), 4);
        assert_eq!(modify(5, 1.5), 7);
        assert_eq!(chain(4097.0 / 4096.0, 0.5), 2049.0 / 4096.0);
        assert_eq!(
            modify(100, chain_all([5325.0 / 4096.0, 4915.0 / 4096.0])),
            156
        );
    }

    #[test]
    fn base_damage_floors_level_and_divisions_and_clamps_zero_stats() {
        assert_eq!(base(53, 73, 103, 97), 37);
        assert_eq!(base(1, 1, 0, 0), 2);
    }
}
