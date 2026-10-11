//! Regression tests for the Sword/Shield status audit.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, Pokemon,
    PokemonType, Side, Status,
};

struct Fixed(f32);
impl BattleRng for Fixed {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        self.0
    }
}

fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|m| m.to_string()).collect());
    p.hp = 1000;
    p.max_hp = 1000;
    p.types = vec![PokemonType::Normal];
    p
}

fn game(user: Pokemon, target: Pokemon, dice: f32) -> Battle {
    Battle::with_rng(
        Fixed(dice),
        [vec![user], vec![target]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap()
}

fn turn(battle: &mut Battle, ally: &str, foe: &str) -> Vec<BattleEvent> {
    let start = battle.turn();
    let mut events = vec![];
    for _ in 0..30 {
        let result = battle.advance().unwrap();
        events.extend(result.events);
        if battle.turn() > start {
            return events;
        }
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            return events;
        };
        let wanted = if prompt.actor.side == Side::Allies {
            ally
        } else {
            foe
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| match c {
                Choice::UseMove { move_id, .. } => move_id == wanted,
                Choice::Switch { to, .. } => wanted == format!("switch:{}", to.index),
            })
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id(),
            })
            .unwrap();
    }
    panic!("turn did not finish");
}

fn foe(battle: &Battle) -> &Pokemon {
    &battle.participants(Side::Foes)[0]
}
fn ally(battle: &Battle) -> &Pokemon {
    &battle.participants(Side::Allies)[0]
}

#[test]
fn burn_residual_uses_gen8_fraction_floor_and_active_abilities() {
    for (ability, suppressed, max_hp, loss) in [
        ("", false, 159, 9),
        ("", false, 15, 1),
        ("heatproof", false, 159, 4),
        ("heatproof", false, 15, 1),
        ("magic-guard", false, 159, 0),
        ("magic-guard", true, 159, 9),
        ("heatproof", true, 159, 9),
    ] {
        let mut target = mon("Target", &["splash"]);
        target.hp = max_hp;
        target.max_hp = max_hp;
        target.status = Some(Status::Burned);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(mon("User", &["splash"]), target, 0.5);
        turn(&mut battle, "splash", "splash");
        assert_eq!(
            foe(&battle).hp,
            max_hp - loss,
            "{ability} suppressed={suppressed}"
        );
    }
}

#[test]
fn burn_immunity_abilities_prevent_and_cure_burn_unless_suppressed() {
    for ability in ["water-veil", "water-bubble"] {
        for suppressed in [false, true] {
            for pre_burned in [false, true] {
                let mut target = mon("Target", &["splash"]);
                target.status = pre_burned.then_some(Status::Burned);
                target.ability = Some(ability.into());
                target.ability_suppressed = suppressed;
                let mut battle = game(mon("User", &["will-o-wisp"]), target, 0.5);
                turn(&mut battle, "will-o-wisp", "splash");
                assert_eq!(foe(&battle).status, suppressed.then_some(Status::Burned));
            }
        }
    }
}

fn attack_damage(move_id: &str, status: Option<Status>, ability: Option<&str>) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.status = status;
    user.ability = ability.map(str::to_owned);
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, move_id, "splash");
    1000 - foe(&battle).hp
}

#[test]
fn burn_halves_physical_damage_except_facade_and_guts() {
    let normal = attack_damage("tackle", None, None);
    let burned = attack_damage("tackle", Some(Status::Burned), None);
    assert!((i32::from(normal) - 2 * i32::from(burned)).abs() <= 1);
    assert_eq!(
        attack_damage("swift", None, None),
        attack_damage("swift", Some(Status::Burned), None)
    );
    assert_eq!(
        attack_damage("facade", Some(Status::Burned), None),
        attack_damage("facade", Some(Status::Poisoned), None)
    );
    let guts = attack_damage("tackle", Some(Status::Burned), Some("guts"));
    assert!(guts > normal);
    assert_eq!(
        guts,
        attack_damage("tackle", Some(Status::Poisoned), Some("guts"))
    );
    assert_eq!(normal, attack_damage("tackle", None, Some("guts")));
}

#[test]
fn flare_boost_only_boosts_special_attacks_while_burned() {
    assert!(
        attack_damage("swift", Some(Status::Burned), Some("flare-boost"))
            > attack_damage("swift", None, Some("flare-boost"))
    );
    assert_eq!(
        attack_damage("tackle", Some(Status::Burned), Some("flare-boost")),
        attack_damage("tackle", Some(Status::Burned), None)
    );
}

