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
        let wanted = if prompt.actor.index > 0 {
            "splash"
        } else if prompt.actor.side == Side::Allies {
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

#[test]
fn rattled_boosts_speed_from_bug_dark_and_ghost_hits_except_substitutes() {
    for move_id in ["bug-bite", "bite", "shadow-ball", "tackle"] {
        for (suppressed, substitute) in [(false, false), (true, false), (false, true)] {
            let mut target = mon("Target", &["splash"]);
            target.types = vec![PokemonType::Water];
            target.ability = Some("rattled".into());
            target.ability_suppressed = suppressed;
            target.substitute_hp = if substitute { 250 } else { 0 };
            let mut battle = game(mon("User", &[move_id]), target, 0.5);
            turn(&mut battle, move_id, "splash");
            assert_eq!(
                foe(&battle).stage(wilds_battle_engine::Stat::Speed),
                i8::from(!suppressed && !substitute && move_id != "tackle")
            );
        }
    }
}

#[test]
fn long_reach_removes_contact_for_hit_reactions_and_fluffy() {
    for (ability, suppressed, burned) in [
        ("", false, true),
        ("long-reach", false, false),
        ("long-reach", true, true),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        user.speed = 200;
        let mut battle = game(user, mon("Target", &["beak-blast"]), 0.5);
        turn(&mut battle, "tackle", "beak-blast");
        assert_eq!(ally(&battle).status, burned.then_some(Status::Burned));
        assert_eq!(foe(&battle).last_hit.as_ref().unwrap().contact, burned);
    }
    assert_eq!(
        ability_damage("tackle", "long-reach", "fluffy", false),
        ability_damage("tackle", "", "", false)
    );
    assert!(
        ability_damage("tackle", "", "fluffy", false) < ability_damage("tackle", "", "", false)
    );
}

#[test]
fn scrappy_bypasses_ghost_immunity_for_normal_fighting_and_fixed_damage() {
    for move_id in ["tackle", "brick-break", "seismic-toss", "ember"] {
        for suppressed in [false, true] {
            let mut user = mon("User", &[move_id]);
            user.ability = Some("scrappy".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.types = vec![PokemonType::Ghost];
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, move_id, "splash");
            assert_eq!(
                foe(&battle).hp < 1000,
                !suppressed || move_id == "ember",
                "{move_id}"
            );
        }
    }
}

fn check_weather_suppression(ability: &str) {
    let run = |weather_move, attack, ability: &str, suppressed| {
        let mut user = mon("User", &[weather_move, attack]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        user.speed = 200;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, weather_move, "splash");
        assert!(battle.weather().is_some());
        let before = foe(&battle).hp;
        turn(&mut battle, attack, "splash");
        (
            before - foe(&battle).hp,
            ally(&battle).hp,
            battle.weather().unwrap().turns_left,
        )
    };
    for weather in ["sunny_day", "rain-dance"] {
        assert_eq!(
            run(weather, "ember", ability, false).0,
            ability_damage("ember", "", "", false)
        );
        assert_eq!(
            run(weather, "weather-ball", ability, false).0,
            ability_damage("weather-ball", "", "", false)
        );
        assert_eq!(
            run(weather, "ember", ability, true).0,
            run(weather, "ember", "", false).0
        );
    }
    for weather in ["hail", "sandstorm"] {
        let cloud = run(weather, "tackle", ability, false);
        assert_eq!(cloud.0, ability_damage("tackle", "", "", false));
        assert_eq!(cloud.1, 1000);
        assert!(cloud.2 < 5);
        assert!(run(weather, "tackle", ability, true).1 < 1000);
    }
}

#[test]
fn cloud_nine_suppresses_weather_damage_modifiers_and_lua_weather_without_clearing_it() {
    check_weather_suppression("cloud-nine");
}

#[test]
fn air_lock_suppresses_weather_damage_and_lua_weather() {
    check_weather_suppression("air-lock");
}

#[test]
fn overcoat_prevents_sand_and_hail_damage_only_while_active() {
    for move_id in ["hail", "sandstorm"] {
        for suppressed in [false, true] {
            let mut target = mon("Target", &["splash"]);
            target.ability = Some("overcoat".into());
            target.ability_suppressed = suppressed;
            let mut battle = game(mon("User", &[move_id]), target, 0.5);
            turn(&mut battle, move_id, "splash");
            assert_eq!(foe(&battle).hp, if suppressed { 938 } else { 1000 });
            assert_eq!(ally(&battle).hp, 938);
        }
    }
}

#[test]
fn dry_skin_absorbs_status_water_and_responds_to_sun_rain_and_fire() {
    for (move_id, expected) in [("soak", 750), ("rain-dance", 625), ("sunny_day", 375)] {
        let mut target = mon("Target", &["splash"]);
        target.hp = 500;
        target.ability = Some("dry-skin".into());
        let mut battle = game(mon("User", &[move_id]), target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(foe(&battle).hp, expected, "{move_id}");
        assert_eq!(foe(&battle).types, vec![PokemonType::Normal]);
    }
    let base = ability_damage("ember", "", "", false);
    assert!(ability_damage("ember", "", "dry-skin", false) > base);
    assert_eq!(
        ability_damage("ember", "mold-breaker", "dry-skin", false),
        base
    );
    for ability in ["cloud-nine", "air-lock"] {
        let mut user = mon("User", &["sunny_day"]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("dry-skin".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "sunny_day", "splash");
        assert_eq!(foe(&battle).hp, 1000);
    }
}

#[test]
fn magic_guard_blocks_indirect_damage_but_keeps_hp_costs_and_struggle_recoil() {
    for attack in ["leech-seed", "sandstorm", "hail"] {
        let mut user = mon("User", &[attack]);
        user.hp = 500;
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("magic-guard".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).hp, 1000);
        if attack == "leech-seed" {
            assert_eq!(ally(&battle).hp, 500);
        }
    }
    for attack in ["double_edge", "steel-beam", "mind-blown"] {
        let mut user = mon("User", &[attack]);
        user.ability = Some("magic-guard".into());
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, attack, "splash");
        assert_eq!(ally(&battle).hp, 1000, "{attack}");
        assert!(foe(&battle).hp < 1000);
    }
    let mut user = mon("User", &["belly-drum"]);
    user.ability = Some("magic-guard".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "belly-drum", "splash");
    assert_eq!(ally(&battle).hp, 500);
    let mut user = mon("User", &["tackle"]);
    user.ability = Some("magic-guard".into());
    user.move_pp.insert("tackle".into(), 0);
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "struggle", "splash");
    assert_eq!(ally(&battle).hp, 750);
}

#[test]
fn magic_guard_stops_binding_and_nightmare_but_not_direct_hits() {
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("magic-guard".into());
    let mut battle = game(mon("User", &["wrap", "splash"]), target, 0.5);
    turn(&mut battle, "wrap", "splash");
    let hp = foe(&battle).hp;
    assert!(hp < 1000);
    turn(&mut battle, "splash", "splash");
    assert_eq!(foe(&battle).hp, hp);
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("magic-guard".into());
    target.status = Some(Status::Asleep);
    target.status_turns = 3;
    let mut battle = game(mon("User", &["nightmare"]), target, 0.5);
    turn(&mut battle, "nightmare", "splash");
    assert_eq!(foe(&battle).hp, 1000);
}

#[test]
fn normalize_changes_damage_and_status_types_and_boosts_power_except_variable_types() {
    for move_id in ["ember", "thunder-wave"] {
        for (kind, expected) in [(PokemonType::Ghost, false), (PokemonType::Ground, true)] {
            let mut user = mon("User", &[move_id]);
            user.ability = Some("normalize".into());
            let mut target = mon("Target", &["splash"]);
            target.types = vec![kind];
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, move_id, "splash");
            assert_eq!(
                foe(&battle).hp < 1000 || foe(&battle).status.is_some(),
                expected,
                "{move_id}"
            );
            assert_eq!(ally(&battle).last_move_type, Some(PokemonType::Normal));
        }
    }
    assert!(
        ability_damage("tackle", "normalize", "", false) > ability_damage("tackle", "", "", false)
    );
    let run = |ability: &str| {
        let mut user = mon("User", &["rain-dance", "weather-ball"]);
        user.ability = Some(ability.into());
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "rain-dance", "splash");
        turn(&mut battle, "weather-ball", "splash");
        assert_eq!(ally(&battle).last_move_type, Some(PokemonType::Water));
        foe(&battle).hp
    };
    assert_eq!(run("normalize"), run(""));
}

fn check_knockout_boost(ability: &str, stat: wilds_battle_engine::Stat) {
    for (hp, suppressed, boosts) in [(1, false, 2), (1000, false, 0), (1, true, 0)] {
        let mut user = mon("User", &["surf"]);
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        user.speed = 200;
        let mut target = mon("Target", &["splash"]);
        target.hp = hp;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target.clone(), target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "surf", "splash");
        assert_eq!(ally(&battle).stage(stat), boosts, "{ability} hp={hp}");
    }
}

#[test]
fn as_one_glastrier_gains_attack_for_each_move_knockout() {
    check_knockout_boost("as-one-glastrier", wilds_battle_engine::Stat::Attack);
}

#[test]
fn as_one_spectrier_gains_sp_attack_after_all_spread_hits_resolve() {
    check_knockout_boost("as-one-spectrier", wilds_battle_engine::Stat::SpAttack);
    let run = |ability: &str| {
        let mut user = mon("User", &["surf"]);
        user.ability = Some(ability.into());
        user.speed = 200;
        let mut first = mon("First", &["splash"]);
        first.hp = 1;
        let second = mon("Second", &["splash"]);
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![first, second]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "surf", "splash");
        (
            battle.participants(Side::Foes)[1].hp,
            ally(&battle).stage(wilds_battle_engine::Stat::SpAttack),
        )
    };
    let boosted = run("as-one-spectrier");
    assert_eq!(boosted.1, 1);
    assert_eq!(boosted.0, run("").0);
}

#[test]
fn chillingneigh_rewards_move_knockouts() {
    check_knockout_boost("chilling-neigh", wilds_battle_engine::Stat::Attack);
}

#[test]
fn grimneigh_rewards_move_knockouts() {
    check_knockout_boost("grim-neigh", wilds_battle_engine::Stat::SpAttack);
}

#[test]
fn moxie_rewards_move_knockouts() {
    check_knockout_boost("moxie", wilds_battle_engine::Stat::Attack);
}

fn check_power_ability(ability: &str, boosted: &[&str], neutral: &str, multiplier: f32, hp: u16) {
    let run = |move_id: &str, suppressed| {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        user.hp = hp;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, move_id, "splash");
        1000 - foe(&battle).hp
    };
    for &attack in boosted {
        let base = run(attack, true);
        let modified = run(attack, false);
        assert!(modified > base, "{ability}: {attack} did not boost");
        assert!(
            (f32::from(modified) - f32::from(base) * multiplier).abs() <= 3.0,
            "{ability} {attack}: base={base} modified={modified}"
        );
    }
    assert_eq!(
        run(neutral, false),
        run(neutral, true),
        "{ability} boosted unrelated {neutral}"
    );
}

#[test]
fn steelworker_boosts_steel_attacks_only() {
    check_power_ability("steelworker", &["iron-head"], "tackle", 1.5, 1000);
}

#[test]
fn transistor_boosts_matching_moves_only() {
    check_power_ability("transistor", &["thunderbolt"], "tackle", 1.5, 1000);
}

#[test]
fn dragonsmaw_boosts_matching_moves_only() {
    check_power_ability("dragons-maw", &["dragon-breath"], "tackle", 1.5, 1000);
}

#[test]
fn ironfist_boosts_matching_moves_only() {
    check_power_ability(
        "iron-fist",
        &["mach-punch", "ice-punch"],
        "tackle",
        1.2,
        1000,
    );
}

#[test]
fn strongjaw_boosts_matching_moves_only() {
    check_power_ability("strong-jaw", &["bite", "crunch"], "tackle", 1.5, 1000);
}

#[test]
fn toughclaws_boosts_matching_moves_only() {
    check_power_ability("tough-claws", &["tackle", "bite"], "thunderbolt", 1.3, 1000);
}

#[test]
fn technician_boosts_matching_moves_only() {
    check_power_ability("technician", &["tackle", "swift"], "hyper_beam", 1.5, 1000);
}

#[test]
fn overgrow_boosts_matching_moves_only() {
    check_power_ability("overgrow", &["mega-drain"], "tackle", 1.5, 333);
    check_power_ability("overgrow", &[], "mega-drain", 1.0, 334);
}

#[test]
fn blaze_boosts_matching_moves_only() {
    check_power_ability("blaze", &["ember"], "tackle", 1.5, 333);
    check_power_ability("blaze", &[], "ember", 1.0, 334);
}

#[test]
fn torrent_boosts_matching_moves_only() {
    check_power_ability("torrent", &["water-gun"], "tackle", 1.5, 333);
    check_power_ability("torrent", &[], "water-gun", 1.0, 334);
}

#[test]
fn swarm_boosts_matching_moves_only() {
    check_power_ability("swarm", &["bug-bite"], "tackle", 1.5, 333);
    check_power_ability("swarm", &[], "bug-bite", 1.0, 334);
}

