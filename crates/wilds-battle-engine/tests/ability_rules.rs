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
