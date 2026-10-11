//! Regression tests for the Sword/Shield ability audit.
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
fn volt_absorb_catches_status_moves_and_respects_suppression_and_bypass() {
    for (suppressed, breaker, hp, expected_hp, expected_status) in [
        (false, false, 500, 750, None),
        (false, false, 1000, 1000, None),
        (true, false, 500, 500, Some(Status::Paralyzed)),
        (false, true, 500, 500, Some(Status::Paralyzed)),
    ] {
        let mut user = mon("User", &["thunder-wave"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = hp;
        target.ability = Some("volt-absorb".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "thunder-wave", "splash");
        assert_eq!(foe(&battle).hp, expected_hp);
        assert_eq!(foe(&battle).status, expected_status);
        assert_eq!(ally(&battle).hp, 1000);
    }
}

#[test]
fn water_absorb_catches_soak_without_changing_types() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["soak"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = 500;
        target.ability = Some("water-absorb".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "soak", "splash");
        let active = !suppressed && !breaker;
        assert_eq!(foe(&battle).hp, if active { 750 } else { 500 });
        assert_eq!(
            foe(&battle).types,
            vec![if active {
                PokemonType::Normal
            } else {
                PokemonType::Water
            }]
        );
    }
}

#[test]
fn flash_fire_activates_from_will_o_wisp_and_boosts_fire_damage() {
    let run = |activate, suppressed, breaker: bool| {
        let mut user = mon("User", &["will-o-wisp", "splash"]);
        user.speed = 200;
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["ember"]);
        target.ability = Some("flash-fire".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(
            &mut battle,
            if activate { "will-o-wisp" } else { "splash" },
            "ember",
        );
        (1000 - ally(&battle).hp, foe(&battle).status)
    };
    let base = run(false, false, false).0;
    let boosted = run(true, false, false);
    assert_eq!(boosted.1, None);
    assert!(boosted.0 > base);
    assert_eq!(run(true, true, false).1, Some(Status::Burned));
    assert_eq!(run(true, false, true).1, Some(Status::Burned));
}

#[test]
fn motor_drive_absorbs_thunder_wave_and_boosts_speed_once() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["thunder-wave"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("motor-drive".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "thunder-wave", "splash");
        let active = !suppressed && !breaker;
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Speed),
            i8::from(active)
        );
        assert_eq!(foe(&battle).status, (!active).then_some(Status::Paralyzed));
    }
}

#[test]
fn sap_sipper_blocks_grass_status_moves_without_secondary_effects() {
    for move_id in ["spore", "leech-seed", "worry-seed"] {
        for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
            let mut user = mon("User", &[move_id]);
            user.ability = breaker.then(|| "mold-breaker".into());
            let mut target = mon("Target", &["splash"]);
            target.ability = Some("sap-sipper".into());
            target.ability_suppressed = suppressed;
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, move_id, "splash");
            let active = !suppressed && !breaker;
            assert_eq!(
                foe(&battle).stage(wilds_battle_engine::Stat::Attack),
                i8::from(active),
                "{move_id}"
            );
            if active {
                assert_eq!(foe(&battle).status, None);
                assert_eq!(foe(&battle).ability.as_deref(), Some("sap-sipper"));
                assert_eq!(foe(&battle).hp, 1000);
            }
        }
    }
}

#[test]
fn lightning_rod_redirects_damage_and_status_but_not_mold_breaker() {
    for move_id in ["thunder-wave", "thunderbolt"] {
        for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
            let mut user = mon("User", &[move_id]);
            user.ability = breaker.then(|| "mold-breaker".into());
            let mut rod = mon("Rod", &["splash"]);
            rod.ability = Some("lightning-rod".into());
            rod.ability_suppressed = suppressed;
            let mut battle = Battle::with_rng(
                Fixed(0.5),
                [vec![user], vec![mon("Selected", &["splash"]), rod]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            turn(&mut battle, move_id, "splash");
            let active = !suppressed && !breaker;
            let rod = &battle.participants(Side::Foes)[1];
            assert_eq!(
                rod.stage(wilds_battle_engine::Stat::SpAttack),
                i8::from(active)
            );
            assert_eq!(rod.hp, 1000);
            assert_eq!(rod.status, None);
            if active {
                assert_eq!(foe(&battle).hp, 1000);
                assert_eq!(foe(&battle).status, None);
            } else if move_id == "thunder-wave" {
                assert_eq!(foe(&battle).status, Some(Status::Paralyzed));
            } else {
                assert!(foe(&battle).hp < 1000);
            }
        }
    }
}

#[test]
fn storm_drain_redirects_damage_and_status_but_not_mold_breaker() {
    for move_id in ["soak", "water-gun"] {
        for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
            let mut user = mon("User", &[move_id]);
            user.ability = breaker.then(|| "mold-breaker".into());
            let mut rod = mon("Rod", &["splash"]);
            rod.ability = Some("storm-drain".into());
            rod.ability_suppressed = suppressed;
            let mut battle = Battle::with_rng(
                Fixed(0.5),
                [vec![user], vec![mon("Selected", &["splash"]), rod]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            turn(&mut battle, move_id, "splash");
            let active = !suppressed && !breaker;
            let rod = &battle.participants(Side::Foes)[1];
            assert_eq!(
                rod.stage(wilds_battle_engine::Stat::SpAttack),
                i8::from(active)
            );
            assert_eq!(rod.hp, 1000);
            assert_eq!(rod.status, None);
            if active {
                assert_eq!(foe(&battle).hp, 1000);
                assert_eq!(foe(&battle).status, None);
            } else if move_id == "soak" {
                assert_eq!(foe(&battle).types, vec![PokemonType::Water]);
            } else {
                assert!(foe(&battle).hp < 1000);
            }
        }
    }
}

fn ability_damage(move_id: &str, attacker: &str, defender: &str, suppressed: bool) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.ability = (!attacker.is_empty()).then(|| attacker.into());
    let mut target = mon("Target", &["splash"]);
    target.ability = (!defender.is_empty()).then(|| defender.into());
    target.ability_suppressed = suppressed;
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, move_id, "splash");
    1000 - foe(&battle).hp
}