#[test]
fn mega_launcher_boosts_pulses_and_heal_pulse() {
    check_power_ability(
        "mega-launcher",
        &["aura-sphere", "dark-pulse"],
        "water-gun",
        1.5,
        1000,
    );
    for suppressed in [false, true] {
        let mut user = mon("User", &["heal-pulse"]);
        user.ability = Some("mega-launcher".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.hp = 1;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "heal-pulse", "splash");
        assert_eq!(foe(&battle).hp, if suppressed { 501 } else { 751 });
    }
}

fn check_plus_minus(ability: &str) {
    let run = |partner: &str, suppressed, attack: &str| {
        let mut user = mon("User", &[attack]);
        user.ability = Some(ability.into());
        let mut teammate = mon("Partner", &["splash"]);
        teammate.ability = Some(partner.into());
        teammate.ability_suppressed = suppressed;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, teammate], vec![mon("Target", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, attack, "splash");
        1000 - foe(&battle).hp
    };
    let base = run("", false, "water-gun");
    for partner in ["plus", "minus"] {
        assert!(run(partner, false, "water-gun") > base);
        assert_eq!(run(partner, true, "water-gun"), base);
        assert_eq!(run(partner, false, "tackle"), run("", false, "tackle"));
    }
}

#[test]
fn plus_needs_an_active_plus_or_minus_teammate() {
    check_plus_minus("plus");
}

#[test]
fn minus_needs_an_active_plus_or_minus_teammate() {
    check_plus_minus("minus");
}

#[test]
fn defeatist_halves_both_attacking_stats_at_half_hp() {
    let run = |attack: &str, hp, suppressed| {
        let mut user = mon("User", &[attack]);
        user.hp = hp;
        user.ability = Some("defeatist".into());
        user.ability_suppressed = suppressed;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, attack, "splash");
        1000 - foe(&battle).hp
    };
    for attack in ["tackle", "water-gun"] {
        assert!(run(attack, 500, false) < run(attack, 501, false));
        assert_eq!(run(attack, 500, true), run(attack, 501, false));
    }
}

#[test]
fn hustle_boosts_attack_and_reduces_only_physical_accuracy() {
    assert!(
        ability_damage("tackle", "hustle", "", false) > ability_damage("tackle", "", "", false)
    );
    assert_eq!(
        ability_damage("water-gun", "hustle", "", false),
        ability_damage("water-gun", "", "", false)
    );
    for (attack, hits) in [("tackle", false), ("water-gun", true), ("aerial-ace", true)] {
        let mut user = mon("User", &[attack]);
        user.ability = Some("hustle".into());
        let mut battle = game(user, mon("Target", &["splash"]), 0.85);
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).hp < 1000, hits);
    }
}

#[test]
fn compound_eyes_boosts_damage_and_status_accuracy_only_when_active() {
    for attack in ["fire-blast", "will-o-wisp"] {
        for suppressed in [false, true] {
            let mut user = mon("User", &[attack]);
            user.ability = Some("compound-eyes".into());
            user.ability_suppressed = suppressed;
            let mut battle = game(user, mon("Target", &["splash"]), 0.9);
            turn(&mut battle, attack, "splash");
            assert_eq!(
                foe(&battle).hp < 1000 || foe(&battle).status.is_some(),
                !suppressed
            );
        }
    }
}

#[test]
fn victory_star_boosts_self_and_allies_and_stacks() {
    for (user_star, ally_star, expected) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (true, true, true),
    ] {
        let mut user = mon("User", &["focus-blast"]);
        user.ability = user_star.then(|| "victory-star".into());
        let mut partner = mon("Partner", &["splash"]);
        partner.ability = ally_star.then(|| "victory-star".into());
        let mut battle = Battle::with_rng(
            Fixed(0.8),
            [vec![user, partner], vec![mon("Target", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "focus-blast", "splash");
        assert_eq!(foe(&battle).hp < 1000, expected);
    }
}

#[test]
fn keen_eye_blocks_accuracy_drops_and_ignores_positive_evasion() {
    for breaker in [false, true] {
        let mut user = mon("User", &["sand-attack"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("keen-eye".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "sand-attack", "splash");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Accuracy),
            if breaker { -1 } else { 0 }
        );
    }
    for suppressed in [false, true] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some("keen-eye".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.stages.insert(wilds_battle_engine::Stat::Evasion, 6);
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(foe(&battle).hp < 1000, !suppressed);
    }
}

#[test]
fn big_pecks_blocks_external_defense_drops_only() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["tail-whip", "growl"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["superpower", "splash"]);
        target.ability = Some("big-pecks".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "tail-whip", "splash");
        let external = if suppressed || breaker { -1 } else { 0 };
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Defense),
            external
        );
        turn(&mut battle, "growl", "superpower");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Defense),
            external - 1
        );
        assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Attack), -2);
    }
}

#[test]
fn thickfat_halves_fire_and_ice_damage_and_is_bypassable() {
    for move_id in ["ember", "fire-punch", "ice-beam", "ice-punch"] {
        let base = ability_damage(move_id, "", "", false);
        let reduced = ability_damage(move_id, "", "thickfat", false);
        assert!((i32::from(base) - 2 * i32::from(reduced)).abs() <= 1);
        assert_eq!(
            base,
            ability_damage(move_id, "mold-breaker", "thickfat", false)
        );
        assert_eq!(base, ability_damage(move_id, "", "thickfat", true));
    }
    assert_eq!(
        ability_damage("tackle", "", "thickfat", false),
        ability_damage("tackle", "", "", false)
    );
}

#[test]
fn adaptability_increases_stab_to_double_without_boosting_other_types() {
    let normal = ability_damage("tackle", "", "", false);
    let boosted = ability_damage("tackle", "adaptability", "", false);
    assert!((f32::from(boosted) - f32::from(normal) * 4.0 / 3.0).abs() <= 1.0);
    assert_eq!(
        ability_damage("water-gun", "adaptability", "", false),
        ability_damage("water-gun", "", "", false)
    );
}

#[test]
fn no_guard_hits_from_either_side_and_reaches_hidden_targets() {
    for on_user in [true, false] {
        let mut user = mon("User", &["fire-blast"]);
        user.speed = 200;
        let mut target = mon("Target", &["splash"]);
        target.semi_invulnerable = true;
        target.hidden_kind = wilds_battle_engine::HiddenKind::Air;
        if on_user {
            user.ability = Some("no-guard".into());
        } else {
            target.ability = Some("no-guard".into());
        }
        let mut battle = game(user, target, 0.99);
        turn(&mut battle, "fire-blast", "splash");
        assert!(foe(&battle).hp < 1000);
    }
    for (target_level, kind, ko) in [
        (50, PokemonType::Normal, true),
        (51, PokemonType::Normal, false),
        (50, PokemonType::Ghost, false),
    ] {
        let mut user = mon("User", &["guillotine"]);
        user.level = 50;
        user.ability = Some("no-guard".into());
        let mut target = mon("Target", &["splash"]);
        target.level = target_level;
        target.types = vec![kind];
        let mut battle = game(user, target, 0.99);
        turn(&mut battle, "guillotine", "splash");
        assert_eq!(foe(&battle).hp == 0, ko);
    }
    let mut user = mon("User", &["tackle"]);
    user.ability = Some("no-guard".into());
    let mut battle = game(user, mon("Target", &["protect"]), 0.5);
    turn(&mut battle, "tackle", "protect");
    assert_eq!(foe(&battle).hp, 1000);
}

#[test]
fn tintedlens_only_modifies_the_relevant_type_matchups() {
    for (kind, multiplier) in [
        (PokemonType::Rock, 2.0),
        (PokemonType::Normal, 1.0),
        (PokemonType::Ghost, 1.0),
    ] {
        let run = |suppressed| {
            let mut user = mon("User", &["tackle"]);
            user.ability = Some("tinted-lens".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.types = vec![kind];
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, "tackle", "splash");
            1000 - foe(&battle).hp
        };
        assert!((f32::from(run(false)) - f32::from(run(true)) * multiplier).abs() <= 1.0);
    }
}

#[test]
fn neuroforce_only_modifies_the_relevant_type_matchups() {
    for (kind, multiplier) in [
        (PokemonType::Grass, 1.25),
        (PokemonType::Normal, 1.0),
        (PokemonType::Water, 1.0),
    ] {
        let run = |suppressed| {
            let mut user = mon("User", &["ember"]);
            user.ability = Some("neuroforce".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.types = vec![kind];
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, "ember", "splash");
            1000 - foe(&battle).hp
        };
        assert!((f32::from(run(false)) - f32::from(run(true)) * multiplier).abs() <= 1.0);
    }
}

#[test]
fn super_luck_adds_a_critical_stage_but_respects_critical_immunity() {
    for (suppressed, armor, critical) in [
        (false, false, true),
        (true, false, false),
        (false, true, false),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some("super-luck".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.ability = armor.then(|| "battle-armor".into());
        let mut battle = game(user, target, 0.1);
        let events = turn(&mut battle, "tackle", "splash");
        assert_eq!(
            events.iter().any(
                |event| matches!(event, BattleEvent::Message(text) if text == "A critical hit!")
            ),
            critical
        );
    }
}

#[test]
fn sniper_only_boosts_critical_damage() {
    let normal = ability_damage("frost-breath", "", "", false);
    let boosted = ability_damage("frost-breath", "sniper", "", false);
    assert!((f32::from(boosted) - f32::from(normal) * 1.5).abs() <= 1.0);
    assert_eq!(
        ability_damage("tackle", "sniper", "", false),
        ability_damage("tackle", "", "", false)
    );
    assert_eq!(
        ability_damage("frost-breath", "sniper", "battle-armor", false),
        ability_damage("frost-breath", "", "battle-armor", false)
    );
}

#[test]
fn rivalry_scales_damage_by_gender_and_leaves_genderless_targets_unchanged() {
    for (gender, factor) in [
        (wilds_battle_engine::Gender::Male, 1.25),
        (wilds_battle_engine::Gender::Female, 0.75),
        (wilds_battle_engine::Gender::Genderless, 1.0),
    ] {
        let run = |suppressed| {
            let mut user = mon("User", &["water-gun"]);
            user.gender = wilds_battle_engine::Gender::Male;
            user.ability = Some("rivalry".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.gender = gender;
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, "water-gun", "splash");
            1000 - foe(&battle).hp
        };
        assert!((f32::from(run(false)) - f32::from(run(true)) * factor).abs() <= 2.0);
    }
}

fn weight_damage(ability: &str, suppressed: bool, breaker: bool) -> u16 {
    let mut user = mon("User", &["low-kick"]);
    user.ability = breaker.then(|| "mold-breaker".into());
    let mut target = mon("Target", &["splash"]);
    target.weight = 120.0;
    target.ability = Some(ability.into());
    target.ability_suppressed = suppressed;
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "low-kick", "splash");
    1000 - foe(&battle).hp
}

#[test]
fn heavy_metal_modifies_battle_weight_without_compounding_autotomize() {
    assert!(weight_damage("heavy-metal", false, false) > weight_damage("", false, false));
    assert_eq!(
        weight_damage("heavy-metal", true, false),
        weight_damage("", false, false)
    );
    assert_eq!(
        weight_damage("heavy-metal", false, true),
        weight_damage("", false, false)
    );
    let mut user = mon("User", &["autotomize"]);
    user.ability = Some("heavy-metal".into());
    user.weight = 120.0;
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "autotomize", "splash");
    assert_eq!(ally(&battle).weight, 20.0);
}

#[test]
fn light_metal_halves_weight_and_respects_suppression_and_mold_breaker() {
    assert!(weight_damage("light-metal", false, false) < weight_damage("", false, false));
    assert_eq!(
        weight_damage("light-metal", true, false),
        weight_damage("", false, false)
    );
    assert_eq!(
        weight_damage("light-metal", false, true),
        weight_damage("", false, false)
    );
}

#[test]
fn speed_boost_triggers_each_turn_except_the_switch_in_turn() {
    let mut lead = mon("Lead", &["splash"]);
    lead.ability = Some("speed-boost".into());
    let reserve = lead.clone();
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![lead, reserve], vec![mon("Target", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).stage(wilds_battle_engine::Stat::Speed), 1);
    turn(&mut battle, "switch:1", "splash");
    assert_eq!(
        battle.participants(Side::Allies)[1].stage(wilds_battle_engine::Stat::Speed),
        0
    );
    turn(&mut battle, "splash", "splash");
    assert_eq!(
        battle.participants(Side::Allies)[1].stage(wilds_battle_engine::Stat::Speed),
        1
    );
}

