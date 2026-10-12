//! Exact Gen 8 damage vectors, including values that distinguish rounding stages.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, Condition, MoveCatalog,
    Pokemon, PokemonType, Rule, Side, Stat, Status,
};

struct Roll(u8);
impl BattleRng for Roll {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        0.5
    }
    fn damage_roll(&mut self) -> f32 {
        f32::from(self.0) / 100.0
    }
}
fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|s| s.to_string()).collect());
    p.hp = 10000;
    p.max_hp = 10000;
    p.level = 53;
    p.attack = 103;
    p.sp_attack = 103;
    p.defense = 97;
    p.sp_defense = 97;
    p
}
fn hit(
    user: Pokemon,
    target: Pokemon,
    power: u16,
    crit: bool,
    typeless: bool,
    roll: u8,
    kind: PokemonType,
) -> u16 {
    let source = format!(
        r#"return {{
        {{id='test', name='Test', type=Type.{kind:?}, category=Category.Physical, pp=10,
          accuracy=Accuracy.Always, script=function(ctx) ctx:damage({power}, {{always_crit={crit},typeless={typeless}}}) end}},
        {{id='idle', name='Idle', type=Type.Normal, category=Category.Status, pp=40,
          target=Target.User, script=function(ctx) end}}
    }}"#
    );
    let mut b = Battle::with_rng(
        Roll(roll),
        [vec![user], vec![target]],
        MoveCatalog::from_lua(&source).unwrap(),
    )
    .unwrap();
    for _ in 0..4 {
        let result = b.advance().unwrap();
        for e in result.events {
            if let BattleEvent::Damage { target, amount, .. } = e
                && target.side == Side::Foes
            {
                return amount;
            }
        }
        let AdvanceStatus::Awaiting(p) = result.status else {
            panic!("unexpected end")
        };
        let c = p
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap();
        b.set_response(ActionSelection {
            prompt_id: p.id,
            choice_id: c.id(),
        })
        .unwrap();
    }
    panic!("no damage")
}
fn screen() -> Condition {
    Condition {
        name: "reflect".into(),
        value: 1,
        turns_left: None,
        group: None,
        end_message: None,
        rules: vec![Rule::DamageTaken {
            category: None,
            move_type: None,
            factor: 0.5,
            not_on_crit: true,
            multi_battle_factor: None,
            screen: true,
            unless_side_effect: None,
        }],
    }
}

#[test]
fn all_sixteen_damage_rolls_match_integer_reference_vectors() {
    // Level 53, 73 BP, 103 Atk, 97 Def: unmodified damage is 37.
    let expected = [
        31, 31, 32, 32, 32, 33, 33, 34, 34, 34, 35, 35, 35, 36, 36, 37,
    ];
    for (roll, expected) in (85..=100).zip(expected) {
        assert_eq!(
            hit(
                mon("User", &["test"]),
                mon("Foe", &["idle"]),
                73,
                false,
                false,
                roll,
                PokemonType::Ice
            ),
            expected,
            "roll {roll}"
        );
    }
}

#[test]
fn default_rng_produces_only_the_sixteen_gen8_percentages() {
    struct Fraction(f32);
    impl BattleRng for Fraction {
        fn next_u64(&mut self) -> u64 {
            panic!("not used")
        }
        fn fraction(&mut self) -> f32 {
            self.0
        }
    }
    for i in 0..16 {
        for offset in [0.0, 0.5, 0.999] {
            let mut rng = Fraction((i as f32 + offset) / 16.0);
            assert_eq!((rng.damage_roll() * 100.0).round() as u8, 85 + i);
        }
    }
}

#[test]
fn weather_crit_random_stab_type_burn_and_sniper_have_ordered_rounding() {
    let mut user = mon("User", &["test"]);
    user.types = vec![PokemonType::Fire];
    user.status = Some(Status::Burned);
    user.ability = Some("sniper".into());
    let mut target = mon("Foe", &["idle"]);
    target.types = vec![PokemonType::Grass];
    target.ability = Some("drought".into());
    target.conditions.push(screen()); // A critical ignores the screen.
    // 37 -> sun 55 -> critical 82 -> 85% 69 -> STAB 103 -> type 206
    // -> burn 103 -> Sniper 154 (half down), with no Reflect reduction.
    assert_eq!(
        hit(user, target, 73, true, false, 85, PokemonType::Fire),
        154
    );
}