#[test]
fn heatproof_halves_fire_damage_and_is_bypassable() {
    for move_id in ["ember", "fire-punch"] {
        let base = ability_damage(move_id, "", "", false);
        let reduced = ability_damage(move_id, "", "heatproof", false);
        assert!((i32::from(base) - 2 * i32::from(reduced)).abs() <= 1);
        assert_eq!(
            base,
            ability_damage(move_id, "mold-breaker", "heatproof", false)
        );
        assert_eq!(base, ability_damage(move_id, "", "heatproof", true));
    }
    assert_eq!(
        ability_damage("tackle", "", "heatproof", false),
        ability_damage("tackle", "", "", false)
    );
}

#[test]
fn water_bubble_boosts_water_and_resists_fire() {
    for move_id in ["water-gun", "aqua-jet"] {
        let base = ability_damage(move_id, "", "", false);
        let boosted = ability_damage(move_id, "water-bubble", "", false);
        assert!((i32::from(boosted) - 2 * i32::from(base)).abs() <= 1);
    }
    let base = ability_damage("ember", "", "", false);
    let reduced = ability_damage("ember", "", "water-bubble", false);
    assert!((i32::from(base) - 2 * i32::from(reduced)).abs() <= 1);
    assert_eq!(
        base,
        ability_damage("ember", "mold-breaker", "water-bubble", false)
    );
    assert_eq!(base, ability_damage("ember", "", "water-bubble", true));
    assert_eq!(
        ability_damage("tackle", "water-bubble", "", false),
        ability_damage("tackle", "", "", false)
    );
}

#[test]
fn punk_rock_boosts_only_sound_attacks_and_reduces_incoming_sound() {
    let base = ability_damage("hyper-voice", "", "", false);
    let boosted = ability_damage("hyper-voice", "punk-rock", "", false);
    assert!((f32::from(boosted) - f32::from(base) * 1.3).abs() < 2.0);
    let reduced = ability_damage("hyper-voice", "", "punk-rock", false);
    assert!((i32::from(base) - 2 * i32::from(reduced)).abs() <= 1);
    assert_eq!(
        base,
        ability_damage("hyper-voice", "mold-breaker", "punk-rock", false)
    );
    assert_eq!(
        ability_damage("tackle", "punk-rock", "", false),
        ability_damage("tackle", "", "", false)
    );
}

#[test]
fn oblivious_prevents_captivate_unless_suppressed_or_bypassed() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["captivate"]);
        user.gender = wilds_battle_engine::Gender::Male;
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.gender = wilds_battle_engine::Gender::Female;
        target.ability = Some("oblivious".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "captivate", "splash");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::SpAttack),
            if suppressed || breaker { -2 } else { 0 }
        );
    }
}

#[test]
fn hyper_cutter_blocks_external_attack_drops_only() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["growl", "tail-whip"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["superpower", "splash"]);
        target.ability = Some("hyper-cutter".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "growl", "splash");
        let external = if suppressed || breaker { -1 } else { 0 };
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Attack),
            external
        );
        turn(&mut battle, "tail-whip", "superpower");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Attack),
            external - 1
        );
        assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Defense), -2);
    }
}

#[test]
fn flower_veil_protects_grass_teammate_stats_with_bypass_and_type_checks() {
    for (grass, breaker, suppressed) in [
        (true, false, false),
        (false, false, false),
        (true, true, false),
        (true, false, true),
    ] {
        let mut user = mon("User", &["growl", "tail-whip"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.types = vec![if grass {
            PokemonType::Grass
        } else {
            PokemonType::Normal
        }];
        let mut veil = mon("Veil", &["splash"]);
        veil.ability = Some("flower-veil".into());
        veil.ability_suppressed = suppressed;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target, veil]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "growl", "splash");
        turn(&mut battle, "tail-whip", "splash");
        let expected = if grass && !breaker && !suppressed {
            0
        } else {
            -1
        };
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Attack),
            expected
        );
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Defense),
            expected
        );
    }
}

#[test]
fn infiltrator_bypasses_mist_but_not_ability_stat_protection() {
    for (infiltrator, defender, expected) in
        [(false, "", 0), (true, "", -1), (true, "clear-body", 0)]
    {
        let mut user = mon("User", &["growl"]);
        user.ability = infiltrator.then(|| "infiltrator".into());
        let mut target = mon("Target", &["mist"]);
        target.speed = 200;
        target.ability = (!defender.is_empty()).then(|| defender.into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "growl", "mist");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Attack),
            expected
        );
    }
}