fn check_weather_entry(ability: &str, kind: wilds_battle_engine::WeatherKind) {
    for suppressed in [false, true] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        battle.advance().unwrap();
        assert_eq!(
            battle.weather().map(|w| (w.kind, w.turns_left)),
            (!suppressed).then_some((kind, 5))
        );
    }
    let lead = mon("Lead", &["splash"]);
    let mut reserve = mon("Reserve", &["splash"]);
    reserve.ability = Some(ability.into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![lead, reserve], vec![mon("Target", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    battle.advance().unwrap();
    assert!(battle.weather().is_none());
    turn(&mut battle, "switch:1", "splash");
    assert_eq!(battle.weather().unwrap().kind, kind);
}

#[test]
fn drizzle_starts_rain_on_entry_only_when_active() {
    check_weather_entry("drizzle", wilds_battle_engine::WeatherKind::Rain);
}

#[test]
fn drought_starts_weather_on_entry_only_when_active() {
    check_weather_entry("drought", wilds_battle_engine::WeatherKind::Sun);
}

#[test]
fn sandstream_starts_weather_on_entry_only_when_active() {
    check_weather_entry("sand-stream", wilds_battle_engine::WeatherKind::Sandstorm);
}

#[test]
fn snowwarning_starts_weather_on_entry_only_when_active() {
    check_weather_entry("snow-warning", wilds_battle_engine::WeatherKind::Hail);
}

fn check_weather_healing(ability: &str, weather_move: &str) {
    for (suppressed, cloud, expected) in [
        (false, false, 562),
        (true, false, if weather_move == "hail" { 438 } else { 500 }),
        (false, true, 500),
    ] {
        let mut user = mon("User", &[weather_move]);
        user.ability = cloud.then(|| "cloud-nine".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = 500;
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, weather_move, "splash");
        assert_eq!(foe(&battle).hp, expected);
    }
}

#[test]
fn rain_dish_heals_one_sixteenth_in_effective_rain() {
    check_weather_healing("rain-dish", "rain-dance");
}

#[test]
fn ice_body_heals_in_hail_and_prevents_hail_damage() {
    check_weather_healing("ice-body", "hail");
}

fn check_weather_speed(ability: &str, weather: &str) {
    for (suppressed, cloud, faster) in [
        (false, false, true),
        (true, false, false),
        (false, true, false),
    ] {
        let mut user = mon("User", &[weather, "tackle"]);
        user.speed = 60;
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash", "tackle"]);
        target.speed = 100;
        target.ability = cloud.then(|| "cloud-nine".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, weather, "splash");
        let events = turn(&mut battle, "tackle", "tackle");
        let first = events
            .iter()
            .find_map(|event| match event {
                BattleEvent::Message(text) if text.contains("used Tackle!") => Some(text.as_str()),
                _ => None,
            })
            .unwrap();
        assert_eq!(first.starts_with("User"), faster, "{ability}");
    }
}

#[test]
fn swift_swim_doubles_speed_in_effective_rain() {
    check_weather_speed("swift-swim", "rain-dance");
}

#[test]
fn chlorophyll_doubles_speed_in_effective_weather() {
    check_weather_speed("chlorophyll", "sunny_day");
}

#[test]
fn slushrush_doubles_speed_in_effective_weather() {
    check_weather_speed("slush-rush", "hail");
}

fn check_weather_immunity(ability: &str, weather: &str) {
    for suppressed in [false, true] {
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(mon("User", &[weather]), target, 0.5);
        turn(&mut battle, weather, "splash");
        assert_eq!(foe(&battle).hp, if suppressed { 938 } else { 1000 });
    }
}

#[test]
fn sand_rush_doubles_speed_and_prevents_sandstorm_damage() {
    check_weather_speed("sand-rush", "sandstorm");
    check_weather_immunity("sand-rush", "sandstorm");
}

#[test]
fn solar_power_boosts_special_attacks_and_costs_hp_only_in_effective_sun() {
    let run = |attack: &str, suppressed, cloud: bool| {
        let mut user = mon("User", &["sunny_day", attack]);
        user.ability = Some("solar-power".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.ability = cloud.then(|| "cloud-nine".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "sunny_day", "splash");
        turn(&mut battle, attack, "splash");
        (1000 - foe(&battle).hp, ally(&battle).hp)
    };
    assert!(run("water-gun", false, false).0 > run("water-gun", true, false).0);
    assert_eq!(run("tackle", false, false).0, run("tackle", true, false).0);
    assert_eq!(run("water-gun", false, false).1, 750);
    assert_eq!(run("water-gun", true, false).1, 1000);
    assert_eq!(run("water-gun", false, true), run("water-gun", true, true));
}

fn check_weather_evasion(ability: &str, weather: &str) {
    for (suppressed, source_ability, hit) in [
        (false, "", false),
        (true, "", true),
        (false, "mold-breaker", true),
        (false, "cloud-nine", true),
    ] {
        let mut user = mon("User", &[weather, "will-o-wisp"]);
        user.ability = Some(source_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.75);
        turn(&mut battle, weather, "splash");
        turn(&mut battle, "will-o-wisp", "splash");
        assert_eq!(foe(&battle).status, hit.then_some(Status::Burned));
    }
}

#[test]
fn sand_veil_reduces_accuracy_and_grants_weather_immunity() {
    check_weather_evasion("sand-veil", "sandstorm");
    check_weather_immunity("sand-veil", "sandstorm");
}

#[test]
fn snow_cloak_reduces_accuracy_and_grants_hail_immunity() {
    check_weather_evasion("snow-cloak", "hail");
    check_weather_immunity("snow-cloak", "hail");
}

#[test]
fn sand_force_boosts_only_rock_ground_and_steel_in_sand() {
    check_weather_immunity("sand-force", "sandstorm");
    for attack in ["rock-slide", "earthquake", "iron-head", "tackle"] {
        let run = |suppressed| {
            let mut user = mon("User", &["sandstorm", attack]);
            user.ability = Some("sand-force".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.types = vec![PokemonType::Ground];
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, "sandstorm", "splash");
            turn(&mut battle, attack, "splash");
            1000 - foe(&battle).hp
        };
        if attack == "tackle" {
            assert_eq!(run(false), run(true));
        } else {
            assert!(run(false) > run(true));
        }
    }
    assert_eq!(
        ability_damage("iron-head", "sand-force", "", false),
        ability_damage("iron-head", "", "", false)
    );
}

fn check_repeated_entry_boost(ability: &str, stat: wilds_battle_engine::Stat) {
    let mut lead = mon("Lead", &["splash"]);
    lead.ability = Some(ability.into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [
            vec![lead, mon("Reserve", &["splash"])],
            vec![mon("Target", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    battle.advance().unwrap();
    assert_eq!(ally(&battle).stage(stat), 1);
    turn(&mut battle, "switch:1", "splash");
    // Choose the switch explicitly because the helper defaults other active slots to Splash.
    let result = battle.advance().unwrap();
    let AdvanceStatus::Awaiting(prompt) = result.status else {
        panic!("no switch prompt");
    };
    let choice = prompt
        .choices
        .iter()
        .find(|c| matches!(c, Choice::Switch { to, .. } if to.index == 0))
        .unwrap();
    battle
        .set_response(ActionSelection {
            prompt_id: prompt.id,
            choice_id: choice.id(),
        })
        .unwrap();
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).stage(stat), 1);
}

#[test]
fn intrepid_sword_boosts_attack_on_every_gen8_entry() {
    check_repeated_entry_boost("intrepid-sword", wilds_battle_engine::Stat::Attack);
}

#[test]
fn dauntless_shield_boosts_defense_on_every_gen8_entry() {
    check_repeated_entry_boost("dauntless-shield", wilds_battle_engine::Stat::Defense);
}

#[test]
fn download_compares_opponents_combined_staged_defenses_and_breaks_ties_to_special() {
    use wilds_battle_engine::Stat;
    for (first_def, second_def, stage, boosted) in [
        (50, 50, 0, Stat::Attack),
        (100, 100, 0, Stat::SpAttack),
        (50, 150, 0, Stat::SpAttack),
        (50, 100, 2, Stat::SpAttack),
    ] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some("download".into());
        let mut first = mon("First", &["splash"]);
        first.defense = first_def;
        first.sp_defense = 100;
        first.stages.insert(Stat::Defense, stage);
        let mut second = mon("Second", &["splash"]);
        second.defense = second_def;
        second.sp_defense = 100;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![first, second]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        battle.advance().unwrap();
        assert_eq!(ally(&battle).stage(boosted), 1);
        assert_eq!(
            ally(&battle).stage(if boosted == Stat::Attack {
                Stat::SpAttack
            } else {
                Stat::Attack
            }),
            0
        );
    }
}

#[test]
fn curious_medicine_resets_only_active_teammates() {
    use wilds_battle_engine::Stat;
    let mut holder = mon("Holder", &["splash"]);
    holder.ability = Some("curious-medicine".into());
    holder.stages.insert(Stat::Attack, 2);
    let mut partner = mon("Partner", &["splash"]);
    partner.stages.insert(Stat::Attack, 3);
    partner.stages.insert(Stat::Defense, -2);
    let bench = partner.clone();
    let target = partner.clone();
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![holder, partner, bench], vec![target]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0, 1], vec![0]]).unwrap();
    battle.advance().unwrap();
    assert_eq!(ally(&battle).stage(Stat::Attack), 2);
    assert!(battle.participants(Side::Allies)[1].stages.is_empty());
    assert_eq!(battle.participants(Side::Allies)[2].stage(Stat::Attack), 3);
    assert_eq!(foe(&battle).stage(Stat::Defense), -2);
}

#[test]
fn screen_cleaner_removes_both_sides_screens_but_preserves_mist() {
    let lead = mon("Lead", &["hail", "aurora-veil", "reflect"]);
    let mut reserve = mon("Reserve", &["splash"]);
    reserve.ability = Some("screen-cleaner".into());
    let target = mon("Target", &["splash", "light-screen", "mist"]);
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![lead, reserve], vec![target]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut battle, "hail", "splash");
    turn(&mut battle, "aurora-veil", "light-screen");
    turn(&mut battle, "reflect", "mist");
    assert_eq!(battle.side_conditions(Side::Allies).len(), 2);
    turn(&mut battle, "switch:1", "splash");
    for side in [Side::Allies, Side::Foes] {
        assert!(
            !battle.side_conditions(side).iter().any(|c| [
                "reflect",
                "light_screen",
                "aurora_veil"
            ]
            .contains(&c.name.as_str()))
        );
    }
    assert!(
        battle
            .side_conditions(Side::Foes)
            .iter()
            .any(|c| c.name == "mist")
    );
}

fn check_trap(
    ability: &str,
    kind: PokemonType,
    victim_ability: &str,
    suppressed: bool,
    trapped: bool,
) {
    let mut victim = mon("Victim", &["splash"]);
    victim.types = vec![kind];
    victim.ability = Some(victim_ability.into());
    let mut source = mon("Trapper", &["splash"]);
    source.ability = Some(ability.into());
    source.ability_suppressed = suppressed;
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![victim, mon("Bench", &["splash"])], vec![source]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    let id = wilds_battle_engine::ParticipantId {
        side: Side::Allies,
        index: 0,
    };
    assert_eq!(battle.trapped(id), trapped);
    let result = battle.advance().unwrap();
    let AdvanceStatus::Awaiting(prompt) = result.status else {
        panic!("no prompt");
    };
    assert_eq!(
        prompt
            .choices
            .iter()
            .any(|c| matches!(c, Choice::Switch { .. })),
        !trapped
    );
}

#[test]
fn shadow_tag_blocks_voluntary_switches_except_ghosts_and_other_shadow_tag_users() {
    check_trap("shadow-tag", PokemonType::Normal, "", false, true);
    check_trap("shadow-tag", PokemonType::Ghost, "", false, false);
    check_trap(
        "shadow-tag",
        PokemonType::Normal,
        "shadow-tag",
        false,
        false,
    );
    check_trap("shadow-tag", PokemonType::Normal, "", true, false);
}

#[test]
fn arena_trap_only_prevents_grounded_non_ghost_switches() {
    check_trap("arena-trap", PokemonType::Normal, "", false, true);
    check_trap("arena-trap", PokemonType::Flying, "", false, false);
    check_trap("arena-trap", PokemonType::Normal, "levitate", false, false);
    check_trap("arena-trap", PokemonType::Ghost, "", false, false);
    check_trap("arena-trap", PokemonType::Normal, "", true, false);
}

#[test]
fn magnet_pull_traps_only_steel_types_even_when_airborne() {
    check_trap("magnet-pull", PokemonType::Steel, "", false, true);
    check_trap("magnet-pull", PokemonType::Normal, "", false, false);
    check_trap("magnet-pull", PokemonType::Steel, "levitate", false, true);
    check_trap("magnet-pull", PokemonType::Steel, "", true, false);
}

#[test]
fn prankster_prioritizes_status_and_dark_immunity_includes_called_attacks() {
    for kind in [PokemonType::Normal, PokemonType::Dark] {
        let mut user = mon("User", &["thunder-wave"]);
        user.speed = 10;
        user.ability = Some("prankster".into());
        let mut target = mon("Target", &["tackle"]);
        target.speed = 100;
        target.types = vec![kind];
        let mut battle = game(user, target, 0.5);
        let events = turn(&mut battle, "thunder-wave", "tackle");
        let first = events
            .iter()
            .find_map(|e| match e {
                BattleEvent::Message(m) if m.contains(" used ") => Some(m),
                _ => None,
            })
            .unwrap();
        assert!(first.starts_with("User"));
        assert_eq!(
            foe(&battle).status,
            (kind != PokemonType::Dark).then_some(Status::Paralyzed)
        );
    }
    for ability in ["prankster", ""] {
        let mut user = mon("User", &["nature-power"]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.types = vec![PokemonType::Dark];
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "nature-power", "splash");
        assert_eq!(foe(&battle).hp == 1000, ability == "prankster");
    }
}

fn first_move_is_user(events: &[BattleEvent]) -> bool {
    events
        .iter()
        .find_map(|e| match e {
            BattleEvent::Message(m) if m.contains(" used ") => Some(m.starts_with("User")),
            _ => None,
        })
        .unwrap()
}