#[test]
fn screens_and_resistance_berries_chain_after_burn() {
    let mut user = mon("User", &["test"]);
    user.types = vec![PokemonType::Fire];
    user.status = Some(Status::Burned);
    let mut target = mon("Foe", &["idle"]);
    target.types = vec![PokemonType::Grass];
    target.ability = Some("drought".into());
    target.item = Some("occa-berry".into());
    target.conditions.push(screen());
    // 37 -> 55 -> 46 -> 69 -> 138 -> 69 -> final 1/4 modifier -> 17.
    assert_eq!(
        hit(user, target, 73, false, false, 85, PokemonType::Fire),
        17
    );
}

#[test]
fn stab_rounds_half_down_and_resistance_floors_before_minimum_damage() {
    let mut user = mon("User", &["test"]);
    user.types = vec![PokemonType::Ice];
    assert_eq!(
        hit(
            user.clone(),
            mon("Foe", &["idle"]),
            73,
            false,
            false,
            100,
            PokemonType::Ice
        ),
        55
    );
    let mut target = mon("Foe", &["idle"]);
    target.types = vec![PokemonType::Water, PokemonType::Ice];
    // 37 -> STAB 55 -> double resistance 13.
    assert_eq!(
        hit(user, target, 73, false, false, 100, PokemonType::Ice),
        13
    );
    let mut user = mon("User", &["test"]);
    user.level = 1;
    user.attack = 1;
    let mut target = mon("Foe", &["idle"]);
    target.defense = 1000;
    target.types = vec![PokemonType::Water, PokemonType::Ice];
    target.conditions.push(screen());
    assert_eq!(hit(user, target, 1, false, false, 85, PokemonType::Ice), 1);
}

#[test]
fn critical_hits_ignore_only_negative_attack_and_positive_defense_stages() {
    let user = mon("User", &["test"]);
    let target = mon("Foe", &["idle"]);
    let base = hit(
        user.clone(),
        target.clone(),
        73,
        true,
        false,
        100,
        PokemonType::Ice,
    );
    assert_eq!(base, 55);
    let mut debuffed = user.clone();
    debuffed.stages.insert(Stat::Attack, -6);
    let mut buffed = target.clone();
    buffed.stages.insert(Stat::Defense, 6);
    assert_eq!(
        hit(debuffed, buffed, 73, true, false, 100, PokemonType::Ice),
        base
    );
    let mut buffed = user;
    buffed.stages.insert(Stat::Attack, 1);
    let mut debuffed = target;
    debuffed.stages.insert(Stat::Defense, -1);
    assert_eq!(
        hit(buffed, debuffed, 73, true, false, 100, PokemonType::Ice),
        123
    );
}

#[test]
fn thick_fat_and_gen8_heatproof_modify_stats_and_power_before_base_damage() {
    for ability in ["thick-fat", "heatproof", "water-bubble"] {
        let mut user = mon("User", &["test"]);
        user.level = 50;
        user.attack = 7;
        let mut target = mon("Foe", &["idle"]);
        target.defense = 6;
        target.ability = Some(ability.into());
        // Atk 7 -> 3 (Thick Fat/Water Bubble), or BP 5 -> 2 (Gen 8 Heatproof).
        // Both give 3 damage here; halving the final 4 damage would give 2.
        assert_eq!(
            hit(user, target, 5, false, false, 100, PokemonType::Fire),
            3,
            "{ability}"
        );
    }
}

#[test]
fn typeless_damage_skips_stab_and_effectiveness_and_keeps_integer_math() {
    let mut user = mon("User", &["test"]);
    user.types = vec![PokemonType::Normal];
    let mut target = mon("Foe", &["idle"]);
    target.types = vec![PokemonType::Ghost];
    assert_eq!(
        hit(user, target, 73, false, true, 100, PokemonType::Normal),
        37
    );
}