#[test]
fn poison_rounds_down_and_toxic_multiplies_the_floored_unit_with_a_capped_counter() {
    for (status, stage, loss, next) in [
        (Status::Poisoned, 0, 19, 0),
        (Status::BadlyPoisoned, 0, 9, 2),
        (Status::BadlyPoisoned, 2, 18, 3),
        (Status::BadlyPoisoned, 15, 135, 15),
        (Status::BadlyPoisoned, 255, 135, 15),
    ] {
        let mut target = mon("Target", &["splash"]);
        target.hp = 159;
        target.max_hp = 159;
        target.status = Some(status);
        target.status_turns = stage;
        let mut battle = game(mon("User", &["splash"]), target, 0.5);
        turn(&mut battle, "splash", "splash");
        assert_eq!(foe(&battle).hp, 159 - loss);
        assert_eq!(foe(&battle).status_turns, next);
    }
}

#[test]
fn poison_heal_and_magic_guard_keep_toxic_counting_and_respect_suppression() {
    for (ability, suppressed, move_id, hp) in [
        ("poison-heal", false, "splash", 119),
        ("poison-heal", true, "splash", 82),
        ("poison-heal", false, "heal-block", 100),
        ("magic-guard", false, "splash", 100),
        ("magic-guard", true, "splash", 82),
    ] {
        let mut target = mon("Target", &["splash"]);
        target.hp = 100;
        target.max_hp = 159;
        target.status = Some(Status::BadlyPoisoned);
        target.status_turns = 2;
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(mon("User", &[move_id]), target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            foe(&battle).hp,
            hp,
            "{ability} {move_id} suppressed={suppressed}"
        );
        assert_eq!(foe(&battle).status_turns, 3);
    }
}

#[test]
fn toxic_resets_on_switching_but_keeps_the_bad_poison_status() {
    let mut poisoned = mon("Poisoned", &["splash"]);
    poisoned.status = Some(Status::BadlyPoisoned);
    poisoned.status_turns = 10;
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [
            vec![poisoned, mon("Bench", &["splash"])],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut battle, "switch:1", "splash");
    assert_eq!(ally(&battle).status, Some(Status::BadlyPoisoned));
    assert_eq!(ally(&battle).status_turns, 1);
    turn(&mut battle, "switch:0", "splash");
    assert_eq!(ally(&battle).hp, 938);
    assert_eq!(ally(&battle).status_turns, 2);
}

#[test]
fn poison_immunity_and_pastel_veil_protect_and_cure_active_allies() {
    for ability in ["immunity", "pastel-veil"] {
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::BadlyPoisoned);
        target.ability = Some(ability.into());
        let mut battle = game(mon("User", &["toxic"]), target, 0.5);
        turn(&mut battle, "toxic", "splash");
        assert_eq!(foe(&battle).status, None);
    }
    let mut poisoned = mon("Poisoned", &["splash"]);
    poisoned.status = Some(Status::Poisoned);
    let mut veil = mon("Veil", &["splash"]);
    veil.ability = Some("pastel-veil".into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![mon("User", &["toxic"])], vec![poisoned, veil]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0, 1]]).unwrap();
    turn(&mut battle, "toxic", "splash");
    assert!(
        battle
            .participants(Side::Foes)
            .iter()
            .all(|p| p.status.is_none())
    );
}

#[test]
fn corrosion_bypasses_poison_status_type_immunity_but_not_damage_immunity() {
    for kind in [PokemonType::Steel, PokemonType::Poison] {
        let mut user = mon("User", &["toxic"]);
        user.ability = Some("corrosion".into());
        let mut target = mon("Target", &["splash"]);
        target.types = vec![kind];
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "toxic", "splash");
        assert_eq!(foe(&battle).status, Some(Status::BadlyPoisoned));
    }
    let mut user = mon("User", &["sludge-bomb"]);
    user.ability = Some("corrosion".into());
    let mut target = mon("Target", &["splash"]);
    target.types = vec![PokemonType::Steel];
    let mut battle = game(user, target, 0.01);
    turn(&mut battle, "sludge-bomb", "splash");
    assert_eq!(foe(&battle).status, None);
    assert_eq!(foe(&battle).hp, 1000);
}