#[test]
fn gale_wings_only_prioritizes_flying_moves_at_full_health() {
    for (move_id, hp, suppressed, expected) in [
        ("aerial-ace", 1000, false, true),
        ("tailwind", 1000, false, true),
        ("aerial-ace", 999, false, false),
        ("aerial-ace", 1000, true, false),
        ("tackle", 1000, false, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some("gale-wings".into());
        user.ability_suppressed = suppressed;
        user.hp = hp;
        user.speed = 10;
        let mut target = mon("Target", &["tackle"]);
        target.speed = 100;
        let mut battle = game(user, target, 0.5);
        assert_eq!(
            first_move_is_user(&turn(&mut battle, move_id, "tackle")),
            expected,
            "{move_id}/{hp}/{suppressed}"
        );
    }
}

#[test]
fn triage_gives_healing_and_drain_moves_three_priority() {
    for (move_id, suppressed, expected) in [
        ("giga-drain", false, true),
        ("rest", false, true),
        ("wish", false, true),
        ("giga-drain", true, false),
        ("pain-split", false, false),
        ("aromatherapy", false, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some("triage".into());
        user.ability_suppressed = suppressed;
        user.hp = 500;
        user.speed = 10;
        let mut target = mon("Target", &["quick-attack"]);
        target.speed = 100;
        let mut battle = game(user, target, 0.5);
        assert_eq!(
            first_move_is_user(&turn(&mut battle, move_id, "quick-attack")),
            expected,
            "{move_id}/{suppressed}"
        );
    }
}

fn check_priority_protection(ability: &str) {
    for (move_id, suppressed, breaker, protected) in [
        ("quick-attack", false, false, true),
        ("tackle", false, false, false),
        ("quick-attack", true, false, false),
        ("quick-attack", false, true, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let target = mon("Target", &["splash"]);
        let mut partner = mon("Partner", &["splash"]);
        partner.ability = Some(ability.into());
        partner.ability_suppressed = suppressed;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target, partner]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            foe(&battle).hp == 1000,
            protected,
            "{ability}/{move_id}/{suppressed}/{breaker}"
        );
    }
}

#[test]
fn queenly_majesty_protects_allies_from_priority_and_allows_mold_breaker() {
    check_priority_protection("queenly-majesty");
}

#[test]
fn dazzling_protects_allies_from_priority_and_allows_mold_breaker() {
    check_priority_protection("dazzling");
}

fn check_contact_damage(ability: &str) {
    for (move_id, attacker_ability, suppressed, hp, expected_loss) in [
        ("tackle", "", false, 1000, 125),
        ("tackle", "", false, 1, 125),
        ("tackle", "mold-breaker", false, 1000, 125),
        ("tackle", "long-reach", false, 1000, 0),
        ("tackle", "magic-guard", false, 1000, 0),
        ("tackle", "", true, 1000, 0),
        ("swift", "", false, 1000, 0),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        target.hp = hp;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            1000 - ally(&battle).hp,
            expected_loss,
            "{ability}/{move_id}/{attacker_ability}/{suppressed}/{hp}"
        );
    }
    let user = mon("User", &["tackle", "splash"]);
    let mut target = mon("Target", &["substitute", "splash"]);
    target.ability = Some(ability.into());
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "splash", "substitute");
    turn(&mut battle, "tackle", "splash");
    assert_eq!(ally(&battle).hp, 1000);
}

#[test]
fn rough_skin_reacts_to_contact_even_on_faint_but_not_substitutes() {
    check_contact_damage("rough-skin");
}

#[test]
fn iron_barbs_reacts_to_contact_even_on_faint_but_not_substitutes() {
    check_contact_damage("iron-barbs");
}

fn check_contact_status(ability: &str, status: Status, immune_type: PokemonType) {
    for (move_id, attacker_ability, suppressed, kind, dice, hp, expected) in [
        ("tackle", "", false, PokemonType::Normal, 0.2, 1000, true),
        ("tackle", "", false, PokemonType::Normal, 0.2, 1, true),
        (
            "tackle",
            "mold-breaker",
            false,
            PokemonType::Normal,
            0.2,
            1000,
            true,
        ),
        (
            "tackle",
            "long-reach",
            false,
            PokemonType::Normal,
            0.2,
            1000,
            false,
        ),
        ("tackle", "", true, PokemonType::Normal, 0.2, 1000, false),
        ("swift", "", false, PokemonType::Normal, 0.2, 1000, false),
        ("tackle", "", false, immune_type, 0.2, 1000, false),
        ("tackle", "", false, PokemonType::Normal, 0.4, 1000, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(attacker_ability.into());
        user.types = vec![kind];
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        target.hp = hp;
        let mut battle = game(user, target, dice);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            ally(&battle).status,
            expected.then_some(status),
            "{ability}/{move_id}/{attacker_ability}/{suppressed}/{kind:?}/{dice}/{hp}"
        );
    }
}

#[test]
fn static_has_thirty_percent_contact_paralysis_with_type_immunity() {
    check_contact_status("static", Status::Paralyzed, PokemonType::Electric);
}

#[test]
fn flame_body_has_thirty_percent_contact_burn_with_type_immunity() {
    check_contact_status("flame-body", Status::Burned, PokemonType::Fire);
}

#[test]
fn poison_point_has_thirty_percent_contact_poison_with_type_immunity() {
    check_contact_status("poison-point", Status::Poisoned, PokemonType::Poison);
    check_contact_status("poison-point", Status::Poisoned, PokemonType::Steel);
}

#[test]
fn effect_spore_uses_gen8_distribution_and_powder_immunities() {
    for (dice, kind, ability, expected) in [
        (0.1, PokemonType::Normal, "", Some(Status::Asleep)),
        (0.15, PokemonType::Normal, "", Some(Status::Paralyzed)),
        (0.25, PokemonType::Normal, "", Some(Status::Poisoned)),
        (0.4, PokemonType::Normal, "", None),
        (0.1, PokemonType::Grass, "", None),
        (0.1, PokemonType::Normal, "overcoat", None),
        (0.1, PokemonType::Normal, "long-reach", None),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.types = vec![kind];
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("effect-spore".into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(ally(&battle).status, expected, "{dice}/{kind:?}/{ability}");
    }
}

#[test]
fn cute_charm_infatuates_only_opposite_gender_contact_attackers() {
    use wilds_battle_engine::Gender;
    for (gender, ability, dice, expected) in [
        (Gender::Male, "", 0.2, true),
        (Gender::Female, "", 0.2, false),
        (Gender::Genderless, "", 0.2, false),
        (Gender::Male, "oblivious", 0.2, false),
        (Gender::Male, "aroma-veil", 0.2, false),
        (Gender::Male, "long-reach", 0.2, false),
        (Gender::Male, "", 0.4, false),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.gender = gender;
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.gender = Gender::Female;
        target.ability = Some("cute-charm".into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(
            ally(&battle).conditions.iter().any(|c| c.name == "attract"),
            expected,
            "{gender:?}/{ability}/{dice}"
        );
    }
}

fn check_contact_speed_drop(ability: &str) {
    for (move_id, attacker_ability, hp, expected) in [
        ("tackle", "", 1000, -1),
        ("tackle", "", 1, -1),
        ("tackle", "mold-breaker", 1000, -1),
        ("tackle", "long-reach", 1000, 0),
        ("tackle", "clear-body", 1000, 0),
        ("tackle", "contrary", 1000, 1),
        ("swift", "", 1000, 0),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.hp = hp;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            ally(&battle).stage(wilds_battle_engine::Stat::Speed),
            expected,
            "{ability}/{move_id}/{attacker_ability}/{hp}"
        );
    }
}

#[test]
fn gooey_lowers_contact_attackers_speed_with_stat_protection() {
    check_contact_speed_drop("gooey");
}

#[test]
fn tangling_hair_lowers_contact_attackers_speed_with_stat_protection() {
    check_contact_speed_drop("tangling-hair");
}

fn check_hit_stat_boost(ability: &str, move_id: &str, stat: wilds_battle_engine::Stat, amount: i8) {
    for suppressed in [false, true] {
        let user = mon("User", &[move_id]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            foe(&battle).stage(stat),
            if suppressed { 0 } else { amount },
            "{ability}/{move_id}/{suppressed}"
        );
    }
}

#[test]
fn stamina_raises_defense_per_direct_hit() {
    use wilds_battle_engine::Stat;
    check_hit_stat_boost("stamina", "tackle", Stat::Defense, 1);
    check_hit_stat_boost("stamina", "swift", Stat::Defense, 1);
    check_hit_stat_boost("stamina", "double_kick", Stat::Defense, 2);
    check_hit_stat_boost("stamina", "growl", Stat::Defense, 0);
}

#[test]
fn weak_armor_trades_defense_for_two_speed_per_physical_hit() {
    use wilds_battle_engine::Stat;
    check_hit_stat_boost("weak-armor", "tackle", Stat::Defense, -1);
    check_hit_stat_boost("weak-armor", "tackle", Stat::Speed, 2);
    check_hit_stat_boost("weak-armor", "double_kick", Stat::Speed, 4);
    check_hit_stat_boost("weak-armor", "swift", Stat::Defense, 0);
    check_hit_stat_boost("weak-armor", "swift", Stat::Speed, 0);
}

#[test]
fn justified_boosts_attack_when_damaged_by_dark_moves() {
    use wilds_battle_engine::Stat;
    check_hit_stat_boost("justified", "bite", Stat::Attack, 1);
    check_hit_stat_boost("justified", "dark-pulse", Stat::Attack, 1);
    check_hit_stat_boost("justified", "tackle", Stat::Attack, 0);
}

#[test]
fn water_compaction_boosts_defense_by_two_when_damaged_by_water() {
    use wilds_battle_engine::Stat;
    check_hit_stat_boost("water-compaction", "water-gun", Stat::Defense, 2);
    check_hit_stat_boost("water-compaction", "tackle", Stat::Defense, 0);
}

#[test]
fn steam_engine_raises_speed_six_stages_after_water_or_fire_damage() {
    use wilds_battle_engine::Stat;
    check_hit_stat_boost("steam-engine", "water-gun", Stat::Speed, 6);
    check_hit_stat_boost("steam-engine", "ember", Stat::Speed, 6);
    check_hit_stat_boost("steam-engine", "tackle", Stat::Speed, 0);
    let user = mon("User", &["water-gun"]);
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("steam-engine".into());
    target.stages.insert(Stat::Speed, -4);
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "water-gun", "splash");
    assert_eq!(foe(&battle).stage(Stat::Speed), 2);
}

#[test]
fn sand_spit_starts_sandstorm_after_damage_but_not_status_moves() {
    for (move_id, suppressed, expected) in [
        ("tackle", false, true),
        ("swift", false, true),
        ("tackle", true, false),
        ("growl", false, false),
    ] {
        let user = mon("User", &[move_id]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("sand-spit".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            battle.weather().map(|w| w.kind),
            expected.then_some(wilds_battle_engine::WeatherKind::Sandstorm)
        );
    }
}

#[test]
fn cotton_down_lowers_every_other_active_pokemons_speed() {
    for suppressed in [false, true] {
        let user = mon("User", &["swift"]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("cotton-down".into());
        target.ability_suppressed = suppressed;
        let partner = mon("Partner", &["splash"]);
        let mut immune = mon("Immune", &["splash"]);
        immune.ability = Some("clear-body".into());
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, immune], vec![target, partner]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "swift", "splash");
        let expected = if suppressed { 0 } else { -1 };
        assert_eq!(
            ally(&battle).stage(wilds_battle_engine::Stat::Speed),
            expected
        );
        assert_eq!(
            battle.participants(Side::Foes)[1].stage(wilds_battle_engine::Stat::Speed),
            expected
        );
        assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Speed), 0);
        assert_eq!(
            battle.participants(Side::Allies)[1].stage(wilds_battle_engine::Stat::Speed),
            0
        );
    }
}

