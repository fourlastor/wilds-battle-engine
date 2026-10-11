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