#[test]
fn poison_type_toxic_bypasses_accuracy_and_semi_invulnerability() {
    for poison in [false, true] {
        let mut user = mon("User", &["toxic"]);
        user.speed = 1;
        if poison {
            user.types = vec![PokemonType::Poison];
        }
        let mut battle = game(user, mon("Target", &["fly"]), 0.99);
        turn(&mut battle, "toxic", "fly");
        assert_eq!(foe(&battle).status, poison.then_some(Status::BadlyPoisoned));
    }
}

#[test]
fn toxic_boost_only_boosts_physical_power_while_poisoned() {
    assert!(
        attack_damage("tackle", Some(Status::Poisoned), Some("toxic-boost"))
            > attack_damage("tackle", None, Some("toxic-boost"))
    );
    assert_eq!(
        attack_damage("swift", Some(Status::Poisoned), Some("toxic-boost")),
        attack_damage("swift", Some(Status::Poisoned), None)
    );
}

#[test]
fn merciless_criticals_respect_critical_immunity() {
    for armor in [None, Some("battle-armor"), Some("shell-armor")] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some("merciless".into());
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::Poisoned);
        target.ability = armor.map(str::to_owned);
        let mut battle = game(user, target, 0.5);
        let events = turn(&mut battle, "tackle", "splash");
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "A critical hit!")),
            armor.is_none()
        );
    }
}

#[test]
fn paralysis_limber_prevents_and_cures_without_consuming_a_berry() {
    for pre_paralyzed in [false, true] {
        let mut target = mon("Target", &["splash"]);
        target.status = pre_paralyzed.then_some(Status::Paralyzed);
        target.ability = Some("limber".into());
        target.item = Some("cheri-berry".into());
        let mut battle = game(mon("User", &["glare"]), target, 0.5);
        turn(&mut battle, "glare", "splash");
        assert_eq!(foe(&battle).status, None);
        assert_eq!(foe(&battle).item.as_deref(), Some("cheri-berry"));
    }
}

#[test]
fn quick_feet_removes_speed_penalty_but_not_full_paralysis() {
    for (ability, suppressed, first) in [
        (None, false, "Target"),
        (Some("quick-feet"), false, "User"),
        (Some("quick-feet"), true, "Target"),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.speed = 100;
        user.status = Some(Status::Paralyzed);
        user.ability = ability.map(str::to_owned);
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["tackle"]);
        target.speed = 120;
        let mut battle = game(user.clone(), target.clone(), 0.5);
        let events = turn(&mut battle, "tackle", "tackle");
        let used = events
            .iter()
            .find_map(|e| match e {
                BattleEvent::Message(m) if m.contains(" used ") => Some(m),
                _ => None,
            })
            .unwrap();
        assert!(used.starts_with(first), "{used}");
        let mut battle = game(user, target, 0.1);
        turn(&mut battle, "tackle", "tackle");
        assert_eq!(foe(&battle).hp, 1000);
        assert_eq!(
            ally(&battle).move_pp["tackle"],
            battle.catalog().get("tackle").unwrap().pp
        );
    }
}

#[test]
fn paralysis_checks_after_flinch_and_confusion() {
    let mut user = mon("User", &["splash"]);
    user.status = Some(Status::Paralyzed);
    user.confused_turns = Some(2);
    let mut battle = game(user.clone(), mon("Target", &["splash"]), 0.1);
    turn(&mut battle, "splash", "splash");
    assert!(ally(&battle).hp < 1000);
    assert_eq!(ally(&battle).confused_turns, Some(1));
    user.flinched = true;
    let mut battle = game(user, mon("Target", &["splash"]), 0.1);
    let events = turn(&mut battle, "splash", "splash");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m == "User flinched!"))
    );
    assert_eq!(ally(&battle).confused_turns, Some(2));
}

#[test]
fn sleep_lasts_one_to_three_attempts_and_early_bird_can_wake_immediately() {
    for turns in 1..=3 {
        for early in [false, true] {
            let mut user = mon("User", &["tackle"]);
            user.status = Some(Status::Asleep);
            user.status_turns = turns;
            if early {
                user.ability = Some("early-bird".into());
            }
            let mut battle = game(user, mon("Target", &["splash"]), 0.5);
            let blocked = if early { turns / 2 } else { turns };
            for _ in 0..blocked {
                turn(&mut battle, "tackle", "splash");
                assert_eq!(foe(&battle).hp, 1000);
                assert_eq!(ally(&battle).status, Some(Status::Asleep));
            }
            turn(&mut battle, "tackle", "splash");
            assert_eq!(ally(&battle).status, None);
            assert!(foe(&battle).hp < 1000);
        }
    }
}