#[test]
fn cursed_body_disables_the_damaging_move_for_four_turns_including_the_hit() {
    for (dice, expected) in [(0.2, true), (0.4, false)] {
        let user = mon("User", &["swift", "splash"]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("cursed-body".into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, "swift", "splash");
        assert_eq!(
            ally(&battle).conditions.iter().any(|c| c.name == "disable"),
            expected
        );
        if expected {
            for _ in 0..3 {
                let result = battle.advance().unwrap();
                let AdvanceStatus::Awaiting(prompt) = result.status else {
                    panic!("missing choice")
                };
                assert!(
                    !prompt.choices.iter().any(
                        |c| matches!(c, Choice::UseMove { move_id, .. } if move_id == "swift")
                    )
                );
                turn(&mut battle, "splash", "splash");
            }
            assert!(!ally(&battle).conditions.iter().any(|c| c.name == "disable"));
        }
    }
}

#[test]
fn aftermath_damages_only_contact_knockouts() {
    for (move_id, attacker_ability, hp, expected) in [
        ("tackle", "", 1, 250),
        ("tackle", "", 1000, 0),
        ("swift", "", 1, 0),
        ("tackle", "long-reach", 1, 0),
        ("tackle", "magic-guard", 1, 0),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.hp = hp;
        target.ability = Some("aftermath".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(1000 - ally(&battle).hp, expected);
    }
}

#[test]
fn damp_stops_explosive_moves_and_aftermath_but_is_bypassable() {
    for move_id in [
        "explosion",
        "self-destruct",
        "mind-blown",
        "misty-explosion",
    ] {
        for breaker in [false, true] {
            let mut user = mon("User", &[move_id]);
            user.ability = breaker.then(|| "mold-breaker".into());
            let mut target = mon("Target", &["splash"]);
            target.ability = Some("damp".into());
            let mut battle = game(user, target, 0.5);
            let pp = ally(&battle).move_pp[move_id];
            turn(&mut battle, move_id, "splash");
            assert_eq!(ally(&battle).hp == 1000, !breaker, "{move_id}/{breaker}");
            assert_eq!(foe(&battle).hp == 1000, !breaker, "{move_id}/{breaker}");
            assert_eq!(ally(&battle).move_pp[move_id], pp - 1);
        }
    }
    let mut user = mon("User", &["tackle"]);
    user.ability = Some("damp".into());
    let mut target = mon("Target", &["splash"]);
    target.hp = 1;
    target.ability = Some("aftermath".into());
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "tackle", "splash");
    assert_eq!(ally(&battle).hp, 1000);
}

#[test]
fn innards_out_returns_actual_hp_lost_only_on_a_direct_knockout() {
    for (move_id, ability, hp, expected) in [
        ("tackle", "", 17, 17),
        ("swift", "", 17, 17),
        ("tackle", "", 1000, 0),
        ("tackle", "magic-guard", 17, 0),
        ("tackle", "damp", 17, 17),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.hp = hp;
        target.ability = Some("innards-out".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(1000 - ally(&battle).hp, expected);
    }
}

#[test]
fn anger_point_maximizes_attack_after_a_critical_hit() {
    use wilds_battle_engine::Stat;
    for (move_id, suppressed, expected) in [
        ("frost-breath", false, 6),
        ("tackle", false, -4),
        ("frost-breath", true, -4),
    ] {
        let user = mon("User", &[move_id]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("anger-point".into());
        target.ability_suppressed = suppressed;
        target.stages.insert(Stat::Attack, -4);
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(foe(&battle).stage(Stat::Attack), expected);
    }
}

fn check_drop_response(ability: &str, boosted_stat: wilds_battle_engine::Stat) {
    use wilds_battle_engine::Stat;
    for (move_id, source_ability, suppressed, expected) in [
        ("growl", "", false, 2),
        ("splash", "intimidate", false, 2),
        ("growl", "", true, 0),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(source_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            foe(&battle).stage(boosted_stat),
            expected - i8::from(boosted_stat == Stat::Attack)
        );
    }
    let mut user = mon("User", &["superpower"]);
    user.ability = Some(ability.into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "superpower", "splash");
    assert_eq!(
        ally(&battle).stage(boosted_stat),
        -i8::from(boosted_stat == Stat::Attack)
    );
    let user = mon("User", &["growl"]);
    let mut target = mon("Target", &["splash"]);
    target.stages.insert(Stat::Attack, -6);
    target.ability = Some(ability.into());
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "growl", "splash");
    assert_eq!(
        foe(&battle).stage(boosted_stat),
        -6 * i8::from(boosted_stat == Stat::Attack)
    );
}

#[test]
fn defiant_responds_to_opposing_stat_drops_and_intimidate() {
    check_drop_response("defiant", wilds_battle_engine::Stat::Attack);
}

#[test]
fn competitive_responds_to_opposing_stat_drops_and_intimidate() {
    check_drop_response("competitive", wilds_battle_engine::Stat::SpAttack);
}

#[test]
fn mirror_armor_reflects_stat_drops_once_and_respects_mold_breaker() {
    use wilds_battle_engine::Stat;
    for (ability, expected_user, expected_target) in [
        ("", -1, 0),
        ("mirror-armor", -1, 0),
        ("contrary", 1, 0),
        ("mold-breaker", 0, -1),
    ] {
        let mut user = mon("User", &["growl"]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("mirror-armor".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "growl", "splash");
        assert_eq!(
            ally(&battle).stage(Stat::Attack),
            expected_user,
            "{ability}"
        );
        assert_eq!(
            foe(&battle).stage(Stat::Attack),
            expected_target,
            "{ability}"
        );
    }
}

#[test]
fn serene_grace_doubles_opponent_and_self_secondary_chances() {
    for suppressed in [false, true] {
        let mut user = mon("User", &["thunderbolt", "ancient_power"]);
        user.ability = Some("serene-grace".into());
        user.ability_suppressed = suppressed;
        let mut battle = game(user, mon("Target", &["splash"]), 0.15);
        turn(&mut battle, "thunderbolt", "splash");
        assert_eq!(
            foe(&battle).status,
            (!suppressed).then_some(Status::Paralyzed)
        );
        turn(&mut battle, "ancient_power", "splash");
        assert_eq!(
            ally(&battle).stage(wilds_battle_engine::Stat::Attack),
            i8::from(!suppressed)
        );
    }
}

#[test]
fn wonder_skin_caps_status_accuracy_but_not_always_hit_moves() {
    for (move_id, attacker_ability, dice, expected) in [
        ("thunder-wave", "", 0.6, false),
        ("thunder-wave", "", 0.4, true),
        ("thunder-wave", "mold-breaker", 0.6, true),
        ("thunder-wave", "no-guard", 0.6, true),
        ("toxic", "", 0.6, false),
        ("toxic-thread", "", 0.6, false),
        ("yawn", "", 0.6, true),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("wonder-skin".into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, move_id, "splash");
        let hit = foe(&battle).status.is_some()
            || foe(&battle).conditions.iter().any(|c| c.name == "yawn");
        assert_eq!(hit, expected, "{move_id}/{attacker_ability}/{dice}");
    }
}

#[test]
fn poison_touch_poisons_contact_targets_and_respects_shield_dust() {
    for (move_id, target_ability, dice, suppressed, expected) in [
        ("tackle", "", 0.2, false, true),
        ("tackle", "", 0.4, false, false),
        ("swift", "", 0.2, false, false),
        ("tackle", "shield-dust", 0.2, false, false),
        ("tackle", "", 0.2, true, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some("poison-touch".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(target_ability.into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, move_id, "splash");
        assert_eq!(foe(&battle).status, expected.then_some(Status::Poisoned));
    }
}

#[test]
fn skill_link_maximizes_variable_hits_and_checks_multi_accuracy_once() {
    for suppressed in [false, true] {
        let mut user = mon("User", &["fury-attack"]);
        user.ability = Some("skill-link".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("stamina".into());
        let mut battle = game(user, target, 0.1);
        turn(&mut battle, "fury-attack", "splash");
        assert_eq!(
            foe(&battle).stage(wilds_battle_engine::Stat::Defense),
            if suppressed { 2 } else { 5 }
        );
    }
    struct AccuracyThenMiss(u8);
    impl BattleRng for AccuracyThenMiss {
        fn next_u64(&mut self) -> u64 {
            0
        }
        fn fraction(&mut self) -> f32 {
            self.0 += 1;
            if self.0 <= 3 { 0.1 } else { 0.95 }
        }
    }
    for move_id in ["triple_kick", "triple-axel"] {
        for suppressed in [false, true] {
            let mut user = mon("User", &[move_id]);
            user.speed = 100;
            user.ability = Some("skill-link".into());
            user.ability_suppressed = suppressed;
            let mut target = mon("Target", &["splash"]);
            target.speed = 10;
            target.ability = Some("stamina".into());
            let mut battle = Battle::with_rng(
                AccuracyThenMiss(0),
                [vec![user], vec![target]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            turn(&mut battle, move_id, "splash");
            assert_eq!(
                foe(&battle).stage(wilds_battle_engine::Stat::Defense),
                if suppressed { 1 } else { 3 },
                "{move_id}/{suppressed}"
            );
        }
    }
}

#[test]
fn rock_head_blocks_normal_recoil_but_not_crashes_or_struggle() {
    for suppressed in [false, true] {
        let mut user = mon("User", &["double_edge"]);
        user.ability = Some("rock-head".into());
        user.ability_suppressed = suppressed;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "double_edge", "splash");
        assert_eq!(ally(&battle).hp == 1000, !suppressed);
    }
    let mut user = mon("User", &["high-jump-kick"]);
    user.ability = Some("rock-head".into());
    let mut target = mon("Target", &["splash"]);
    target.types = vec![PokemonType::Ghost];
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "high-jump-kick", "splash");
    assert_eq!(ally(&battle).hp, 500);
    let mut user = mon("User", &["splash"]);
    user.ability = Some("rock-head".into());
    user.move_pp.insert("splash".into(), 0);
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "struggle", "splash");
    assert_eq!(ally(&battle).hp, 750);
}

#[test]
fn reckless_boosts_recoil_and_crash_attacks_only() {
    check_power_ability(
        "reckless",
        &["double_edge", "high-jump-kick", "jump-kick", "flare-blitz"],
        "tackle",
        1.2,
        1000,
    );
}

#[test]
fn liquid_ooze_reverses_drain_seed_and_strength_sap_even_at_full_hp() {
    for move_id in ["mega-drain", "draining_kiss", "leech-seed", "strength-sap"] {
        for hp in [500, 1000] {
            let mut user = mon("User", &[move_id]);
            user.hp = hp;
            let mut target = mon("Target", &["splash"]);
            target.ability = Some("liquid-ooze".into());
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, move_id, "splash");
            assert!(ally(&battle).hp < hp, "{move_id}/{hp}");
        }
    }
    let mut user = mon("User", &["mega-drain"]);
    user.ability = Some("magic-guard".into());
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("liquid-ooze".into());
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "mega-drain", "splash");
    assert_eq!(ally(&battle).hp, 1000);
}

fn check_normal_conversion(ability: &str, kind: PokemonType) {
    let run = |suppressed| {
        let mut user = mon("User", &["tackle", "growl", "water-gun"]);
        user.types = vec![PokemonType::None];
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(
            ally(&battle).last_move_type,
            Some(if suppressed {
                PokemonType::Normal
            } else {
                kind
            })
        );
        assert_eq!(
            foe(&battle).last_hit.as_ref().unwrap().move_type,
            if suppressed {
                PokemonType::Normal
            } else {
                kind
            }
        );
        let damage = 1000 - foe(&battle).hp;
        turn(&mut battle, "growl", "splash");
        assert_eq!(
            ally(&battle).last_move_type,
            Some(if suppressed {
                PokemonType::Normal
            } else {
                kind
            })
        );
        turn(&mut battle, "water-gun", "splash");
        assert_eq!(ally(&battle).last_move_type, Some(PokemonType::Water));
        damage
    };
    let base = run(true);
    let converted = run(false);
    assert!((f32::from(converted) - f32::from(base) * 1.2).abs() <= 2.0);
}

#[test]
fn refrigerate_converts_normal_moves_to_ice_and_boosts_power() {
    check_normal_conversion("refrigerate", PokemonType::Ice);
}

#[test]
fn aerilate_converts_normal_moves_to_flying_and_boosts_power() {
    check_normal_conversion("aerilate", PokemonType::Flying);
}

#[test]
fn pixilate_converts_normal_moves_to_fairy_and_boosts_power() {
    check_normal_conversion("pixilate", PokemonType::Fairy);
}

#[test]
fn galvanize_converts_normal_moves_to_electric_and_boosts_power() {
    check_normal_conversion("galvanize", PokemonType::Electric);
}

#[test]
fn liquid_voice_converts_sound_moves_without_an_extra_power_boost() {
    for (move_id, expected_type, damaging) in [
        ("hyper-voice", PokemonType::Water, true),
        ("growl", PokemonType::Water, false),
        ("swift", PokemonType::Normal, true),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some("liquid-voice".into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("water-absorb".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(ally(&battle).last_move_type, Some(expected_type));
        assert_eq!(
            foe(&battle).hp < 1000,
            damaging && expected_type != PokemonType::Water
        );
        assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Attack), 0);
    }
    let run = |ability: &str| {
        let mut user = mon("User", &["hyper-voice"]);
        user.types = vec![PokemonType::None];
        user.ability = Some(ability.into());
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "hyper-voice", "splash");
        foe(&battle).hp
    };
    assert_eq!(run("liquid-voice"), run(""));
}

#[test]
fn friend_guard_reduces_ally_damage_and_is_bypassed_by_mold_breaker() {
    let run = |holder_on_target, suppressed, breaker| {
        let mut user = mon("User", &["water-gun"]);
        user.ability = if breaker {
            Some("mold-breaker".into())
        } else {
            None
        };
        let mut target = mon("Target", &["splash"]);
        let mut partner = mon("Partner", &["splash"]);
        let holder = if holder_on_target {
            &mut target
        } else {
            &mut partner
        };
        holder.ability = Some("friend-guard".into());
        holder.ability_suppressed = suppressed;
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target, partner]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "water-gun", "splash");
        1000 - foe(&battle).hp
    };
    let base = run(false, true, false);
    assert_eq!(run(true, false, false), base);
    assert_eq!(run(false, false, true), base);
    assert!((f32::from(run(false, false, false)) - f32::from(base) * 0.75).abs() <= 1.0);
}

fn team_boost_damage(ability: &str, move_id: &str, partners: usize, on_user: bool) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.types = vec![PokemonType::None];
    if on_user {
        user.ability = Some(ability.into());
    }
    let mut party = vec![user];
    for _ in 0..partners {
        let mut partner = mon("Partner", &["splash"]);
        partner.ability = Some(ability.into());
        party.push(partner);
    }
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [party, vec![mon("Target", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle, move_id, "splash");
    1000 - foe(&battle).hp
}

#[test]
fn battery_boosts_special_attacks_of_allies_and_stacks() {
    let base = team_boost_damage("battery", "water-gun", 0, false);
    assert_eq!(team_boost_damage("battery", "water-gun", 0, true), base);
    for count in 1..=2 {
        let damage = team_boost_damage("battery", "water-gun", count, false);
        assert!((f32::from(damage) - f32::from(base) * 1.3f32.powi(count as i32)).abs() <= 3.0);
    }
    assert_eq!(
        team_boost_damage("battery", "tackle", 1, false),
        team_boost_damage("battery", "tackle", 0, false)
    );
}

#[test]
fn power_spot_boosts_ally_attacks_but_not_the_holders() {
    for move_id in ["water-gun", "tackle"] {
        let base = team_boost_damage("power-spot", move_id, 0, false);
        assert_eq!(team_boost_damage("power-spot", move_id, 0, true), base);
        assert!(
            (f32::from(team_boost_damage("power-spot", move_id, 1, false)) - f32::from(base) * 1.3)
                .abs()
                <= 3.0
        );
    }
}

#[test]
fn steely_spirit_boosts_steel_for_holder_and_allies_and_stacks() {
    let base = team_boost_damage("steely-spirit", "iron-head", 0, false);
    for (partners, on_user, factor) in [(0, true, 1.5), (1, false, 1.5), (1, true, 2.25)] {
        let damage = team_boost_damage("steely-spirit", "iron-head", partners, on_user);
        assert!((f32::from(damage) - f32::from(base) * factor).abs() <= 3.0);
    }
    assert_eq!(
        team_boost_damage("steely-spirit", "tackle", 1, true),
        team_boost_damage("steely-spirit", "tackle", 0, false)
    );
}

fn aura_damage(
    move_id: &str,
    attacker_ability: &str,
    defender_ability: &str,
    partner_ability: &str,
) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.ability = Some(attacker_ability.into());
    let mut target = mon("Target", &["splash"]);
    target.ability = Some(defender_ability.into());
    let mut partner = mon("Partner", &["splash"]);
    partner.ability = Some(partner_ability.into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![target, partner]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle, move_id, "splash");
    1000 - foe(&battle).hp
}

fn check_aura(ability: &str, move_id: &str) {
    let base = aura_damage(move_id, "", "", "");
    let boosted = aura_damage(move_id, "", ability, "");
    assert!((f32::from(boosted) - f32::from(base) * (4.0 / 3.0)).abs() <= 3.0);
    assert_eq!(aura_damage(move_id, ability, "", ""), boosted);
    assert_eq!(aura_damage(move_id, ability, ability, ability), boosted);
    assert_eq!(
        aura_damage("tackle", "", ability, ""),
        aura_damage("tackle", "", "", "")
    );
}

#[test]
fn dark_aura_boosts_both_sides_without_stacking() {
    check_aura("dark-aura", "dark-pulse");
}

#[test]
fn fairy_aura_boosts_both_sides_without_stacking() {
    check_aura("fairy-aura", "moonblast");
}

#[test]
fn aura_break_reverses_only_active_auras_and_is_bypassable() {
    for (move_id, aura) in [("dark-pulse", "dark-aura"), ("moonblast", "fairy-aura")] {
        let base = aura_damage(move_id, "", "", "");
        assert_eq!(aura_damage(move_id, "", "", "aura-break"), base);
        let weakened = aura_damage(move_id, "", aura, "aura-break");
        assert!((f32::from(weakened) - f32::from(base) * 0.75).abs() <= 2.0);
        assert_eq!(
            aura_damage(move_id, "mold-breaker", aura, "aura-break"),
            aura_damage(move_id, "", aura, "")
        );
    }
}

fn check_terrain_entry(ability: &str, move_id: &str, terrain: &str) {
    let mut user = mon("User", &["splash"]);
    user.ability = Some(ability.into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "splash", "splash");
    let ability_field = battle.field_conditions().to_vec();
    assert!(ability_field.iter().any(|c| c.name == terrain));
    let mut control = game(mon("User", &[move_id]), mon("Target", &["splash"]), 0.5);
    turn(&mut control, move_id, "splash");
    assert_eq!(ability_field, control.field_conditions());
    for _ in 0..4 {
        turn(&mut battle, "splash", "splash");
    }
    assert!(!battle.field_conditions().iter().any(|c| c.name == terrain));
    let mut user = mon("User", &["splash"]);
    user.ability = Some(ability.into());
    user.ability_suppressed = true;
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "splash", "splash");
    assert!(battle.field_conditions().is_empty());
}

#[test]
fn electric_surge_starts_five_turn_electric_terrain() {
    check_terrain_entry("electric-surge", "electric-terrain", "electric_terrain");
}

#[test]
fn psychic_surge_starts_five_turn_psychic_terrain() {
    check_terrain_entry("psychic-surge", "psychic-terrain", "psychic_terrain");
}

#[test]
fn misty_surge_starts_five_turn_misty_terrain() {
    check_terrain_entry("misty-surge", "misty-terrain", "misty_terrain");
}

#[test]
fn grassy_surge_starts_five_turn_grassy_terrain() {
    check_terrain_entry("grassy-surge", "grassy-terrain", "grassy_terrain");
}

#[test]
fn surge_surfer_doubles_speed_during_electric_terrain_even_when_airborne() {
    for (terrain, kind, suppressed, first) in [
        ("electric-terrain", PokemonType::Normal, false, true),
        ("electric-terrain", PokemonType::Flying, false, true),
        ("grassy-terrain", PokemonType::Normal, false, false),
        ("electric-terrain", PokemonType::Normal, true, false),
    ] {
        let mut user = mon("User", &[terrain, "tackle"]);
        user.speed = 60;
        user.types = vec![kind];
        user.ability = Some("surge-surfer".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["splash", "tackle"]);
        target.speed = 100;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, terrain, "splash");
        assert_eq!(
            first_move_is_user(&turn(&mut battle, "tackle", "tackle")),
            first
        );
    }
}

#[test]
fn grass_pelt_boosts_defense_in_grassy_terrain_and_is_bypassable() {
    let run = |terrain: &str, breaker, suppressed| {
        let mut user = mon("User", &[terrain, "tackle"]);
        user.ability = if breaker {
            Some("mold-breaker".into())
        } else {
            None
        };
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("grass-pelt".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, terrain, "splash");
        turn(&mut battle, "tackle", "splash")
            .iter()
            .filter_map(|e| match e {
                BattleEvent::Damage { target, amount, .. } if target.side == Side::Foes => {
                    Some(*amount)
                }
                _ => None,
            })
            .sum::<u16>()
    };
    let base = run("grassy-terrain", false, true);
    assert_eq!(run("electric-terrain", false, false), base);
    assert_eq!(run("grassy-terrain", true, false), base);
    let reduced = run("grassy-terrain", false, false);
    assert!((f32::from(reduced) - f32::from(base) / 1.5).abs() <= 2.0);
}

fn check_redirection_bypass(ability: &str) {
    for suppressed in [false, true] {
        let mut user = mon("User", &["thunderbolt"]);
        user.ability = Some(ability.into());
        user.ability_suppressed = suppressed;
        let target = mon("Target", &["splash"]);
        let mut redirector = mon("Redirector", &["splash"]);
        redirector.ability = Some("lightning-rod".into());
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target, redirector]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "thunderbolt", "splash");
        assert_eq!(foe(&battle).hp == 1000, suppressed);
        assert_eq!(
            battle.participants(Side::Foes)[1].stage(wilds_battle_engine::Stat::SpAttack),
            i8::from(suppressed)
        );
    }
    assert_eq!(
        ability_damage("thunderbolt", ability, "lightning-rod", false),
        0
    );
}

#[test]
fn stalwart_ignores_redirection_without_ignoring_type_absorption() {
    check_redirection_bypass("stalwart");
}

#[test]
fn propeller_tail_ignores_redirection_without_ignoring_type_absorption() {
    check_redirection_bypass("propeller-tail");
}

#[test]
fn unseen_fist_contact_moves_pierce_protection_without_reprisals() {
    for protect in ["protect", "spiky-shield", "baneful-bunker", "kings-shield"] {
        for (attack, suppressed, hits) in [
            ("tackle", false, true),
            ("aerial-ace", false, true),
            ("swift", false, false),
            ("tackle", true, false),
        ] {
            let mut user = mon("User", &[attack]);
            user.ability = Some("unseen-fist".into());
            user.ability_suppressed = suppressed;
            let mut battle = game(user, mon("Target", &[protect]), 0.5);
            turn(&mut battle, attack, protect);
            assert_eq!(
                foe(&battle).hp < 1000,
                hits,
                "{protect}/{attack}/{suppressed}"
            );
            if hits {
                assert_eq!(ally(&battle).hp, 1000);
                assert_eq!(ally(&battle).status, None);
                assert_eq!(ally(&battle).stage(wilds_battle_engine::Stat::Attack), 0);
            }
        }
    }
}

#[test]
fn mummy_replaces_contact_attackers_abilities_except_protected_ones() {
    for (move_id, ability, target_hp, expected) in [
        ("tackle", "huge-power", 1000, "mummy"),
        ("tackle", "huge-power", 1, "mummy"),
        ("swift", "huge-power", 1000, "huge-power"),
        ("tackle", "long-reach", 1000, "long-reach"),
        ("tackle", "multitype", 1000, "multitype"),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.hp = target_hp;
        target.ability = Some("mummy".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(ally(&battle).ability.as_deref(), Some(expected));
    }
}

#[test]
fn wandering_spirit_exchanges_contact_abilities_with_skill_swap_restrictions() {
    for (move_id, ability, swaps) in [
        ("tackle", "huge-power", true),
        ("swift", "huge-power", false),
        ("tackle", "long-reach", false),
        ("tackle", "multitype", false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("wandering-spirit".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(
            ally(&battle).ability.as_deref(),
            Some(if swaps { "wandering-spirit" } else { ability })
        );
        assert_eq!(
            foe(&battle).ability.as_deref(),
            Some(if swaps { ability } else { "wandering-spirit" })
        );
    }
}

#[test]
fn perish_body_counts_down_both_contact_participants_and_ignores_soundproof() {
    for attacker_ability in ["", "soundproof", "long-reach"] {
        let mut user = mon("User", &["tackle", "splash"]);
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("perish-body".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(
            ally(&battle)
                .conditions
                .iter()
                .any(|c| c.name == "perish_song"),
            attacker_ability != "long-reach"
        );
        for _ in 0..2 {
            turn(&mut battle, "splash", "splash");
        }
        assert!(ally(&battle).hp > 0 && foe(&battle).hp > 0);
        let events = turn(&mut battle, "splash", "splash");
        assert_eq!(
            ally(&battle).hp == 0 || foe(&battle).hp == 0,
            attacker_ability != "long-reach"
        );
        assert_eq!(events.iter().filter(|event| matches!(event, BattleEvent::Message(m) if m.contains("perish count fell to 0"))).count(), if attacker_ability == "long-reach" { 0 } else { 2 });
    }
}

#[test]
fn beast_boost_uses_highest_raw_stat_with_attack_first_ties() {
    use wilds_battle_engine::Stat;
    for (special_high, expected) in [(false, Stat::Attack), (true, Stat::SpDefense)] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some("beast-boost".into());
        user.attack = 200;
        user.defense = 200;
        user.sp_attack = 100;
        user.sp_defense = if special_high { 300 } else { 100 };
        user.speed = 100;
        user.stages.insert(Stat::Speed, 5);
        let mut target = mon("Target", &["splash"]);
        target.hp = 1;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "tackle", "splash");
        assert_eq!(ally(&battle).stage(expected), 1);
        assert_eq!(ally(&battle).stage(Stat::Speed), 5);
    }
}

#[test]
fn soul_heart_rewards_any_faint_including_allies_and_residual_damage() {
    use wilds_battle_engine::Stat;
    check_knockout_boost("soul-heart", Stat::SpAttack);
    let mut user = mon("User", &["splash"]);
    user.ability = Some("soul-heart".into());
    let mut partner = mon("Partner", &["splash"]);
    partner.hp = 1;
    partner.status = Some(Status::Poisoned);
    let mut target = mon("Target", &["splash"]);
    target.hp = 1;
    target.status = Some(Status::Poisoned);
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user, partner], vec![target]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).stage(Stat::SpAttack), 2);
}

fn check_ally_ability_inheritance(ability: &str) {
    for inherited in [
        "intimidate",
        "multitype",
        "trace",
        "receiver",
        "power-of-alchemy",
        "forecast",
        "wonder-guard",
        "gulp-missile",
    ] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some(ability.into());
        let mut partner = mon("Partner", &["splash"]);
        partner.ability = Some(inherited.into());
        partner.hp = 1;
        partner.status = Some(Status::Poisoned);
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, partner], vec![mon("Target", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "splash", "splash");
        assert_eq!(
            ally(&battle).ability.as_deref(),
            Some(if inherited == "intimidate" {
                inherited
            } else {
                ability
            })
        );
        if inherited == "intimidate" {
            assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Attack), -2);
        }
    }
}