#[test]
fn rest_sleeps_two_turns_or_one_with_early_bird() {
    for early in [false, true] {
        let mut user = mon("User", &["rest", "tackle"]);
        user.hp = 500;
        user.status = Some(Status::Poisoned);
        if early {
            user.ability = Some("early-bird".into());
        }
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "rest", "splash");
        assert_eq!(ally(&battle).hp, 1000);
        assert_eq!(ally(&battle).status, Some(Status::Asleep));
        for _ in 0..if early { 1 } else { 2 } {
            turn(&mut battle, "tackle", "splash");
            assert_eq!(foe(&battle).hp, 1000);
        }
        turn(&mut battle, "tackle", "splash");
        assert!(foe(&battle).hp < 1000);
        assert_eq!(ally(&battle).status, None);
    }
}

#[test]
fn sleep_immunity_and_sweet_veil_distinguish_prevention_from_curing() {
    for ability in ["insomnia", "vital-spirit", "sweet-veil"] {
        let mut user = mon("User", &["rest"]);
        user.ability = Some(ability.into());
        user.hp = 500;
        let mut battle = game(user.clone(), mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "rest", "splash");
        assert_eq!(ally(&battle).hp, 500);
        assert_eq!(ally(&battle).status, None);
        user.status = Some(Status::Asleep);
        user.status_turns = 3;
        let mut battle = game(user, mon("Target", &["yawn"]), 0.5);
        turn(&mut battle, "rest", "yawn");
        assert_eq!(
            ally(&battle).status,
            (ability == "sweet-veil").then_some(Status::Asleep)
        );
    }
    let mut veil = mon("Veil", &["splash"]);
    veil.ability = Some("sweet-veil".into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [
            vec![mon("User", &["yawn", "spore"])],
            vec![mon("Target", &["splash"]), veil],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle, "yawn", "splash");
    assert!(foe(&battle).condition("yawn").is_none());
    turn(&mut battle, "spore", "splash");
    assert_eq!(foe(&battle).status, None);
}

#[test]
fn sleep_talk_cannot_use_rest_to_reset_sleep_and_heal() {
    let mut user = mon("User", &["sleep-talk", "rest"]);
    user.hp = 500;
    user.status = Some(Status::Asleep);
    user.status_turns = 3;
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "sleep-talk", "splash");
    assert_eq!(ally(&battle).hp, 500);
    assert_eq!(ally(&battle).status_turns, 2);
}

#[test]
fn comatose_counts_as_sleep_for_snore_dream_eater_and_wake_up_slap() {
    let mut user = mon("User", &["snore"]);
    user.ability = Some("Comatose".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "snore", "splash");
    assert!(foe(&battle).hp < 1000);
    assert_eq!(ally(&battle).status, None);
    for move_id in ["dream_eater", "wake-up-slap", "hex"] {
        let run = |ability: Option<&str>| {
            let mut target = mon("Target", &["splash"]);
            target.types = vec![PokemonType::Water];
            target.ability = ability.map(str::to_owned);
            let mut battle = game(mon("User", &[move_id]), target, 0.5);
            turn(&mut battle, move_id, "splash");
            1000 - foe(&battle).hp
        };
        assert!(run(Some("comatose")) > run(None), "{move_id}");
    }
}

#[test]
fn bad_dreams_damages_sleep_and_comatose_but_respects_magic_guard() {
    for (ability, asleep, expected) in [
        (None, true, 19),
        (Some("comatose"), false, 19),
        (Some("magic-guard"), true, 0),
        (None, false, 0),
    ] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some("bad-dreams".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = 159;
        target.max_hp = 159;
        target.ability = ability.map(str::to_owned);
        target.status = asleep.then_some(Status::Asleep);
        target.status_turns = 2;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "splash", "splash");
        assert_eq!(foe(&battle).hp, 159 - expected);
    }
}

#[test]
fn uproar_blocks_rest_and_wakes_sleeping_switch_ins() {
    let mut user = mon("User", &["uproar"]);
    user.speed = 200;
    let mut target = mon("Target", &["rest"]);
    target.hp = 500;
    let mut bench = mon("Bench", &["splash"]);
    bench.status = Some(Status::Asleep);
    bench.status_turns = 3;
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![target, bench]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut battle, "uproar", "rest");
    assert!(foe(&battle).hp < 500);
    assert_eq!(foe(&battle).status, None);
    turn(&mut battle, "uproar", "switch:1");
    assert_eq!(battle.participants(Side::Foes)[1].status, None);
}

#[test]
fn confusion_own_tempo_prevents_new_confusion_and_cures_existing_confusion() {
    for suppressed in [false, true] {
        for confused in [false, true] {
            let mut target = mon("Target", &["splash"]);
            target.ability = Some("own-tempo".into());
            target.ability_suppressed = suppressed;
            target.confused_turns = confused.then_some(3);
            let mut battle = game(mon("User", &["confuse_ray"]), target, 0.5);
            turn(&mut battle, "confuse_ray", "splash");
            assert_eq!(foe(&battle).confused_turns.is_some(), suppressed);
        }
    }
}

#[test]
fn confusion_self_hit_uses_floored_40_power_damage_and_a_random_roll() {
    struct Roll(f32);
    impl BattleRng for Roll {
        fn next_u64(&mut self) -> u64 {
            0
        }
        fn fraction(&mut self) -> f32 {
            0.1
        }
        fn damage_roll(&mut self) -> f32 {
            self.0
        }
    }
    for (roll, expected) in [(0.85, 17), (1.0, 21)] {
        for (status, ability) in [
            (None, None),
            (Some(Status::Burned), Some("guts")),
            (None, Some("huge-power")),
            (None, Some("magic-guard")),
        ] {
            let mut user = mon("User", &["tackle"]);
            user.level = 53;
            user.attack = 103;
            user.defense = 97;
            user.confused_turns = Some(1);
            user.status = status;
            user.ability = ability.map(str::to_owned);
            let mut battle = Battle::with_rng(
                Roll(roll),
                [vec![user], vec![mon("Target", &["splash"])]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            let events = turn(&mut battle, "tackle", "splash");
            let damage = events.iter().find_map(|e| match e {
                BattleEvent::Damage { target, amount, .. } if target.side == Side::Allies => {
                    Some(*amount)
                }
                _ => None,
            });
            assert_eq!(damage, Some(expected));
            assert_eq!(foe(&battle).hp, 1000);
            turn(&mut battle, "tackle", "splash");
            assert_eq!(ally(&battle).confused_turns, None);
            assert!(foe(&battle).hp < 1000);
        }
    }
}

#[test]
fn tangled_feet_halves_accuracy_only_while_confused_and_preserves_always_hit_moves() {
    for (move_id, confused, suppressed, hit) in [
        ("tackle", true, false, false),
        ("tackle", false, false, true),
        ("tackle", true, true, true),
        ("swift", true, false, true),
    ] {
        let mut user = mon("User", &[move_id]);
        user.speed = 200;
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("tangled-feet".into());
        target.ability_suppressed = suppressed;
        target.confused_turns = confused.then_some(3);
        let mut battle = game(user, target, 0.75);
        turn(&mut battle, move_id, "splash");
        assert_eq!(foe(&battle).hp < 1000, hit);
    }
}

#[test]
fn freeze_fire_hits_thaw_before_the_targets_pending_action() {
    for attack in ["ember", "fire-punch", "fire-spin", "flamethrower"] {
        let mut user = mon("User", &[attack]);
        user.speed = 200;
        let mut target = mon("Target", &["tackle"]);
        target.status = Some(Status::Frozen);
        let mut battle = game(user, target, 0.5);
        let events = turn(&mut battle, attack, "tackle");
        assert_eq!(foe(&battle).status, None, "{attack}");
        assert!(
            ally(&battle).hp < 1000,
            "thawed target should act: {attack}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "Target thawed out!"))
        );
    }
}

#[test]
fn freeze_fire_status_misses_immunity_and_substitute_do_not_thaw() {
    for case in ["status", "miss", "flash-fire", "substitute"] {
        let attack = if case == "status" {
            "will-o-wisp"
        } else {
            "fire-blast"
        };
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::Frozen);
        if case == "flash-fire" {
            target.ability = Some("flash-fire".into());
        }
        if case == "substitute" {
            target.substitute_hp = 500;
        }
        let mut battle = game(
            mon("User", &[attack]),
            target,
            if case == "miss" { 0.99 } else { 0.5 },
        );
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).status, Some(Status::Frozen), "{case}");
    }
}