#[test]
fn receiver_inherits_eligible_fainted_ally_abilities_and_activates_them() {
    check_ally_ability_inheritance("receiver");
}

#[test]
fn power_of_alchemy_inherits_eligible_fainted_ally_abilities_and_activates_them() {
    check_ally_ability_inheritance("power-of-alchemy");
}

#[test]
fn frisk_reveals_all_active_opponents_items_even_when_inactive() {
    let mut user = mon("User", &["splash"]);
    user.ability = Some("frisk".into());
    let mut target = mon("Target", &["splash"]);
    target.item = Some("sitrus-berry".into());
    target.ability = Some("klutz".into());
    let mut partner = mon("Partner", &["splash"]);
    partner.item = Some("power-herb".into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![target, partner]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn(&mut battle, "splash", "splash");
    for item in ["sitrus-berry", "power-herb"] {
        assert!(events.iter().any(|e| matches!(e, BattleEvent::Message(m) if m.contains("Frisk revealed") && m.contains(item))));
    }
    assert_eq!(foe(&battle).item.as_deref(), Some("sitrus-berry"));
}

#[test]
fn trace_copies_and_activates_eligible_foe_abilities_and_retries_later() {
    for copied in ["intimidate", "wonder-guard", "multitype"] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some("trace".into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(copied.into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "splash", "splash");
        assert_eq!(
            ally(&battle).ability.as_deref(),
            Some(if copied == "multitype" {
                "trace"
            } else {
                copied
            })
        );
        if copied == "intimidate" {
            assert_eq!(foe(&battle).stage(wilds_battle_engine::Stat::Attack), -1);
        }
    }
    let mut user = mon("User", &["splash"]);
    user.ability = Some("trace".into());
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("multitype".into());
    let mut bench = mon("Bench", &["splash"]);
    bench.ability = Some("huge-power".into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![target, bench]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).ability.as_deref(), Some("trace"));
    turn(&mut battle, "splash", "switch:1");
    assert_eq!(ally(&battle).ability.as_deref(), Some("huge-power"));
}

#[test]
fn anticipation_warns_about_weaknesses_ohko_and_hidden_power() {
    for (kind, move_id, expected) in [
        (PokemonType::Water, "thunderbolt", true),
        (PokemonType::Water, "thunder-wave", false),
        (PokemonType::Normal, "fissure", true),
        (PokemonType::Normal, "tackle", false),
        (PokemonType::Psychic, "hidden-power", true),
    ] {
        let mut user = mon("User", &["splash"]);
        user.types = vec![kind];
        user.ability = Some("anticipation".into());
        let target = mon("Target", &[move_id, "splash"]);
        let mut battle = game(user, target, 0.5);
        let events = turn(&mut battle, "splash", "splash");
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m.contains("Anticipation"))),
            expected
        );
    }
}

#[test]
fn analytic_boosts_only_after_every_other_pokemon_has_moved() {
    let run = |ability: &str, user_speed, partner| {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some(ability.into());
        user.speed = user_speed;
        let mut target = mon("Target", &["splash"]);
        target.speed = 100;
        let mut party = vec![user];
        if partner {
            let mut p = mon("Partner", &["splash"]);
            p.speed = 1;
            party.push(p);
        }
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [party, vec![target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut battle, "tackle", "splash");
        1000 - foe(&battle).hp
    };
    let base = run("", 10, false);
    assert_eq!(run("analytic", 200, false), base);
    assert_eq!(run("analytic", 10, true), base);
    assert!((f32::from(run("analytic", 10, false)) - f32::from(base) * 1.3).abs() <= 3.0);
}

#[test]
fn stakeout_boosts_attacks_only_on_the_targets_switch_in_turn() {
    for move_id in ["tackle", "water-gun"] {
        let run = |ability: &str| {
            let mut user = mon("User", &[move_id]);
            user.ability = Some(ability.into());
            let mut battle = Battle::with_rng(
                Fixed(0.5),
                [
                    vec![user],
                    vec![mon("Target", &["splash"]), mon("Bench", &["splash"])],
                ],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            battle.set_active([vec![0], vec![0]]).unwrap();
            turn(&mut battle, move_id, "switch:1");
            let first = 1000 - battle.participants(Side::Foes)[1].hp;
            turn(&mut battle, move_id, "splash");
            (first, 1000 - battle.participants(Side::Foes)[1].hp - first)
        };
        let base = run("");
        let boosted = run("stakeout");
        assert!((f32::from(boosted.0) - f32::from(base.0) * 2.0).abs() <= 3.0);
        assert_eq!(boosted.1, base.1);
    }
}

#[test]
fn unaware_ignores_opponents_relevant_stages_but_preserves_foul_play_source() {
    use wilds_battle_engine::Stat;
    let run =
        |move_id, attacking_ability: &str, defending_ability: &str, attack_stage, defense_stage| {
            let mut user = mon("User", &[move_id]);
            user.ability = Some(attacking_ability.into());
            user.stages.insert(Stat::Attack, attack_stage);
            user.stages.insert(Stat::Defense, attack_stage);
            let mut target = mon("Target", &["splash"]);
            target.ability = Some(defending_ability.into());
            target.stages.insert(Stat::Defense, defense_stage);
            target.stages.insert(Stat::Attack, attack_stage);
            let mut battle = game(user, target, 0.5);
            turn(&mut battle, move_id, "splash");
            1000 - foe(&battle).hp
        };
    for move_id in ["tackle", "body-press"] {
        let base = run(move_id, "", "", 0, 0);
        assert_eq!(run(move_id, "", "unaware", 4, 0), base);
        assert_eq!(run(move_id, "", "unaware", -4, 0), base);
        assert!(run(move_id, "mold-breaker", "unaware", 4, 0) > base);
        for stage in [-4, 4] {
            assert_eq!(run(move_id, "unaware", "", 0, stage), base);
        }
    }
    assert_eq!(
        run("foul-play", "", "unaware", 4, 0),
        run("foul-play", "", "", 4, 0)
    );
    for (attacker_ability, defender_ability, accuracy, evasion) in
        [("unaware", "", 0, 6), ("", "unaware", -6, 0)]
    {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some(attacker_ability.into());
        user.stages.insert(Stat::Accuracy, accuracy);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(defender_ability.into());
        target.stages.insert(Stat::Evasion, evasion);
        let mut battle = game(user, target, 0.75);
        turn(&mut battle, "tackle", "splash");
        assert!(foe(&battle).hp < 1000);
    }
}

#[test]
fn moody_raises_and_lowers_distinct_non_accuracy_stats_with_caps() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["splash"]);
    user.ability = Some("moody".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).stage(Stat::Attack), 2);
    assert_eq!(ally(&battle).stage(Stat::Defense), -1);
    assert_eq!(ally(&battle).stage(Stat::Accuracy), 0);
    assert_eq!(ally(&battle).stage(Stat::Evasion), 0);
    let mut user = mon("User", &["splash"]);
    user.ability = Some("moody".into());
    for stat in [
        Stat::Attack,
        Stat::Defense,
        Stat::SpAttack,
        Stat::SpDefense,
        Stat::Speed,
    ] {
        user.stages.insert(stat, 6);
    }
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "splash", "splash");
    assert_eq!(ally(&battle).stage(Stat::Attack), 5);
    assert_eq!(ally(&battle).stage(Stat::Defense), 6);
}

#[test]
fn slow_start_halves_attack_and_speed_for_five_turns() {
    let mut user = mon("User", &["tackle"]);
    user.speed = 150;
    user.ability = Some("slow-start".into());
    let mut target = mon("Target", &["splash"]);
    target.speed = 100;
    let mut battle = game(user, target, 0.5);
    let mut losses = vec![];
    for n in 0..6 {
        let before = foe(&battle).hp;
        let events = turn(&mut battle, "tackle", "splash");
        assert_eq!(first_move_is_user(&events), n == 5);
        losses.push(before - foe(&battle).hp);
    }
    assert!(losses[..5].iter().all(|x| *x == losses[0]));
    assert!((f32::from(losses[5]) - f32::from(losses[0]) * 2.0).abs() <= 3.0);
}

#[test]
fn truant_skips_alternate_actions_without_spending_pp_and_shares_recharge() {
    let mut user = mon("User", &["tackle"]);
    user.ability = Some("truant".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    for n in 0..4 {
        let before = foe(&battle).hp;
        let pp = ally(&battle).move_pp["tackle"];
        turn(&mut battle, "tackle", "splash");
        assert_eq!(foe(&battle).hp < before, n % 2 == 0);
        assert_eq!(ally(&battle).move_pp["tackle"], pp - u8::from(n % 2 == 0));
    }
    let mut user = mon("User", &["hyper_beam", "tackle"]);
    user.ability = Some("truant".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "hyper_beam", "splash");
    turn(&mut battle, "tackle", "splash");
    let before = foe(&battle).hp;
    turn(&mut battle, "tackle", "splash");
    assert!(foe(&battle).hp < before);
}

#[test]
fn stall_moves_last_within_its_priority_bracket_even_in_trick_room() {
    for (move_id, trick_room, first) in [
        ("tackle", false, false),
        ("tackle", true, false),
        ("quick-attack", false, true),
    ] {
        let mut user = mon("User", &[move_id, "trick-room"]);
        user.speed = 200;
        user.ability = Some("stall".into());
        let mut target = mon("Target", &["tackle", "splash"]);
        target.speed = 100;
        let mut battle = game(user, target, 0.5);
        if trick_room {
            turn(&mut battle, "trick-room", "splash");
        }
        assert_eq!(
            first_move_is_user(&turn(&mut battle, move_id, "tackle")),
            first
        );
    }
}

#[test]
fn quick_draw_acts_first_on_thirty_percent_of_attacks_within_priority() {
    for (move_id, opponent, dice, first) in [
        ("tackle", "tackle", 0.2, true),
        ("tackle", "tackle", 0.4, false),
        ("growl", "tackle", 0.2, false),
        ("tackle", "quick-attack", 0.2, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.speed = 10;
        user.ability = Some("quick-draw".into());
        let mut target = mon("Target", &[opponent]);
        target.speed = 100;
        let mut battle = game(user, target, dice);
        assert_eq!(
            first_move_is_user(&turn(&mut battle, move_id, opponent)),
            first
        );
    }
    let mut user = mon("User", &["tackle"]);
    user.speed = 200;
    user.ability = Some("quick-draw".into());
    let mut target = mon("Target", &["tackle"]);
    target.speed = 100;
    target.hp = 200;
    target.item = Some("custap-berry".into());
    let mut battle = game(user, target, 0.2);
    assert!(first_move_is_user(&turn(&mut battle, "tackle", "tackle")));
}

#[test]
fn gorilla_tactics_boosts_attack_locks_moves_and_forces_struggle_without_pp() {
    check_power_ability("gorilla-tactics", &["tackle"], "water-gun", 1.5, 1000);
    let mut user = mon("User", &["tackle", "splash"]);
    user.ability = Some("gorilla-tactics".into());
    let mut battle = game(user, mon("Target", &["splash", "gastro-acid"]), 0.5);
    turn(&mut battle, "tackle", "splash");
    let AdvanceStatus::Awaiting(prompt) = battle.advance().unwrap().status else {
        panic!("missing choice")
    };
    assert!(
        !prompt
            .choices
            .iter()
            .any(|c| matches!(c, Choice::UseMove { move_id, .. } if move_id == "splash"))
    );
    turn(&mut battle, "tackle", "gastro-acid");
    let AdvanceStatus::Awaiting(prompt) = battle.advance().unwrap().status else {
        panic!("missing choice")
    };
    assert!(
        prompt
            .choices
            .iter()
            .any(|c| matches!(c, Choice::UseMove { move_id, .. } if move_id == "splash"))
    );
    let mut user = mon("User", &["tackle", "splash"]);
    user.ability = Some("gorilla-tactics".into());
    user.move_pp.insert("tackle".into(), 1);
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "tackle", "splash");
    assert_eq!(ally(&battle).move_pp["tackle"], 0);
    let events = turn(&mut battle, "struggle", "splash");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m == "User used Struggle!"))
    );
}