#[test]
fn freeze_uses_actual_attack_type_and_nonfire_thawing_moves() {
    for attack in ["weather-ball", "scald", "steam-eruption", "scorching-sands"] {
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::Frozen);
        let mut battle = game(mon("User", &["sunny_day", attack]), target, 0.5);
        if attack == "weather-ball" {
            turn(&mut battle, "sunny_day", "splash");
            assert_eq!(
                foe(&battle).status,
                Some(Status::Frozen),
                "sun does not cure an existing freeze"
            );
        }
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).status, None, "{attack}");
    }
}

#[test]
fn freeze_sun_and_magma_armor_prevent_freezing() {
    for case in ["plain", "sun", "magma", "suppressed", "ice"] {
        let mut target = mon("Target", &["splash"]);
        if case == "magma" || case == "suppressed" {
            target.ability = Some("magma-armor".into());
        }
        target.ability_suppressed = case == "suppressed";
        if case == "ice" {
            target.types = vec![PokemonType::Ice];
        }
        let mut battle = game(mon("User", &["sunny_day", "ice-beam"]), target, 0.05);
        if case == "sun" {
            turn(&mut battle, "sunny_day", "splash");
        }
        // Inspect the application event even if the target naturally thaws this turn.
        let events = turn(&mut battle, "ice-beam", "splash");
        let frozen = events.iter().any(|e| {
            matches!(
                e,
                BattleEvent::Status {
                    status: Some(Status::Frozen),
                    ..
                }
            )
        });
        assert_eq!(frozen, case == "plain" || case == "suppressed", "{case}");
    }
}

#[test]
fn freeze_magma_armor_cures_existing_freeze_without_consuming_a_berry() {
    let mut target = mon("Target", &["splash"]);
    target.status = Some(Status::Frozen);
    target.ability = Some("magma-armor".into());
    target.item = Some("aspear-berry".into());
    let mut battle = game(mon("User", &["splash"]), target, 0.5);
    turn(&mut battle, "splash", "splash");
    assert_eq!(foe(&battle).status, None);
    assert_eq!(foe(&battle).item.as_deref(), Some("aspear-berry"));
}

#[test]
fn freeze_burn_up_only_self_thaws_a_fire_type() {
    for fire in [false, true] {
        let mut user = mon("User", &["burn-up"]);
        user.status = Some(Status::Frozen);
        if fire {
            user.types = vec![PokemonType::Fire];
        }
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "burn-up", "splash");
        assert_eq!(ally(&battle).status.is_none(), fire);
        assert_eq!(foe(&battle).hp < 1000, fire);
    }
}

#[test]
fn freeze_all_sword_shield_self_thawing_moves_work() {
    for attack in [
        "flame-wheel",
        "sacred-fire",
        "flare-blitz",
        "fusion-flare",
        "scald",
        "steam-eruption",
        "burn-up",
        "pyro-ball",
        "scorching-sands",
    ] {
        let mut user = mon("User", &[attack]);
        user.types = vec![PokemonType::Fire];
        user.status = Some(Status::Frozen);
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, attack, "splash");
        assert_eq!(ally(&battle).status, None, "{attack}");
        assert!(foe(&battle).hp < 1000, "{attack}");
    }
}

#[test]
fn freeze_sun_immunity_respects_weather_suppressing_abilities() {
    for ability in ["cloud-nine", "air-lock"] {
        let mut user = mon("User", &["sunny_day", "ice-beam"]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.speed = 200;
        let mut battle = game(user, target, 0.05);
        turn(&mut battle, "sunny_day", "splash");
        turn(&mut battle, "ice-beam", "splash");
        assert_eq!(foe(&battle).status, Some(Status::Frozen), "{ability}");
    }
}

#[test]
fn freeze_self_thaw_requires_getting_past_flinch() {
    let mut user = mon("User", &["pyro-ball"]);
    user.status = Some(Status::Frozen);
    let mut battle = game(user, mon("Target", &["fake-out"]), 0.5);
    turn(&mut battle, "pyro-ball", "fake-out");
    assert_eq!(ally(&battle).status, Some(Status::Frozen));
    assert_eq!(ally(&battle).move_pp["pyro-ball"], 5);
    assert_eq!(foe(&battle).hp, 1000);
}