#[test]
fn pressure_deducts_pp_per_opposing_target_with_field_and_hazard_exceptions() {
    for (id, cost) in [
        ("tackle", 2),
        ("surf", 3),
        ("splash", 1),
        ("spikes", 3),
        ("sticky-web", 1),
        ("imprison", 3),
        ("rain-dance", 3),
        ("perish-song", 3),
    ] {
        let mut user = mon("User", &[id]);
        user.ability = Some("mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("pressure".into());
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user], vec![target.clone(), target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        let initial = ally(&battle).move_pp[id];
        turn(&mut battle, id, "splash");
        assert_eq!(ally(&battle).move_pp[id], initial - cost, "{id}");
    }
    let mut user = mon("User", &["sleep-talk", "tackle"]);
    user.status = Some(Status::Asleep);
    user.status_turns = 3;
    let mut target = mon("Target", &["splash"]);
    target.ability = Some("pressure".into());
    let mut battle = game(user, target, 0.5);
    let initial = ally(&battle).move_pp["sleep-talk"];
    turn(&mut battle, "sleep-talk", "splash");
    assert_eq!(ally(&battle).move_pp["sleep-talk"], initial - 2);
}

#[test]
fn unburden_doubles_speed_after_losing_an_item_until_reacquired_or_suppressed() {
    for (item, hp, suppressed, first) in [
        (None, 1000, false, false),
        (Some("oran-berry"), 1000, false, false),
        (Some("oran-berry"), 400, false, true),
        (Some("oran-berry"), 400, true, false),
    ] {
        let mut user = mon("User", &["tackle"]);
        user.ability = Some("unburden".into());
        user.ability_suppressed = suppressed;
        user.item = item.map(str::to_owned);
        user.hp = hp;
        user.speed = 75;
        let mut target = mon("Target", &["tackle"]);
        target.speed = 100;
        let mut battle = game(user, target, 0.5);
        assert_eq!(
            first_move_is_user(&turn(&mut battle, "tackle", "tackle")),
            first
        );
    }
    let mut user = mon("User", &["tackle", "recycle"]);
    user.ability = Some("unburden".into());
    user.item = Some("oran-berry".into());
    user.hp = 500;
    user.speed = 75;
    let mut target = mon("Target", &["splash", "tackle"]);
    target.speed = 100;
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "recycle", "splash");
    assert!(ally(&battle).item.is_some());
    assert!(!first_move_is_user(&turn(&mut battle, "tackle", "tackle")));
}

#[test]
fn harvest_recovers_consumed_berries_with_half_chance_or_guaranteed_sun() {
    for (move_id, dice, cloud, restored) in [
        ("splash", 0.25, false, true),
        ("splash", 0.75, false, false),
        ("sunny_day", 0.75, false, true),
        ("sunny_day", 0.75, true, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.ability = Some("harvest".into());
        user.item = Some("sitrus-berry".into());
        user.hp = 500;
        let mut target = mon("Target", &["splash"]);
        target.ability = cloud.then(|| "cloud-nine".into());
        let mut battle = game(user, target, dice);
        turn(&mut battle, move_id, "splash");
        assert_eq!(ally(&battle).hp, 750);
        assert_eq!(ally(&battle).item.is_some(), restored);
    }
    let mut user = mon("User", &["splash"]);
    user.ability = Some("harvest".into());
    user.recyclable_item = Some("power-herb".into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.25);
    turn(&mut battle, "splash", "splash");
    assert!(ally(&battle).item.is_none());
}

#[test]
fn pickup_reclaims_a_neighbors_item_used_this_turn_only_once() {
    for (suppressed, picked_up) in [(false, true), (true, false)] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some("pickup".into());
        user.ability_suppressed = suppressed;
        let mut target = mon("Target", &["stuff-cheeks", "splash"]);
        target.item = Some("sitrus-berry".into());
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "splash", "stuff-cheeks");
        assert_eq!(ally(&battle).item.is_some(), picked_up);
        assert_eq!(foe(&battle).recyclable_item.is_none(), picked_up);
    }
    let mut user = mon("User", &["splash"]);
    user.ability = Some("pickup".into());
    let mut target = mon("Target", &["splash"]);
    target.recyclable_item = Some("sitrus-berry".into());
    let mut battle = game(user, target, 0.5);
    turn(&mut battle, "splash", "splash");
    assert!(ally(&battle).item.is_none());
}

fn check_changing_move_type(ability: &str) {
    let mut user = mon(
        "User",
        &["water-gun", "ember", "hidden-power", "future-sight"],
    );
    user.ability = Some(ability.into());
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "water-gun", "splash");
    assert_eq!(ally(&battle).types, [PokemonType::Water]);
    turn(&mut battle, "ember", "splash");
    assert_eq!(ally(&battle).types, [PokemonType::Fire]);
    turn(&mut battle, "hidden-power", "splash");
    assert_eq!(ally(&battle).types, [PokemonType::Dark]);
    turn(&mut battle, "future-sight", "splash");
    assert_eq!(ally(&battle).types, [PokemonType::Dark]);
    let mut user = mon("User", &["sleep-talk", "water-gun"]);
    user.ability = Some(ability.into());
    user.status = Some(Status::Asleep);
    user.status_turns = 3;
    let mut battle = game(user, mon("Target", &["splash"]), 0.5);
    turn(&mut battle, "sleep-talk", "splash");
    assert_eq!(ally(&battle).types, [PokemonType::Water]);
}

#[test]
fn protean_changes_type_on_each_move_under_generation_eight_rules() {
    check_changing_move_type("protean");
}

#[test]
fn libero_changes_type_on_each_move_under_generation_eight_rules() {
    check_changing_move_type("libero");
}

#[test]
fn imposter_transforms_on_entry_into_the_opposite_slot_and_respects_substitute() {
    for substitute in [false, true] {
        let mut user = mon("User", &["splash"]);
        user.ability = Some("imposter".into());
        user.hp = 600;
        let target = mon("Near", &["ember"]);
        let mut opposite = mon("Across", &["water-gun"]);
        opposite.attack = 321;
        opposite.ability = Some("water-absorb".into());
        opposite.substitute_hp = if substitute { 100 } else { 0 };
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, mon("Ally", &["splash"])], vec![target, opposite]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        battle.advance().unwrap();
        assert_eq!(ally(&battle).transformed(), !substitute);
        assert_eq!(ally(&battle).hp, 600);
        if !substitute {
            assert_eq!(ally(&battle).attack, 321);
            assert_eq!(ally(&battle).moves, ["water-gun"]);
            assert_eq!(ally(&battle).move_pp["water-gun"], 5);
            assert_eq!(ally(&battle).ability.as_deref(), Some("water-absorb"));
        }
    }
}

#[test]
fn color_change_waits_until_all_hits_finish_and_substitutes_block_it() {
    let run = |ability: &str, substitute| {
        let user = mon("User", &["double_kick"]);
        let mut target = mon("Target", &["splash"]);
        target.ability = Some(ability.into());
        target.substitute_hp = substitute;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "double_kick", "splash");
        (foe(&battle).hp, foe(&battle).types.clone())
    };
    let base = run("", 0);
    let changed = run("color-change", 0);
    assert_eq!(base.0, changed.0);
    assert_eq!(changed.1, [PokemonType::Fighting]);
    assert_eq!(run("color-change", 999).1, [PokemonType::Normal]);
}

#[test]
fn pickpocket_steals_on_contact_after_the_move_with_existing_item_restrictions() {
    for (move_id, attacker_ability, substitute, stolen) in [
        ("tackle", "", 0, true),
        ("water-gun", "", 0, false),
        ("tackle", "long-reach", 0, false),
        ("tackle", "sticky-hold", 0, false),
        ("tackle", "", 999, false),
    ] {
        let mut user = mon("User", &[move_id]);
        user.item = Some("power-herb".into());
        user.ability = Some(attacker_ability.into());
        let mut target = mon("Target", &["splash"]);
        target.ability = Some("pickpocket".into());
        target.substitute_hp = substitute;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, move_id, "splash");
        assert_eq!(foe(&battle).item.is_some(), stolen);
        assert_eq!(ally(&battle).item.is_none(), stolen);
    }
}

#[test]
fn magician_steals_after_damage_but_not_with_fling_gems_or_substitutes() {
    for (id, item, substitute, stolen) in [
        ("water-gun", None, 0, true),
        ("water-gun", Some("water-gem"), 0, false),
        ("fling", Some("oran-berry"), 0, false),
        ("natural-gift", Some("oran-berry"), 0, true),
        ("water-gun", None, 999, false),
    ] {
        let mut user = mon("User", &[id]);
        user.ability = Some("magician".into());
        user.item = item.map(str::to_owned);
        let mut target = mon("Target", &["splash"]);
        target.item = Some("power-herb".into());
        target.substitute_hp = substitute;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, id, "splash");
        assert_eq!(
            ally(&battle).item.as_deref() == Some("power-herb"),
            stolen,
            "{id}"
        );
    }
    let mut user = mon("User", &["surf"]);
    user.ability = Some("magician".into());
    let mut slow = mon("Slow", &["splash"]);
    slow.item = Some("power-herb".into());
    slow.speed = 10;
    let mut fast = mon("Fast", &["splash"]);
    fast.item = Some("sitrus-berry".into());
    fast.speed = 100;
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![slow, fast]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle, "surf", "splash");
    assert_eq!(ally(&battle).item.as_deref(), Some("sitrus-berry"));
}

#[test]
fn symbiosis_transfers_a_neighbors_item_after_consumption_but_not_removal() {
    for (consume, suppressed, transferred) in [
        (true, false, true),
        (true, true, false),
        (false, false, false),
    ] {
        let mut user = mon("User", &["stuff-cheeks", "splash"]);
        user.item = Some("oran-berry".into());
        let mut donor = mon("Donor", &["splash"]);
        donor.item = Some("power-herb".into());
        donor.ability = Some("symbiosis".into());
        donor.ability_suppressed = suppressed;
        let target = mon("Target", &["splash", "knock-off"]);
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, donor], vec![target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(
            &mut battle,
            if consume { "stuff-cheeks" } else { "splash" },
            if consume { "splash" } else { "knock-off" },
        );
        assert_eq!(
            ally(&battle).item.as_deref() == Some("power-herb"),
            transferred
        );
        assert_eq!(
            battle.participants(Side::Allies)[1].item.is_none(),
            transferred
        );
    }
}

fn add_form(p: &mut Pokemon, name: &str, kind: PokemonType) {
    let mut form = wilds_battle_engine::PokemonForm::of(p);
    form.types = vec![kind];
    p.forms.insert(name.into(), form);
}

#[test]
fn forecast_tracks_weather_and_suppression_using_host_supplied_forms() {
    let mut user = mon("User", &["sunny_day", "rain-dance", "hail", "splash"]);
    user.species_id = 351;
    user.ability = Some("forecast".into());
    for (form, kind) in [
        ("castform-sunny", PokemonType::Fire),
        ("castform-rainy", PokemonType::Water),
        ("castform-snowy", PokemonType::Ice),
    ] {
        add_form(&mut user, form, kind);
    }
    let mut battle = game(user, mon("Target", &["splash", "gastro-acid"]), 0.5);
    for (move_id, kind) in [
        ("sunny_day", PokemonType::Fire),
        ("rain-dance", PokemonType::Water),
        ("hail", PokemonType::Ice),
    ] {
        turn(&mut battle, move_id, "splash");
        assert_eq!(ally(&battle).types, [kind]);
    }
    turn(&mut battle, "splash", "gastro-acid");
    assert_eq!(ally(&battle).types, [PokemonType::Normal]);
    assert!(ally(&battle).form.is_none());
}

#[test]
fn flower_gift_changes_cherrim_and_boosts_allied_attack_and_special_defense_in_sun() {
    let run = |sun, suppressed, breaker: bool| {
        let user = mon("User", &["sunny_day", "splash", "tackle"]);
        let mut cherrim = mon("Cherrim", &["splash"]);
        cherrim.species_id = 421;
        cherrim.ability = Some("flower-gift".into());
        cherrim.ability_suppressed = suppressed;
        add_form(&mut cherrim, "cherrim-sunshine", PokemonType::Grass);
        let mut target = mon("Target", &["splash", "water-gun"]);
        target.ability = breaker.then(|| "mold-breaker".into());
        let mut battle = Battle::with_rng(
            Fixed(0.5),
            [vec![user, cherrim], vec![target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(
            &mut battle,
            if sun { "sunny_day" } else { "splash" },
            "splash",
        );
        assert_eq!(
            battle.participants(Side::Allies)[1].form.is_some(),
            sun && !suppressed
        );
        turn(&mut battle, "tackle", "water-gun");
        (1000 - foe(&battle).hp, 1000 - ally(&battle).hp)
    };
    let base = run(true, true, false);
    let boosted = run(true, false, false);
    assert!((f32::from(boosted.0) - f32::from(base.0) * 1.5).abs() <= 3.0);
    assert!((f32::from(boosted.1) * 1.5 - f32::from(base.1)).abs() <= 3.0);
    assert_eq!(run(true, false, true).1, base.1);
    assert_eq!(run(false, false, false), run(false, true, false));
}

#[test]
fn zen_mode_changes_at_turn_end_at_half_hp_and_supports_galar_forms() {
    for galar in [false, true] {
        let mut user = mon("User", &["splash", "recover"]);
        user.species_id = 555;
        user.ability = Some("zen-mode".into());
        user.hp = 500;
        let (base, zen) = if galar {
            ("darmanitan-galar-standard", "darmanitan-galar-zen")
        } else {
            ("darmanitan-standard", "darmanitan-zen")
        };
        user.form = Some(base.into());
        add_form(&mut user, zen, PokemonType::Fire);
        user.forms.get_mut(zen).unwrap().sp_attack = 321;
        let old_spa = user.sp_attack;
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        battle.advance().unwrap();
        assert_eq!(ally(&battle).form.as_deref(), Some(base));
        turn(&mut battle, "splash", "splash");
        assert_eq!(ally(&battle).form.as_deref(), Some(zen));
        assert_eq!(ally(&battle).sp_attack, 321);
        turn(&mut battle, "recover", "splash");
        assert_eq!(ally(&battle).form.as_deref(), Some(base));
        assert_eq!(ally(&battle).sp_attack, old_spa);
    }
}
