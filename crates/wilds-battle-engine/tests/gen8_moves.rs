//! Behavior regressions for the additional Gen 1–8 moves outside the local JSON catalog.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, Pokemon,
    PokemonType, Prompt, Side,
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

fn game(user: Pokemon, foes: Vec<Pokemon>, dice: f32) -> Battle {
    Battle::with_rng(
        Fixed(dice),
        [vec![user], foes],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap()
}

fn turn_with(battle: &mut Battle, mut select: impl FnMut(&Prompt) -> u32) -> Vec<BattleEvent> {
    let start = battle.turn();
    let mut events = vec![];
    for _ in 0..30 {
        let result = battle.advance().unwrap();
        events.extend(result.events);
        if battle.turn() > start
            && matches!(&result.status, AdvanceStatus::Awaiting(p) if p.reason == wilds_battle_engine::PromptReason::Action)
        {
            return events;
        }
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            return events;
        };
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: select(&prompt),
            })
            .unwrap();
    }
    panic!("turn did not advance");
}

fn turn(battle: &mut Battle) -> Vec<BattleEvent> {
    turn_with(battle, |prompt| {
        prompt
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap()
            .id()
    })
}

#[test]
fn whirlwind_phazes_through_protect_and_substitute_but_respects_anchoring() {
    for (ability, switched) in [(None, true), (Some("suction-cups"), false)] {
        let mut foe = mon("Foe", &["protect"]);
        foe.substitute_hp = 50;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(
            mon("User", &["whirlwind"]),
            vec![foe, mon("Bench", &["splash"])],
            0.5,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        assert_eq!(b.active_indices(Side::Foes), &[usize::from(switched)]);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
    let mut b = game(
        mon("User", &["whirlwind"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

fn check_binding(move_id: &str, immune: PokemonType) {
    for (kind, response, sub, binds) in [
        (PokemonType::Normal, "splash", 0, true),
        (immune, "splash", 0, false),
        (PokemonType::Normal, "protect", 0, false),
        (PokemonType::Normal, "splash", 500, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &[move_id]), vec![foe], 0.5);
        turn(&mut b);
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(foe.bound.is_some(), binds);
        if binds {
            assert!((2..=4).contains(&foe.bound.as_ref().unwrap().turns_left));
            assert!(foe.hp < 875);
        }
    }
}

#[test]
fn wrap_binds_for_four_to_five_turns_only_after_reaching_the_target() {
    check_binding("wrap", PokemonType::Ghost);
}

#[test]
fn roar_bypasses_protect_and_substitute_but_soundproof_blocks_it() {
    for (ability, switched) in [
        (None, true),
        (Some("soundproof"), false),
        (Some("suction-cups"), false),
    ] {
        let mut foe = mon("Foe", &["protect"]);
        foe.substitute_hp = 50;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(
            mon("User", &["roar"]),
            vec![foe, mon("Bench", &["splash"])],
            0.5,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        assert_eq!(b.active_indices(Side::Foes), &[usize::from(switched)]);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
}

#[test]
fn mist_prevents_opposing_stat_drops_for_five_turns_without_refreshing() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["mist", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["growl"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for _ in 0..3 {
        turn_with(&mut b, |p| {
            p.choices.iter().find(|c| matches!(c, Choice::UseMove {move_id,..} if move_id == if p.actor.side == Side::Allies {"splash"} else {"growl"})).unwrap().id()
        });
    }
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    assert!(b.side_conditions(Side::Allies).is_empty());
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c| matches!(c, Choice::UseMove {move_id,..} if move_id == if p.actor.side == Side::Allies {"splash"} else {"growl"})).unwrap().id()
    });
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), -1);
}

fn control_damage(
    power: u16,
    kind: &str,
    category: &str,
    mut user: Pokemon,
    mut foe: Pokemon,
) -> u16 {
    let source = format!(
        "return {{{{id='control', name='Control', type=Type.{kind}, category=Category.{category}, pp=40, accuracy=Accuracy.Always, script=function(ctx) ctx:damage({power}) end}}, {{id='wait', name='Wait', type=Type.Normal, category=Category.Status, pp=40, target=Target.User, script=function(_) end}}}}"
    );
    user.moves = vec!["control".into()];
    foe.moves = vec!["wait".into()];
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![foe]],
        MoveCatalog::from_lua(&source).unwrap(),
    )
    .unwrap();
    turn(&mut b);
    1000 - b.participants(Side::Foes)[0].hp
}

#[test]
fn low_kick_obeys_all_six_weight_brackets() {
    for (weight, power) in [
        (0.1, 20),
        (9.9, 20),
        (10.0, 40),
        (24.9, 40),
        (25.0, 60),
        (49.9, 60),
        (50.0, 80),
        (99.9, 80),
        (100.0, 100),
        (199.9, 100),
        (200.0, 120),
    ] {
        let user = mon("User", &["low-kick"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.weight = weight;
        let expected = control_damage(power, "Fighting", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "weight {weight}"
        );
    }
}

#[test]
fn leech_seed_drains_one_eighth_and_heals_its_source_but_respects_grass_and_substitute() {
    for (kind, sub, response, seeded) in [
        (PokemonType::Normal, 0, "splash", true),
        (PokemonType::Grass, 0, "splash", false),
        (PokemonType::Normal, 50, "splash", false),
        (PokemonType::Normal, 0, "protect", false),
    ] {
        let mut user = mon("User", &["leech-seed"]);
        user.hp = 500;
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if seeded { 875 } else { 1000 }
        );
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if seeded { 625 } else { 500 }
        );
        if seeded {
            turn(&mut b);
            assert!(b.participants(Side::Allies)[0].last_move_failed);
            assert_eq!(b.participants(Side::Foes)[0].hp, 750);
        }
    }
}

fn check_partial_trap(move_id: &str, power: u16, kind: &str, category: &str) {
    for (response, sub, dice, binds) in [
        ("splash", 0, 0.5, true),
        ("protect", 0, 0.5, false),
        ("splash", 500, 0.5, false),
    ] {
        let user = mon("User", &[move_id]);
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let expected = control_damage(power, kind, category, user.clone(), foe.clone());
        let mut b = game(user, vec![foe], dice);
        turn(&mut b);
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(foe.bound.is_some(), binds, "{move_id}");
        if binds {
            assert_eq!(1000 - foe.hp, expected + 125, "{move_id}");
            assert!((2..=4).contains(&foe.bound.as_ref().unwrap().turns_left));
        } else {
            assert_eq!(foe.hp, 1000);
        }
    }
}

#[test]
fn fire_spin_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("fire-spin", 35, "Fire", "Special");
}

#[test]
fn clamp_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("clamp", 35, "Water", "Physical");
}

#[test]
fn sand_tomb_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("sand-tomb", 35, "Ground", "Physical");
}

#[test]
fn magma_storm_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("magma-storm", 100, "Fire", "Special");
}

#[test]
fn infestation_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("infestation", 20, "Bug", "Special");
}

#[test]
fn snap_trap_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("snap-trap", 35, "Grass", "Physical");
}

#[test]
fn thunder_cage_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("thunder-cage", 80, "Electric", "Special");
}

#[test]
fn teleport_requests_a_replacement_after_attacks_and_fails_without_a_bench() {
    let user = mon("User", &["teleport"]);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![user.clone(), mon("Bench", &["splash"])],
            vec![mon("Foe", &["tackle"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap_or(&p.choices[0])
            .id()
    });
    assert_eq!(b.active_indices(Side::Allies), &[1]);
    assert!(b.participants(Side::Allies)[0].hp < 1000);
    assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

fn check_screen(move_id: &str, condition: &str, reduced: &str, unchanged: &str) {
    for (attack, weakens) in [(reduced, true), (unchanged, false)] {
        let mut user = mon("User", &[move_id]);
        user.speed = 200;
        let foe = mon("Foe", &[attack]);
        let mut baseline = game(mon("Control", &["splash"]), vec![foe.clone()], 0.5);
        turn(&mut baseline);
        let normal = 1000 - baseline.participants(Side::Allies)[0].hp;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let actual = 1000 - b.participants(Side::Allies)[0].hp;
        if weakens {
            assert!(actual > 0 && actual < normal);
        } else {
            assert_eq!(actual, normal);
        }
        assert_eq!(
            b.side_conditions(Side::Allies)
                .iter()
                .find(|c| c.name == condition)
                .unwrap()
                .turns_left,
            Some(4)
        );
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
        assert_eq!(
            b.side_conditions(Side::Allies)
                .iter()
                .find(|c| c.name == condition)
                .unwrap()
                .turns_left,
            Some(3)
        );
    }
}

#[test]
fn light_screen_reduces_its_damage_category_without_refreshing() {
    check_screen("light-screen", "light_screen", "water-gun", "tackle");
}

#[test]
fn reflect_reduces_its_damage_category_without_refreshing() {
    check_screen("reflect", "reflect", "tackle", "water-gun");
}

#[test]
fn haze_clears_both_sides_through_protect_and_substitute() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["haze"]);
    user.stages.insert(Stat::Attack, -3);
    let mut foe = mon("Foe", &["protect"]);
    foe.stages.insert(Stat::SpDefense, 6);
    foe.substitute_hp = 50;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::SpDefense), 0);
    assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 50);
}

#[test]
fn flail_uses_the_current_hp_brackets() {
    for (hp, power) in [
        (1, 200),
        (41, 200),
        (42, 150),
        (104, 150),
        (105, 100),
        (208, 100),
        (209, 80),
        (354, 80),
        (355, 40),
        (687, 40),
        (688, 20),
        (1000, 20),
    ] {
        let mut user = mon("User", &["flail"]);
        user.hp = hp;
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Normal", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected, "HP {hp}");
    }
}

#[test]
fn reversal_uses_the_current_hp_brackets() {
    for (hp, power) in [
        (1, 200),
        (41, 200),
        (42, 150),
        (104, 150),
        (105, 100),
        (208, 100),
        (209, 80),
        (354, 80),
        (355, 40),
        (687, 40),
        (688, 20),
        (1000, 20),
    ] {
        let mut user = mon("User", &["reversal"]);
        user.hp = hp;
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Fighting", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected, "HP {hp}");
    }
}

#[test]
fn return_uses_host_supplied_happiness() {
    for (happiness, power) in [
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (5, 2),
        (100, 40),
        (254, 101),
        (255, 102),
    ] {
        let mut user = mon("User", &["return"]);
        user.happiness = Some(happiness);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Normal", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "happiness {happiness}"
        );
    }
}

#[test]
fn frustration_uses_host_supplied_happiness() {
    for (happiness, power) in [
        (255, 1),
        (254, 1),
        (253, 1),
        (252, 1),
        (250, 2),
        (155, 40),
        (1, 101),
        (0, 102),
    ] {
        let mut user = mon("User", &["frustration"]);
        user.happiness = Some(happiness);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Normal", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "happiness {happiness}"
        );
    }
}

#[test]
fn pika_papow_uses_host_supplied_happiness() {
    for (happiness, power) in [
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (5, 2),
        (100, 40),
        (254, 101),
        (255, 102),
    ] {
        let mut user = mon("User", &["pika-papow"]);
        user.happiness = Some(happiness);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Electric", "Special", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "happiness {happiness}"
        );
    }
}

#[test]
fn veevee_volley_uses_host_supplied_happiness() {
    for (happiness, power) in [
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (5, 2),
        (100, 40),
        (254, 101),
        (255, 102),
    ] {
        let mut user = mon("User", &["veevee-volley"]);
        user.happiness = Some(happiness);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Normal", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "happiness {happiness}"
        );
    }
}

#[test]
fn grass_knot_obeys_all_weight_brackets() {
    for (weight, power) in [
        (9.9, 20),
        (10.0, 40),
        (24.9, 40),
        (25.0, 60),
        (49.9, 60),
        (50.0, 80),
        (99.9, 80),
        (100.0, 100),
        (199.9, 100),
        (200.0, 120),
    ] {
        let user = mon("User", &["grass-knot"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.weight = weight;
        let expected = control_damage(power, "Grass", "Special", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "weight {weight}"
        );
    }
}

#[test]
fn heavy_slam_uses_relative_weight() {
    for (weight, power) in [
        (19.9, 40),
        (20.0, 60),
        (29.9, 60),
        (30.0, 80),
        (39.9, 80),
        (40.0, 100),
        (49.9, 100),
        (50.0, 120),
    ] {
        let mut user = mon("User", &["heavy-slam"]);
        user.weight = weight;
        let mut foe = mon("Foe", &["splash"]);
        foe.weight = 10.0;
        let expected = control_damage(power, "Steel", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "weight {weight}"
        );
    }
    let mut user = mon("User", &["heavy-slam"]);
    user.weight = 50.0;
    user.speed = 1;
    let mut foe = mon("Foe", &["minimize"]);
    foe.weight = 10.0;
    let expected = control_damage(240, "Steel", "Physical", user.clone(), foe.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
}

#[test]
fn heat_crash_uses_relative_weight() {
    for (weight, power) in [
        (19.9, 40),
        (20.0, 60),
        (29.9, 60),
        (30.0, 80),
        (39.9, 80),
        (40.0, 100),
        (49.9, 100),
        (50.0, 120),
    ] {
        let mut user = mon("User", &["heat-crash"]);
        user.weight = weight;
        let mut foe = mon("Foe", &["splash"]);
        foe.weight = 10.0;
        let expected = control_damage(power, "Fire", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "weight {weight}"
        );
    }
    let mut user = mon("User", &["heat-crash"]);
    user.weight = 50.0;
    user.speed = 1;
    let mut foe = mon("Foe", &["minimize"]);
    foe.weight = 10.0;
    let expected = control_damage(240, "Fire", "Physical", user.clone(), foe.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
}

#[test]
fn gyro_ball_uses_modified_speed_and_caps_at_150_power() {
    for (speed, power) in [
        (1, 150),
        (16, 150),
        (25, 101),
        (50, 51),
        (100, 26),
        (200, 13),
        (500, 6),
    ] {
        let mut user = mon("User", &["gyro-ball"]);
        user.speed = speed;
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Steel", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "speed {speed}"
        );
    }
}

#[test]
fn electro_ball_obeys_all_speed_ratio_thresholds() {
    for (speed, power) in [
        (99, 40),
        (100, 60),
        (199, 60),
        (200, 80),
        (299, 80),
        (300, 120),
        (399, 120),
        (400, 150),
        (1000, 150),
    ] {
        let mut user = mon("User", &["electro-ball"]);
        user.speed = speed;
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Electric", "Special", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "speed {speed}"
        );
    }
}

#[test]
fn wring_out_uses_targets_current_hp_with_fixed_point_rounding() {
    for (hp, power) in [
        (1000, 120),
        (999, 119),
        (500, 60),
        (250, 30),
        (9, 1),
        (1, 1),
    ] {
        let user = mon("User", &["wring-out"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.hp = hp;
        let expected = control_damage(power, "Normal", "Special", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected, "HP {hp}");
    }
}

#[test]
fn crush_grip_uses_targets_current_hp_with_fixed_point_rounding() {
    for (hp, power) in [
        (1000, 120),
        (999, 119),
        (500, 60),
        (250, 30),
        (9, 1),
        (1, 1),
    ] {
        let user = mon("User", &["crush-grip"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.hp = hp;
        let expected = control_damage(power, "Normal", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected, "HP {hp}");
    }
}

#[test]
fn trump_card_uses_pp_remaining_after_spending_this_use() {
    for (pp, power) in [(5, 40), (4, 50), (3, 60), (2, 80), (1, 200)] {
        let mut user = mon("User", &["trump-card"]);
        user.move_pp.insert("trump-card".into(), pp);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Normal", "Special", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected, "PP {pp}");
        assert_eq!(
            b.participants(Side::Allies)[0].move_pp["trump-card"],
            pp - 1
        );
    }
}

#[test]
fn punishment_counts_positive_stages_only_and_caps_power() {
    use wilds_battle_engine::Stat;
    for (attack, speed, power) in [(0, 0, 60), (2, -6, 100), (6, 1, 200), (6, 6, 200)] {
        let user = mon("User", &["punishment"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.stages.insert(Stat::Attack, attack);
        foe.stages.insert(Stat::Speed, speed);
        let expected = control_damage(power, "Dark", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
    }
}

#[test]
fn super_fang_halves_current_hp_with_a_minimum_of_one() {
    for (hp, sub, response, expected) in [
        (1000, 0, "splash", 500),
        (999, 0, "splash", 500),
        (1, 0, "splash", 0),
        (1000, 600, "splash", 1000),
        (1000, 0, "protect", 1000),
    ] {
        let user = mon("User", &["super-fang"]);
        let mut foe = mon("Foe", &[response]);
        foe.hp = hp;
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].hp, expected);
        if sub > 0 {
            assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 100);
        }
    }
}

#[test]
fn natures_madness_halves_current_hp_with_a_minimum_of_one() {
    for (hp, sub, response, expected) in [
        (1000, 0, "splash", 500),
        (999, 0, "splash", 500),
        (1, 0, "splash", 0),
        (1000, 600, "splash", 1000),
        (1000, 0, "protect", 1000),
    ] {
        let user = mon("User", &["natures-madness"]);
        let mut foe = mon("Foe", &[response]);
        foe.hp = hp;
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].hp, expected);
        if sub > 0 {
            assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 100);
        }
    }
}

struct Rolls(u64, f32);
impl BattleRng for Rolls {
    fn next_u64(&mut self) -> u64 {
        self.0
    }
    fn fraction(&mut self) -> f32 {
        self.1
    }
}
#[test]
fn psywave_scales_a_uniform_fifty_to_150_percent_by_level() {
    for (roll, level, damage) in [(0, 50, 25), (50, 50, 50), (100, 50, 75), (0, 1, 1)] {
        let mut user = mon("User", &["psywave"]);
        user.level = level;
        let mut b = Battle::with_rng(
            Rolls(roll, 0.5),
            [vec![user], vec![mon("Foe", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000 - damage);
    }
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Dark];
    let mut b = game(mon("User", &["psywave"]), vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
}

#[test]
fn tri_attack_chooses_one_of_three_statuses_and_respects_chance_and_substitute() {
    use wilds_battle_engine::Status;
    for (roll, status) in [
        (0, Status::Burned),
        (1, Status::Paralyzed),
        (2, Status::Frozen),
    ] {
        let mut user = mon("User", &["tri-attack"]);
        user.speed = 1;
        let mut b = Battle::with_rng(
            Rolls(roll, 0.1),
            [vec![user], vec![mon("Foe", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].status, Some(status));
    }
    for (sub, dice) in [(0, 0.5), (500, 0.1)] {
        let mut foe = mon("Foe", &["splash"]);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["tri-attack"]), vec![foe], dice);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].status, None);
    }
}

#[test]
fn endeavor_matches_users_hp_only_when_target_has_more_and_can_be_hit() {
    for (hp, kind, sub, expected) in [
        (300, PokemonType::Normal, 0, 300),
        (1000, PokemonType::Normal, 0, 1000),
        (300, PokemonType::Ghost, 0, 1000),
        (300, PokemonType::Normal, 800, 1000),
    ] {
        let mut user = mon("User", &["endeavor"]);
        user.hp = hp;
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].hp, expected);
        if sub > 0 {
            assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 100);
        }
    }
}

#[test]
fn final_gambit_sacrifices_user_only_on_a_successful_hit_including_substitute() {
    for (kind, response, sub, hit) in [
        (PokemonType::Normal, "splash", 0, true),
        (PokemonType::Normal, "splash", 500, true),
        (PokemonType::Ghost, "splash", 0, false),
        (PokemonType::Normal, "protect", 0, false),
    ] {
        let mut user = mon("User", &["final-gambit"]);
        user.hp = 300;
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if hit { 0 } else { 300 }
        );
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if hit && sub == 0 { 700 } else { 1000 }
        );
    }
}

fn check_source_trap(move_id: &str, ignores_protect: bool) {
    use wilds_battle_engine::ParticipantId;
    for (response, sub, ghost, expected) in [
        ("splash", 0, false, true),
        ("protect", 0, false, ignores_protect),
        ("splash", 50, false, false),
        ("splash", 0, true, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        if ghost {
            foe.types = vec![PokemonType::Ghost];
        }
        let mut b = Battle::with_rng(
            Fixed(0.5),
            [
                vec![mon("User", &[move_id]), mon("Bench", &["splash"])],
                vec![foe, mon("Foe bench", &["splash"])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        let target = ParticipantId {
            side: Side::Foes,
            index: 0,
        };
        assert_eq!(
            b.trapped(target),
            expected,
            "{move_id} {response} {sub} {ghost}"
        );
        if expected {
            turn_with(&mut b, |p| {
                p.choices
                    .iter()
                    .find(|c| {
                        if p.actor.side == Side::Allies {
                            matches!(c, Choice::Switch { .. })
                        } else {
                            matches!(c, Choice::UseMove { .. })
                        }
                    })
                    .unwrap()
                    .id()
            });
            assert!(!b.trapped(target));
        }
    }
}

#[test]
fn spider_web_traps_until_the_source_leaves() {
    check_source_trap("spider-web", false);
}

#[test]
fn mean_look_traps_until_the_source_leaves() {
    check_source_trap("mean-look", true);
}

#[test]
fn block_traps_until_the_source_leaves() {
    check_source_trap("block", true);
}

#[test]
fn ingrain_heals_each_turn_and_cannot_stack() {
    let mut user = mon("User", &["ingrain"]);
    user.hp = 500;
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 562);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 624);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert!(b.trapped(wilds_battle_engine::ParticipantId {
        side: Side::Allies,
        index: 0
    }));
}

#[test]
fn aqua_ring_heals_each_turn_and_cannot_stack() {
    let mut user = mon("User", &["aqua-ring"]);
    user.hp = 500;
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 562);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 624);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn safeguard_blocks_external_status_but_allows_rest() {
    use wilds_battle_engine::Status;
    let mut user = mon("User", &["safeguard", "rest"]);
    user.speed = 200;
    user.hp = 500;
    let mut b = game(user, vec![mon("Foe", &["toxic"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].status, None);
    assert_eq!(b.side_conditions(Side::Allies)[0].turns_left, Some(4));
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c| matches!(c,Choice::UseMove{move_id,..} if move_id==if p.actor.side==Side::Allies {"rest"} else {"toxic"})).unwrap().id()
    });
    assert_eq!(b.participants(Side::Allies)[0].status, Some(Status::Asleep));
}

#[test]
fn tailwind_doubles_team_speed_for_four_turns_without_refreshing() {
    let mut user = mon("User", &["tailwind", "tackle"]);
    user.speed = 80;
    let mut foe = mon("Foe", &["tackle"]);
    foe.speed = 120;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.side_conditions(Side::Allies)[0].turns_left, Some(3));
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.side_conditions(Side::Allies)[0].turns_left, Some(2));
    let events = turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| matches!(c,Choice::UseMove{move_id,..}if move_id=="tackle"))
            .unwrap()
            .id()
    });
    let first = events
        .iter()
        .find_map(|e| match e {
            BattleEvent::Damage { target, .. } => Some(target.side),
            _ => None,
        })
        .unwrap();
    assert_eq!(first, Side::Foes);
}

#[test]
fn fairy_lock_traps_both_sides_for_the_next_turn_only() {
    use wilds_battle_engine::ParticipantId;
    let mut b = game(
        mon("User", &["fairy-lock", "splash"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    for side in [Side::Allies, Side::Foes] {
        assert!(b.trapped(ParticipantId { side, index: 0 }));
    }
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| matches!(c,Choice::UseMove{move_id,..}if move_id=="splash"))
            .unwrap()
            .id()
    });
    for side in [Side::Allies, Side::Foes] {
        assert!(!b.trapped(ParticipantId { side, index: 0 }));
    }
}

#[test]
fn rain_dance_sets_five_turn_weather_and_cannot_refresh_it() {
    let mut b = game(
        mon("User", &["rain-dance"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert_eq!(
        b.weather().unwrap().kind,
        wilds_battle_engine::WeatherKind::Rain
    );
    assert_eq!(b.weather().unwrap().turns_left, 4);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.weather().unwrap().turns_left, 3);
}

#[test]
fn hail_sets_five_turn_weather_and_cannot_refresh_it() {
    let mut b = game(mon("User", &["hail"]), vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert_eq!(
        b.weather().unwrap().kind,
        wilds_battle_engine::WeatherKind::Hail
    );
    assert_eq!(b.weather().unwrap().turns_left, 4);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.weather().unwrap().turns_left, 3);
}

#[test]
fn flatter_raises_special_attack_and_confuses_but_respects_protection() {
    use wilds_battle_engine::Stat;
    for (response, sub, lands) in [
        ("splash", 0, true),
        ("protect", 0, false),
        ("splash", 50, false),
    ] {
        let mut user = mon("User", &["flatter"]);
        user.speed = 1;
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].stage(Stat::SpAttack),
            i8::from(lands)
        );
        assert_eq!(
            b.participants(Side::Foes)[0].confused_turns.is_some(),
            lands
        );
    }
}

#[test]
fn shell_smash_changes_five_stats_through_contrary_and_simple() {
    use wilds_battle_engine::Stat;
    for (ability, multiplier) in [(None, 1), (Some("contrary"), -1), (Some("simple"), 2)] {
        let mut user = mon("User", &["shell-smash"]);
        user.ability = ability.map(str::to_owned);
        let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut b);
        let p = &b.participants(Side::Allies)[0];
        for stat in [Stat::Attack, Stat::SpAttack, Stat::Speed] {
            assert_eq!(p.stage(stat), 2 * multiplier);
        }
        for stat in [Stat::Defense, Stat::SpDefense] {
            assert_eq!(p.stage(stat), -multiplier);
        }
    }
}

#[test]
fn power_swap_swaps_stages_through_substitute_without_ability_scaling() {
    use wilds_battle_engine::Stat;
    let stats = [Stat::Attack, Stat::SpAttack];
    for (response, success) in [("splash", true), ("protect", false)] {
        let mut user = mon("User", &["power-swap"]);
        user.ability = Some("contrary".into());
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = 50;
        foe.ability = Some("simple".into());
        for stat in stats {
            user.stages.insert(stat, 3);
            foe.stages.insert(stat, -2);
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        for stat in stats {
            assert_eq!(
                b.participants(Side::Allies)[0].stage(stat),
                if success { -2 } else { 3 }
            );
            assert_eq!(
                b.participants(Side::Foes)[0].stage(stat),
                if success { 3 } else { -2 }
            );
        }
    }
}

#[test]
fn guard_swap_swaps_stages_through_substitute_without_ability_scaling() {
    use wilds_battle_engine::Stat;
    let stats = [Stat::Defense, Stat::SpDefense];
    for (response, success) in [("splash", true), ("protect", false)] {
        let mut user = mon("User", &["guard-swap"]);
        user.ability = Some("contrary".into());
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = 50;
        foe.ability = Some("simple".into());
        for stat in stats {
            user.stages.insert(stat, 3);
            foe.stages.insert(stat, -2);
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        for stat in stats {
            assert_eq!(
                b.participants(Side::Allies)[0].stage(stat),
                if success { -2 } else { 3 }
            );
            assert_eq!(
                b.participants(Side::Foes)[0].stage(stat),
                if success { 3 } else { -2 }
            );
        }
    }
}

#[test]
fn heart_swap_swaps_stages_through_substitute_without_ability_scaling() {
    use wilds_battle_engine::Stat;
    let stats = [
        Stat::Attack,
        Stat::Defense,
        Stat::SpAttack,
        Stat::SpDefense,
        Stat::Speed,
        Stat::Accuracy,
        Stat::Evasion,
    ];
    for (response, success) in [("splash", true), ("protect", false)] {
        let mut user = mon("User", &["heart-swap"]);
        user.ability = Some("contrary".into());
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = 50;
        foe.ability = Some("simple".into());
        for stat in stats {
            user.stages.insert(stat, 3);
            foe.stages.insert(stat, -2);
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        for stat in stats {
            assert_eq!(
                b.participants(Side::Allies)[0].stage(stat),
                if success { -2 } else { 3 }
            );
            assert_eq!(
                b.participants(Side::Foes)[0].stage(stat),
                if success { 3 } else { -2 }
            );
        }
    }
}

#[test]
fn topsy_turvy_inverts_positive_and_negative_stages_without_contrary() {
    use wilds_battle_engine::Stat;
    for (sub, changed) in [(0, true), (50, false)] {
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = Some("contrary".into());
        foe.substitute_hp = sub;
        foe.stages.insert(Stat::Attack, 4);
        foe.stages.insert(Stat::Defense, -3);
        foe.stages.insert(Stat::Accuracy, 2);
        let mut b = game(mon("User", &["topsy-turvy"]), vec![foe], 0.5);
        turn(&mut b);
        let p = &b.participants(Side::Foes)[0];
        let sign = if changed { -1 } else { 1 };
        assert_eq!(p.stage(Stat::Attack), 4 * sign);
        assert_eq!(p.stage(Stat::Defense), -3 * sign);
        assert_eq!(p.stage(Stat::Accuracy), 2 * sign);
    }
    let mut b = game(
        mon("User", &["topsy-turvy"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn refresh_cures_burn_poison_and_paralysis_and_fails_when_healthy() {
    use wilds_battle_engine::Status;
    for status in [
        None,
        Some(Status::Burned),
        Some(Status::Poisoned),
        Some(Status::BadlyPoisoned),
        Some(Status::Paralyzed),
    ] {
        let mut user = mon("User", &["refresh"]);
        user.status = status;
        let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].status, None);
        assert_eq!(
            b.participants(Side::Allies)[0].last_move_failed,
            status.is_none()
        );
    }
}

#[test]
fn purify_heals_user_only_when_it_cures_the_target() {
    use wilds_battle_engine::Status;
    for (status, sub, expected) in [
        (Some(Status::Burned), 0, 1000),
        (None, 0, 500),
        (Some(Status::Burned), 50, 500),
    ] {
        let mut user = mon("User", &["purify"]);
        user.hp = 500;
        user.speed = 200;
        let mut foe = mon("Foe", &["splash"]);
        foe.status = status;
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, expected);
        if sub == 0 {
            assert_eq!(b.participants(Side::Foes)[0].status, None);
        }
    }
}

#[test]
fn psycho_shift_cures_user_only_after_successfully_transferring_status() {
    use wilds_battle_engine::Status;
    for (kind, status, transferred) in [
        (PokemonType::Normal, None, true),
        (PokemonType::Fire, None, false),
        (PokemonType::Normal, Some(Status::Poisoned), false),
    ] {
        let mut user = mon("User", &["psycho-shift"]);
        user.status = Some(Status::Burned);
        user.speed = 200;
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![kind];
        foe.status = status;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].status,
            if transferred {
                None
            } else {
                Some(Status::Burned)
            }
        );
        assert_eq!(
            b.participants(Side::Foes)[0].status,
            if transferred {
                Some(Status::Burned)
            } else {
                status
            }
        );
    }
}

#[test]
fn focus_energy_adds_two_critical_stages_and_does_not_stack() {
    for (attack, dice, ability, critical) in [
        ("tackle", 0.25, None, true),
        ("tackle", 0.75, None, false),
        ("slash", 0.99, None, true),
        ("tackle", 0.25, Some("battle-armor"), false),
    ] {
        let user = mon("User", &["focus-energy", attack]);
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = ability.map(str::to_owned);
        let mut b = game(user, vec![foe], dice);
        turn(&mut b);
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
        let events = turn_with(&mut b, |p| {
            p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {attack}else{"splash"})).unwrap().id()
        });
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e,BattleEvent::Message(s)if s=="A critical hit!")),
            critical,
            "{attack} {dice} {ability:?}"
        );
    }
}

#[test]
fn laser_focus_can_refresh_and_guarantees_critical_hits_for_the_next_turn_only() {
    let mut b = game(
        mon("User", &["laser-focus", "tackle"]),
        vec![mon("Foe", &["splash"])],
        0.75,
    );
    turn(&mut b);
    turn(&mut b);
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
    for critical in [true, false] {
        let events = turn_with(&mut b, |p| {
            p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {"tackle"}else{"splash"})).unwrap().id()
        });
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e,BattleEvent::Message(s)if s=="A critical hit!")),
            critical
        );
    }
}

#[test]
fn lucky_chant_blocks_guaranteed_criticals_for_five_turns_without_refreshing() {
    let mut user = mon("User", &["lucky-chant", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["frost-breath"])], 0.5);
    for round in 0..6 {
        let events = turn_with(&mut b, |p| {
            p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {if round<2{"lucky-chant"}else{"splash"}}else{"frost-breath"})).unwrap().id()
        });
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e,BattleEvent::Message(s)if s=="A critical hit!")),
            round == 5
        );
        if round == 1 {
            assert!(b.participants(Side::Allies)[0].last_move_failed);
        }
    }
}

#[test]
fn psych_up_copies_stages_through_protect_and_substitute() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["psych-up"]);
    user.stages.insert(Stat::Defense, 5);
    user.ability = Some("contrary".into());
    let mut foe = mon("Foe", &["protect"]);
    foe.substitute_hp = 50;
    foe.stages.insert(Stat::Attack, 3);
    foe.stages.insert(Stat::Accuracy, -2);
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    let p = &b.participants(Side::Allies)[0];
    assert_eq!(p.stage(Stat::Attack), 3);
    assert_eq!(p.stage(Stat::Accuracy), -2);
    assert_eq!(p.stage(Stat::Defense), 0);
}

#[test]
fn psych_up_copies_focus_energy_critical_bonus() {
    let mut user = mon("User", &["psych-up", "tackle"]);
    user.speed = 1;
    let mut b = game(user, vec![mon("Foe", &["focus-energy", "splash"])], 0.25);
    turn(&mut b);
    let events = turn_with(&mut b, |p| {
        p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {"tackle"}else{"splash"})).unwrap().id()
    });
    assert!(
        events
            .iter()
            .any(|e| matches!(e,BattleEvent::Message(s)if s=="A critical hit!"))
    );
}

fn check_lock_on(move_id: &str) {
    let mut user = mon("User", &[move_id, "tackle"]);
    user.speed = 200;
    let mut b = game(user.clone(), vec![mon("Foe", &["dive", "splash"])], 0.99);
    turn(&mut b);
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {"tackle"}else{"dive"})).unwrap().id()
    });
    let hp = b.participants(Side::Foes)[0].hp;
    assert!(hp < 1000);
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {"tackle"}else{"splash"})).unwrap().id()
    });
    assert_eq!(b.participants(Side::Foes)[0].hp, hp);
    let mut b = game(user.clone(), vec![mon("Foe", &["protect"])], 0.5);
    turn(&mut b);
    assert!(
        b.participants(Side::Allies)[0]
            .conditions
            .iter()
            .all(|c| c.name != "lock_on")
    );
    let mut b = game(
        user,
        vec![mon("Foe", &["splash"]), mon("Bench", &["splash"])],
        0.5,
    );
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Foes {
                    matches!(c, Choice::Switch { .. })
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
    assert!(b.participants(Side::Allies)[0].conditions.iter().any(|c|c.name=="lock_on" && c.rules.iter().any(|r|matches!(r,wilds_battle_engine::Rule::AlwaysHit{against:Some(id)}if id.index==1))));
}

#[test]
fn mind_reader_locks_on_for_one_turn_and_releases_when_target_switches() {
    check_lock_on("mind-reader");
}

#[test]
fn lock_on_locks_on_for_one_turn_and_releases_when_target_switches() {
    check_lock_on("lock-on");
}

#[test]
fn belly_drum_pays_half_max_hp_and_raises_attack_from_any_stage() {
    use wilds_battle_engine::Stat;
    for (hp, stage, ability, expected_hp, expected_stage) in [
        (1000, -6, None, 500, 6),
        (501, 0, None, 1, 6),
        (500, 0, None, 500, 0),
        (1000, 6, None, 1000, 6),
        (1000, 3, Some("contrary"), 500, -6),
        (1000, -6, Some("simple"), 500, 6),
    ] {
        let mut user = mon("User", &["belly-drum"]);
        user.hp = hp;
        user.stages.insert(Stat::Attack, stage);
        user.ability = ability.map(str::to_owned);
        let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut b);
        let p = &b.participants(Side::Allies)[0];
        assert_eq!(p.hp, expected_hp);
        assert_eq!(p.stage(Stat::Attack), expected_stage);
        assert_eq!(p.last_move_failed, expected_hp == hp);
    }
}

#[test]
fn spit_up_scales_with_stockpile_and_consumes_charges_even_when_protected() {
    use wilds_battle_engine::Stat;
    for (charges, protect) in [(0, false), (1, false), (2, false), (3, false), (2, true)] {
        let user = mon("User", &["stockpile", "spit-up"]);
        let foe = mon("Foe", &["splash", "protect"]);
        let expected = if charges == 0 || protect {
            0
        } else {
            control_damage(
                charges * 100,
                "Normal",
                "Special",
                user.clone(),
                foe.clone(),
            )
        };
        let mut b = game(user, vec![foe], 0.5);
        for _ in 0..charges {
            turn(&mut b);
        }
        turn_with(&mut b, |p| {
            p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Allies {"spit-up"}else if protect{"protect"}else{"splash"})).unwrap().id()
        });
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
        let p = &b.participants(Side::Allies)[0];
        assert_eq!(p.stage(Stat::Defense), 0);
        assert_eq!(p.stage(Stat::SpDefense), 0);
        assert!(
            p.conditions
                .iter()
                .all(|c| !c.name.starts_with("stockpile"))
        );
    }
}

#[test]
fn whirlpool_binds_and_hits_diving_targets_at_double_power() {
    check_partial_trap("whirlpool", 35, "Water", "Special");
    let mut user = mon("User", &["whirlpool"]);
    user.speed = 1;
    let foe = mon("Foe", &["dive"]);
    let expected = control_damage(70, "Water", "Special", user.clone(), foe.clone()) + 125;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
    assert!(b.participants(Side::Foes)[0].bound.is_some());
}

#[test]
fn spite_removes_four_pp_from_the_last_move_and_fails_at_zero() {
    for (pp, sub, remaining, failed) in [
        (35, 0, 30, false),
        (3, 0, 0, false),
        (1, 0, 0, true),
        (35, 50, 30, false),
    ] {
        let mut user = mon("User", &["spite"]);
        user.speed = 1;
        let mut foe = mon("Foe", &["tackle"]);
        foe.move_pp.insert("tackle".into(), pp);
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], remaining);
        assert_eq!(b.participants(Side::Allies)[0].last_move_failed, failed);
    }
}

#[test]
fn gastro_acid_suppresses_ordinary_abilities_but_respects_protected_ones_and_substitute() {
    for (ability, sub, suppressed) in [
        (Some("huge-power"), 0, true),
        (Some("multitype"), 0, false),
        (Some("huge-power"), 50, false),
        (None, 0, false),
    ] {
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = ability.map(str::to_owned);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["gastro-acid"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].ability_suppressed, suppressed);
        assert_eq!(
            b.participants(Side::Allies)[0].last_move_failed,
            !suppressed
        );
    }
}

#[test]
fn acupressure_chooses_only_stats_below_the_cap() {
    use wilds_battle_engine::Stat;
    let stats = [
        Stat::Attack,
        Stat::Defense,
        Stat::SpAttack,
        Stat::SpDefense,
        Stat::Speed,
        Stat::Accuracy,
        Stat::Evasion,
    ];
    for capped in [false, true] {
        let mut user = mon("User", &["acupressure"]);
        for stat in stats {
            user.stages.insert(stat, 6);
        }
        if !capped {
            user.stages.insert(Stat::Evasion, 3);
        }
        let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].stage(Stat::Evasion),
            if capped { 6 } else { 5 }
        );
        assert_eq!(b.participants(Side::Allies)[0].last_move_failed, capped);
    }
}

#[test]
fn heal_bell_cures_the_party_including_bench_members() {
    use wilds_battle_engine::Status;
    let mut user = mon("User", &["heal-bell"]);
    user.status = Some(Status::Burned);
    let mut bench = mon("Bench", &["splash"]);
    bench.status = Some(Status::Poisoned);
    let mut blocked = mon("Blocked", &["splash"]);
    blocked.status = Some(Status::Burned);
    blocked.ability = Some("soundproof".into());
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, bench, blocked], vec![mon("Foe", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0, 2], vec![0]]).unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].status, None);
    assert_eq!(b.participants(Side::Allies)[1].status, None);
    assert_eq!(b.participants(Side::Allies)[2].status, Some(Status::Burned));
}

#[test]
fn aromatherapy_cures_the_party_including_bench_members() {
    use wilds_battle_engine::Status;
    let mut user = mon("User", &["aromatherapy"]);
    user.status = Some(Status::Burned);
    let mut bench = mon("Bench", &["splash"]);
    bench.status = Some(Status::Poisoned);
    let mut blocked = mon("Blocked", &["splash"]);
    blocked.status = Some(Status::Burned);
    blocked.ability = Some("sapsipper".into());
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, bench, blocked], vec![mon("Foe", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0, 2], vec![0]]).unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].status, None);
    assert_eq!(b.participants(Side::Allies)[1].status, None);
    assert_eq!(b.participants(Side::Allies)[2].status, Some(Status::Burned));
}

#[test]
fn jungle_healing_recovers_active_allies_and_cures_status_through_substitute() {
    use wilds_battle_engine::Status;
    let mut user = mon("User", &["jungle-healing"]);
    user.hp = 500;
    user.status = Some(Status::Burned);
    let mut ally = mon("Ally", &["protect"]);
    ally.status = Some(Status::Poisoned);
    ally.substitute_hp = 50;
    let mut bench = mon("Bench", &["splash"]);
    bench.hp = 500;
    bench.status = Some(Status::Burned);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, ally, bench], vec![mon("Foe", &["splash"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0, 1], vec![0]]).unwrap();
    turn(&mut b);
    let allies = b.participants(Side::Allies);
    assert_eq!(allies[0].hp, 750);
    assert_eq!(allies[0].status, None);
    assert_eq!(allies[1].status, None);
    assert_eq!(allies[2].hp, 500);
    assert_eq!(allies[2].status, Some(Status::Burned));
}

#[test]
fn celebrate_spends_pp_and_announces_without_changing_combat_state() {
    let mut b = game(
        mon("User", &["celebrate"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    let events = turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert_eq!(b.participants(Side::Allies)[0].move_pp["celebrate"], 39);
    assert!(
        events
            .iter()
            .any(|e| matches!(e,BattleEvent::Message(s)if s=="Congratulations!"))
    );
}

#[test]
fn counter_retaliates_only_against_physical_hp_damage_and_respects_ghost_immunity() {
    for (attack, sub, ghost, returns) in [
        ("tackle", 0, false, true),
        ("water-gun", 0, false, false),
        ("tackle", 500, false, false),
        ("tackle", 0, true, false),
    ] {
        let mut user = mon("User", &["counter"]);
        user.substitute_hp = sub;
        let mut foe = mon("Foe", &[attack]);
        if ghost {
            foe.types = vec![PokemonType::Ghost];
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let lost = 1000 - b.participants(Side::Allies)[0].hp;
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            if returns { 2 * lost } else { 0 }
        );
    }
}

#[test]
fn counter_keeps_the_last_physical_hit_even_after_a_special_hit() {
    let user = mon("User", &["counter"]);
    let mut physical = mon("Physical", &["tackle"]);
    physical.speed = 200;
    let special = mon("Special", &["water-gun"]);
    let expected = 2 * control_damage(40, "Normal", "Physical", physical.clone(), user.clone());
    let mut b = game(user, vec![physical, special], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
    assert_eq!(b.participants(Side::Foes)[1].hp, 1000);
}

#[test]
fn counter_hits_a_pivot_replacement_in_the_attackers_field_position() {
    let mut b = game(
        mon("User", &["counter"]),
        vec![mon("Foe", &["u-turn"]), mon("Bench", &["splash"])],
        0.5,
    );
    b.set_active([vec![0], vec![0]]).unwrap();
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap_or(&p.choices[0])
            .id()
    });
    assert_eq!(b.active_indices(Side::Foes), &[1]);
    assert_eq!(
        1000 - b.participants(Side::Foes)[1].hp,
        2 * (1000 - b.participants(Side::Allies)[0].hp)
    );
}

#[test]
fn counter_applies_redirection_after_resolving_the_attacker() {
    use wilds_battle_engine::{Condition, ParticipantId, Rule};
    let mut foe = mon("Attacker", &["tackle"]);
    foe.conditions.push(Condition {
        name: "redirect".into(),
        value: 1,
        turns_left: None,
        group: None,
        end_message: None,
        rules: vec![Rule::Redirect {
            to: ParticipantId {
                side: Side::Foes,
                index: 1,
            },
            move_type: None,
            priority: 2,
        }],
    });
    let mut b = game(
        mon("User", &["counter"]),
        vec![foe, mon("Redirector", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert_eq!(
        1000 - b.participants(Side::Foes)[1].hp,
        2 * (1000 - b.participants(Side::Allies)[0].hp)
    );
}

#[test]
fn mirror_coat_returns_special_damage_only_and_respects_dark_immunity() {
    for (attack, dark, returns) in [
        ("water-gun", false, true),
        ("tackle", false, false),
        ("water-gun", true, false),
    ] {
        let mut foe = mon("Foe", &[attack]);
        if dark {
            foe.types = vec![PokemonType::Dark];
        }
        let mut b = game(mon("User", &["mirror-coat"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            if returns {
                2 * (1000 - b.participants(Side::Allies)[0].hp)
            } else {
                0
            }
        );
    }
}

#[test]
fn metal_burst_returns_one_and_a_half_times_damage_only_if_it_acts_after_the_hit() {
    for (attack, speed, returns) in [
        ("tackle", 1, true),
        ("water-gun", 1, true),
        ("tackle", 200, false),
    ] {
        let mut user = mon("User", &["metal-burst"]);
        user.speed = speed;
        let mut b = game(user, vec![mon("Foe", &[attack])], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            if returns {
                (1000 - b.participants(Side::Allies)[0].hp) * 3 / 2
            } else {
                0
            }
        );
        assert_eq!(b.participants(Side::Allies)[0].last_move_failed, !returns);
    }
}

#[test]
fn pain_split_averages_hp_with_flooring_and_individual_maximums() {
    for (user_hp, foe_hp, foe_max, sub, user_expected, foe_expected) in [
        (200, 1000, 1000, 0, 600, 600),
        (999, 1000, 1000, 0, 999, 999),
        (900, 100, 100, 0, 500, 100),
        (200, 1000, 1000, 50, 200, 1000),
    ] {
        let mut user = mon("User", &["pain-split"]);
        user.hp = user_hp;
        let mut foe = mon("Foe", &["splash"]);
        foe.hp = foe_hp;
        foe.max_hp = foe_max;
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, user_expected);
        assert_eq!(b.participants(Side::Foes)[0].hp, foe_expected);
    }
}

#[test]
fn pain_split_ignores_ghost_immunity_and_does_not_count_as_hit_damage() {
    let mut user = mon("User", &["pain-split"]);
    user.hp = 200;
    let mut foe = mon("Foe", &["counter"]);
    foe.types = vec![PokemonType::Ghost];
    let mut b = game(user, vec![foe], 0.5);
    let events = turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 600);
    assert_eq!(b.participants(Side::Foes)[0].hp, 600);
    assert_eq!(b.participants(Side::Foes)[0].damage_taken, 0);
    assert!(!b.participants(Side::Foes)[0].hurt_this_turn);
    assert!(b.participants(Side::Foes)[0].last_move_failed);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, BattleEvent::HpSet { .. }))
            .count(),
        2
    );
}

#[test]
fn toxic_thread_still_lowers_speed_when_poison_is_immune() {
    use wilds_battle_engine::{Stat, Status};
    for (kind, sub, slowed, poisoned) in [
        (PokemonType::Normal, 0, true, true),
        (PokemonType::Steel, 0, true, false),
        (PokemonType::Normal, 50, false, false),
    ] {
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["toxic-thread"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].stage(Stat::Speed),
            if slowed { -1 } else { 0 }
        );
        assert_eq!(
            b.participants(Side::Foes)[0].status,
            if poisoned {
                Some(Status::Poisoned)
            } else {
                None
            }
        );
    }
}

#[test]
fn throat_chop_blocks_selected_sound_moves_and_expires_after_the_next_turn() {
    use wilds_battle_engine::Status;
    let mut user = mon("User", &["throat-chop", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["sing", "splash"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].status, None);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Foes {
            assert!(
                !p.choices
                    .iter()
                    .any(|c| matches!(c,Choice::UseMove{move_id,..}if move_id=="sing"))
            );
        }
        p.choices
            .iter()
            .find(|c| matches!(c,Choice::UseMove{move_id,..}if move_id=="splash"))
            .unwrap()
            .id()
    });
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c|matches!(c,Choice::UseMove{move_id,..}if move_id==if p.actor.side==Side::Foes{"sing"}else{"splash"})).unwrap().id()
    });
    assert_eq!(b.participants(Side::Allies)[0].status, Some(Status::Asleep));
}

#[test]
fn flower_shield_raises_defense_of_all_active_grass_types_through_protection() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["flower-shield"]);
    user.types = vec![PokemonType::Grass];
    let mut grass = mon("Grass", &["protect"]);
    grass.types = vec![PokemonType::Grass];
    grass.substitute_hp = 50;
    let normal = mon("Normal", &["splash"]);
    let mut b = game(user, vec![grass, normal], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Defense), 1);
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Defense), 1);
    assert_eq!(b.participants(Side::Foes)[1].stage(Stat::Defense), 0);
}

#[test]
fn memento_lowers_both_attacks_and_faints_only_after_reaching_target() {
    use wilds_battle_engine::Stat;
    for (response, sub, lands) in [
        ("splash", 0, true),
        ("protect", 0, false),
        ("splash", 50, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["memento"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if lands { 0 } else { 1000 }
        );
        for stat in [Stat::Attack, Stat::SpAttack] {
            assert_eq!(
                b.participants(Side::Foes)[0].stage(stat),
                if lands { -2 } else { 0 }
            );
        }
    }
}

#[test]
fn sappy_seed_attacks_then_seeds_non_grass_targets_outside_substitute() {
    for (kind, sub, seeded) in [
        (PokemonType::Normal, 0, true),
        (PokemonType::Grass, 0, false),
        (PokemonType::Normal, 50, false),
    ] {
        let mut user = mon("User", &["sappy-seed"]);
        user.hp = 500;
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let expected = control_damage(100, "Grass", "Physical", user.clone(), foe.clone())
            + if seeded { 125 } else { 0 };
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if seeded { 625 } else { 500 }
        );
    }
}

#[test]
fn sappy_seed_can_be_reflected_by_magic_bounce() {
    let mut foe = mon("Foe", &["splash"]);
    foe.ability = Some("magic-bounce".into());
    let mut b = game(mon("User", &["sappy-seed"]), vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Allies)[0].hp < 875);
    assert!(
        b.participants(Side::Allies)[0]
            .conditions
            .iter()
            .any(|c| c.name == "leech_seed")
    );
}

#[test]
fn corrosive_gas_removes_adjacent_items_but_respects_protect_and_sticky_hold() {
    let mut user = mon("User", &["corrosive-gas"]);
    user.item = Some("power-herb".into());
    let mut open = mon("Open", &["splash"]);
    open.item = Some("power-herb".into());
    let mut protected = mon("Protected", &["protect"]);
    protected.item = Some("power-herb".into());
    let mut b = game(user.clone(), vec![open, protected], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Foes)[0].item.is_none());
    assert!(b.participants(Side::Foes)[1].item.is_some());
    assert!(b.participants(Side::Allies)[0].item.is_some());
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
    let mut sticky = mon("Sticky", &["splash"]);
    sticky.ability = Some("sticky-hold".into());
    sticky.item = Some("power-herb".into());
    let mut b = game(user, vec![sticky], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Foes)[0].item.is_some());
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn teatime_forces_existing_berries_to_be_eaten_through_protect_and_substitute() {
    let mut user = mon("User", &["teatime"]);
    user.hp = 900;
    user.item = Some("oran-berry".into());
    let mut foe = mon("Foe", &["protect"]);
    foe.hp = 900;
    foe.item = Some("sitrus-berry".into());
    foe.substitute_hp = 50;
    let mut other = mon("Other", &["splash"]);
    other.item = Some("power-herb".into());
    let mut b = game(user, vec![foe, other], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 910);
    assert!(b.participants(Side::Allies)[0].item.is_none());
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Foes)[0].item.is_none());
    assert!(b.participants(Side::Foes)[1].item.is_some());
}

#[test]
fn teatime_without_berries_is_not_a_stomping_tantrum_failure() {
    let mut b = game(
        mon("User", &["teatime"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
}

fn choose_move(prompt: &Prompt, allies: &str, foes: &str) -> u32 {
    let desired = if prompt.actor.side == Side::Allies {
        allies
    } else {
        foes
    };
    prompt
        .choices
        .iter()
        .find(|c| matches!(c, Choice::UseMove {move_id,..} if move_id == desired))
        .unwrap()
        .id()
}

#[test]
fn recycle_restores_consumed_held_items_once_but_not_removed_items() {
    let mut user = mon("User", &["solar_beam", "recycle"]);
    user.item = Some("power-herb".into());
    let mut foe = mon("Foe", &["splash", "knock-off"]);
    foe.speed = 200;
    let mut b = game(user, vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, "solar_beam", "splash"));
    assert!(b.participants(Side::Allies)[0].item.is_none());
    assert_eq!(
        b.participants(Side::Allies)[0].recyclable_item.as_deref(),
        Some("power-herb")
    );
    turn_with(&mut b, |p| choose_move(p, "recycle", "splash"));
    assert_eq!(
        b.participants(Side::Allies)[0].item.as_deref(),
        Some("power-herb")
    );
    assert!(b.participants(Side::Allies)[0].recyclable_item.is_none());
    turn_with(&mut b, |p| choose_move(p, "recycle", "knock-off"));
    assert!(b.participants(Side::Allies)[0].item.is_none());
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn recycle_ignores_stolen_berries_and_preserves_prior_consumption_history() {
    for previous in [None, Some("power-herb")] {
        let mut user = mon("User", &["bug-bite", "recycle"]);
        user.recyclable_item = previous.map(str::to_owned);
        let mut foe = mon("Foe", &["splash"]);
        foe.item = Some("oran-berry".into());
        let mut b = game(user, vec![foe], 0.5);
        turn_with(&mut b, |p| choose_move(p, "bug-bite", "splash"));
        assert_eq!(
            b.participants(Side::Allies)[0]
                .last_consumed_item
                .as_deref(),
            Some("oran-berry")
        );
        turn_with(&mut b, |p| choose_move(p, "recycle", "splash"));
        let user = &b.participants(Side::Allies)[0];
        assert_eq!(user.item.as_deref(), previous);
        assert_eq!(user.last_move_failed, previous.is_none());
        assert!(user.ate_berry);
    }
}

#[test]
fn power_split_averages_raw_offenses_without_copying_stages_or_abilities() {
    use wilds_battle_engine::Stat;
    for (response, sub, lands) in [
        ("splash", 0, true),
        ("protect", 0, false),
        ("splash", 50, false),
    ] {
        let mut user = mon("User", &["power-split"]);
        user.attack = 51;
        user.sp_attack = 73;
        user.change_stage(Stat::Attack, 4);
        user.ability = Some("huge-power".into());
        let mut foe = mon("Foe", &[response]);
        foe.attack = 202;
        foe.sp_attack = 300;
        foe.substitute_hp = sub;
        foe.change_stage(Stat::Attack, -2);
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let mine = &b.participants(Side::Allies)[0];
        let theirs = &b.participants(Side::Foes)[0];
        assert_eq!(
            (mine.attack, mine.sp_attack),
            if lands { (126, 186) } else { (51, 73) }
        );
        assert_eq!(
            (theirs.attack, theirs.sp_attack),
            if lands { (126, 186) } else { (202, 300) }
        );
        assert_eq!(mine.stage(Stat::Attack), 4);
        assert_eq!(theirs.stage(Stat::Attack), -2);
    }
}

#[test]
fn power_split_raw_stats_are_restored_when_the_user_switches_out() {
    let mut user = mon("User", &["power-split"]);
    user.attack = 40;
    let mut foe = mon("Foe", &["splash"]);
    foe.attack = 200;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, mon("Bench", &["splash"])], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].attack, 120);
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Allies {
                    matches!(c, Choice::Switch { .. })
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert_eq!(b.participants(Side::Allies)[0].attack, 40);
    assert_eq!(b.participants(Side::Foes)[0].attack, 120);
}

#[test]
fn guard_split_averages_raw_defenses_and_respects_protection() {
    use wilds_battle_engine::Stat;
    for (response, sub, lands) in [
        ("splash", 0, true),
        ("protect", 0, false),
        ("splash", 50, false),
    ] {
        let mut user = mon("User", &["guard-split"]);
        user.defense = 71;
        user.sp_defense = 101;
        user.change_stage(Stat::Defense, 4);
        let mut foe = mon("Foe", &[response]);
        foe.defense = 202;
        foe.sp_defense = 300;
        foe.substitute_hp = sub;
        foe.change_stage(Stat::SpDefense, -3);
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let mine = &b.participants(Side::Allies)[0];
        let theirs = &b.participants(Side::Foes)[0];
        assert_eq!(
            (mine.defense, mine.sp_defense),
            if lands { (136, 200) } else { (71, 101) }
        );
        assert_eq!(
            (theirs.defense, theirs.sp_defense),
            if lands { (136, 200) } else { (202, 300) }
        );
        assert_eq!(mine.stage(Stat::Defense), 4);
        assert_eq!(theirs.stage(Stat::SpDefense), -3);
    }
}

#[test]
fn speed_swap_exchanges_raw_speed_through_substitute_and_keeps_stages() {
    use wilds_battle_engine::Stat;
    for (response, lands) in [("splash", true), ("protect", false)] {
        let mut user = mon("User", &["speed-swap"]);
        user.speed = 71;
        user.change_stage(Stat::Speed, 4);
        let mut foe = mon("Foe", &[response]);
        foe.speed = 202;
        foe.substitute_hp = 50;
        foe.change_stage(Stat::Speed, -2);
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let mine = &b.participants(Side::Allies)[0];
        let theirs = &b.participants(Side::Foes)[0];
        assert_eq!(
            (mine.speed, theirs.speed),
            if lands { (202, 71) } else { (71, 202) }
        );
        assert_eq!(mine.stage(Stat::Speed), 4);
        assert_eq!(theirs.stage(Stat::Speed), -2);
    }
}

#[test]
fn strength_sap_heals_from_attack_with_stages_but_without_huge_power() {
    use wilds_battle_engine::Stat;
    for (stage, sub, response, healed, after) in [
        (2, 0, "splash", 300, 1),
        (-2, 0, "splash", 75, -3),
        (-6, 0, "splash", 0, -6),
        (0, 50, "splash", 0, 0),
        (0, 0, "protect", 0, 0),
    ] {
        let mut user = mon("User", &["strength-sap"]);
        user.hp = 100;
        let mut foe = mon("Foe", &[response]);
        foe.attack = 150;
        foe.change_stage(Stat::Attack, stage);
        foe.substitute_hp = sub;
        foe.ability = Some("huge-power".into());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, 100 + healed);
        assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Attack), after);
    }
    let mut user = mon("User", &["strength-sap"]);
    user.hp = 100;
    let mut foe = mon("Foe", &["splash"]);
    foe.ability = Some("clear-body".into());
    foe.attack = 123;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 223);
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Attack), 0);
}

#[test]
fn power_trick_swaps_raw_attack_and_defense_and_toggles_without_changing_stages() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["power-trick"]);
    user.attack = 40;
    user.defense = 200;
    user.change_stage(Stat::Attack, 3);
    user.change_stage(Stat::Defense, -2);
    user.ability = Some("huge-power".into());
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    for expected in [(200, 40), (40, 200), (200, 40)] {
        turn(&mut b);
        let p = &b.participants(Side::Allies)[0];
        assert_eq!((p.attack, p.defense), expected);
        assert_eq!(p.stage(Stat::Attack), 3);
        assert_eq!(p.stage(Stat::Defense), -2);
    }
}

#[test]
fn soak_replaces_types_but_respects_immutable_species_and_protection() {
    for (species, response, sub, lands) in [
        (0, "splash", 0, true),
        (493, "splash", 0, false),
        (773, "splash", 0, false),
        (0, "protect", 0, false),
        (0, "splash", 50, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![PokemonType::Fire, PokemonType::Flying];
        foe.species_id = species;
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["soak"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].types,
            if lands {
                vec![PokemonType::Water]
            } else {
                vec![PokemonType::Fire, PokemonType::Flying]
            }
        );
        if lands {
            turn(&mut b);
            assert!(b.participants(Side::Allies)[0].last_move_failed);
        }
    }
}

#[test]
fn magic_powder_replaces_types_but_respects_immutable_species_and_protection() {
    for (species, response, sub, lands) in [
        (0, "splash", 0, true),
        (493, "splash", 0, false),
        (773, "splash", 0, false),
        (0, "protect", 0, false),
        (0, "splash", 50, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![PokemonType::Fire, PokemonType::Flying];
        foe.species_id = species;
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["magic-powder"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].types,
            if lands {
                vec![PokemonType::Psychic]
            } else {
                vec![PokemonType::Fire, PokemonType::Flying]
            }
        );
        if lands {
            turn(&mut b);
            assert!(b.participants(Side::Allies)[0].last_move_failed);
        }
    }
}
#[test]
fn magic_powder_cannot_change_a_grass_targets_type() {
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Grass];
    let mut b = game(mon("User", &["magic-powder"]), vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(
        b.participants(Side::Foes)[0].types,
        vec![PokemonType::Grass]
    );
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn trick_or_treat_adds_ghost_without_erasing_base_types_and_blocks_normal_damage() {
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Fire, PokemonType::Flying];
    foe.species_id = 493;
    let mut b = game(
        mon("User", &["trick-or-treat", "tackle", "soak"]),
        vec![foe],
        0.5,
    );
    turn_with(&mut b, |p| choose_move(p, "trick-or-treat", "splash"));
    let foe = &b.participants(Side::Foes)[0];
    assert_eq!(foe.types, vec![PokemonType::Fire, PokemonType::Flying]);
    assert_eq!(foe.added_type, Some(PokemonType::Ghost));
    turn_with(&mut b, |p| choose_move(p, "tackle", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    turn_with(&mut b, |p| choose_move(p, "trick-or-treat", "splash"));
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn trick_or_treat_respects_protect_and_substitute_and_soak_clears_added_type() {
    for (response, sub, lands) in [
        ("splash", 0, true),
        ("protect", 0, false),
        ("splash", 50, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["trick-or-treat", "soak"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].added_type,
            if lands {
                Some(PokemonType::Ghost)
            } else {
                None
            }
        );
        if lands {
            turn_with(&mut b, |p| choose_move(p, "soak", "splash"));
            assert_eq!(
                b.participants(Side::Foes)[0].types,
                vec![PokemonType::Water]
            );
            assert_eq!(b.participants(Side::Foes)[0].added_type, None);
        }
    }
}

#[test]
fn forests_curse_replaces_trick_or_treat_and_preserves_base_types() {
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Fire, PokemonType::Flying];
    let mut b = game(
        mon("User", &["trick-or-treat", "forests-curse"]),
        vec![foe],
        0.5,
    );
    for (id, kind) in [
        ("trick-or-treat", PokemonType::Ghost),
        ("forests-curse", PokemonType::Grass),
        ("trick-or-treat", PokemonType::Ghost),
    ] {
        turn_with(&mut b, |p| choose_move(p, id, "splash"));
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(foe.types, vec![PokemonType::Fire, PokemonType::Flying]);
        assert_eq!(foe.added_type, Some(kind));
    }
}
#[test]
fn forests_curse_respects_substitute_protect_and_existing_grass_type() {
    for (kind, response, sub) in [
        (PokemonType::Grass, "splash", 0),
        (PokemonType::Fire, "protect", 0),
        (PokemonType::Fire, "splash", 50),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["forests-curse"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].added_type, None);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn reflect_type_copies_base_and_added_types_through_substitute() {
    let mut user = mon("User", &["reflect-type"]);
    user.added_type = Some(PokemonType::Grass);
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Fire, PokemonType::Flying];
    foe.added_type = Some(PokemonType::Ghost);
    foe.substitute_hp = 50;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    let user = &b.participants(Side::Allies)[0];
    assert_eq!(user.types, vec![PokemonType::Fire, PokemonType::Flying]);
    assert_eq!(user.added_type, Some(PokemonType::Ghost));
}
#[test]
fn reflect_type_fails_for_immutable_users_protect_and_typeless_targets() {
    for (species, response, kind) in [
        (493, "splash", PokemonType::Fire),
        (773, "splash", PokemonType::Fire),
        (0, "protect", PokemonType::Fire),
        (0, "splash", PokemonType::None),
    ] {
        let mut user = mon("User", &["reflect-type"]);
        user.species_id = species;
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].types,
            vec![PokemonType::Normal]
        );
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

fn used_names(events: &[BattleEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            BattleEvent::Message(m) if m.contains(" used ") => m.split(" used ").next(),
            _ => None,
        })
        .collect()
}

#[test]
fn after_you_promotes_a_pending_ally_immediately_through_substitute() {
    let mut user = mon("User", &["after-you"]);
    user.speed = 300;
    let mut ally = mon("Slow", &["splash"]);
    ally.speed = 10;
    ally.substitute_hp = 50;
    let mut foe = mon("Foe", &["splash"]);
    foe.speed = 200;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, ally], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies && p.actor.index == 0 {
            p.choices.iter().find(|c| matches!(c,Choice::UseMove{target,..} if target.side==Side::Allies && target.index==1)).unwrap().id()
        } else {
            p.choices[0].id()
        }
    });
    assert_eq!(used_names(&events), ["User", "Slow", "Foe"]);
}

#[test]
fn after_you_fails_in_singles_and_when_the_target_has_already_moved() {
    let mut b = game(
        mon("User", &["after-you"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    let mut user = mon("User", &["after-you"]);
    user.speed = 10;
    let mut b = game(
        user,
        vec![mon("Foe", &["splash"]), mon("Other", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn quash_delays_a_pending_action_even_after_negative_priority_moves() {
    let mut user = mon("User", &["quash"]);
    user.speed = 300;
    let mut ally = mon("Ally", &["splash"]);
    ally.speed = 100;
    let mut foe = mon("Foe", &["tackle"]);
    foe.speed = 200;
    let mut slow = mon("Slow", &["whirlwind"]);
    slow.speed = 10;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, ally], vec![foe, slow]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies && p.actor.index == 0 {
            p.choices.iter().find(|c| matches!(c, Choice::UseMove {target,..} if target.side == Side::Foes && target.index == 0)).unwrap().id()
        } else {
            p.choices[0].id()
        }
    });
    assert_eq!(used_names(&events), ["User", "Ally", "Slow", "Foe"]);
}

#[test]
fn quash_fails_in_singles_and_cannot_reach_a_substitute_or_completed_action() {
    let mut b = game(mon("User", &["quash"]), vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for (response, sub) in [("splash", 50), ("protect", 0)] {
        let mut user = mon("User", &["quash"]);
        user.speed = 200;
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe, mon("Other", &["splash"])], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

fn attention_game(
    move_id: &str,
    attack: &str,
    attack_type: PokemonType,
    ability: Option<&str>,
) -> Battle {
    let user = mon("User", &[move_id, "splash"]);
    let partner = mon("Partner", &["splash"]);
    let mut foe = mon("Foe", &[attack]);
    foe.types = vec![attack_type];
    foe.ability = ability.map(str::to_owned);
    Battle::with_rng(
        Fixed(0.5),
        [vec![user, partner], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap()
}
fn attention_turn(b: &mut Battle, move_id: &str) {
    turn_with(b, |p| {
        p.choices
            .iter()
            .find(|c| {
                matches!(c,Choice::UseMove{move_id:id,target,..} if
            if p.actor.side==Side::Foes {target.index==1}
            else if p.actor.index==0 {id==move_id} else {id=="splash"})
            })
            .unwrap()
            .id()
    });
}

#[test]
fn follow_me_draws_opposing_attacks_for_one_turn_but_not_snipe_shot() {
    for attack in ["tackle", "snipe-shot"] {
        let mut b = attention_game("follow-me", attack, PokemonType::Normal, None);
        attention_turn(&mut b, "follow-me");
        assert_eq!(
            b.participants(Side::Allies)[0].hp == 1000,
            attack == "snipe-shot"
        );
        assert_eq!(
            b.participants(Side::Allies)[1].hp == 1000,
            attack != "snipe-shot"
        );
        let prior = b.participants(Side::Allies)[0].hp;
        attention_turn(&mut b, "splash");
        assert_eq!(b.participants(Side::Allies)[0].hp, prior);
        assert!(b.participants(Side::Allies)[1].hp < 1000);
    }
    let mut b = game(
        mon("User", &["follow-me"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn rage_powder_draws_attacks_except_from_grass_or_overcoat_attackers() {
    for (kind, ability, redirects) in [
        (PokemonType::Normal, None, true),
        (PokemonType::Grass, None, false),
        (PokemonType::Normal, Some("overcoat"), false),
    ] {
        let mut b = attention_game("rage-powder", "tackle", kind, ability);
        attention_turn(&mut b, "rage-powder");
        assert_eq!(b.participants(Side::Allies)[0].hp < 1000, redirects);
        assert_eq!(b.participants(Side::Allies)[1].hp < 1000, !redirects);
    }
    let mut b = game(
        mon("User", &["rage-powder"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn spotlight_takes_precedence_over_follow_me() {
    let user = mon("User", &["spotlight"]);
    let ally = mon("Ally", &["tackle"]);
    let follower = mon("Follower", &["follow-me"]);
    let lit = mon("Lit", &["splash"]);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, ally], vec![follower, lit]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies {
            let index = if p.actor.index == 0 { 1 } else { 0 };
            p.choices.iter().find(|c| matches!(c,Choice::UseMove{target,..} if target.side==Side::Foes && target.index==index)).unwrap().id()
        } else {
            p.choices[0].id()
        }
    });
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Foes)[1].hp < 1000);
}
#[test]
fn spotlight_respects_protect_and_substitute_and_fails_in_singles() {
    let mut b = game(
        mon("User", &["spotlight"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for (response, sub) in [("protect", 0), ("splash", 50)] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(
            mon("User", &["spotlight"]),
            vec![foe, mon("Other", &["splash"])],
            0.5,
        );
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

fn check_item_exchange(move_id: &str) {
    for (mine, theirs) in [
        (Some("power-herb"), Some("normal-gem")),
        (Some("power-herb"), None),
        (None, Some("normal-gem")),
        (None, None),
    ] {
        let mut user = mon("User", &[move_id]);
        user.item = mine.map(str::to_owned);
        user.ability = Some("sticky-hold".into());
        let mut foe = mon("Foe", &["splash"]);
        foe.item = theirs.map(str::to_owned);
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let user = &b.participants(Side::Allies)[0];
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(user.item.as_deref(), theirs);
        assert_eq!(foe.item.as_deref(), mine);
        assert_eq!(user.last_move_failed, mine.is_none() && theirs.is_none());
        assert!(user.consumed_items.is_empty() && foe.consumed_items.is_empty());
    }
    for (ability, response, sub) in [
        (Some("sticky-hold"), "splash", 0),
        (None, "protect", 0),
        (None, "splash", 50),
    ] {
        let mut user = mon("User", &[move_id]);
        user.item = Some("power-herb".into());
        let mut foe = mon("Foe", &[response]);
        foe.item = Some("normal-gem".into());
        foe.ability = ability.map(str::to_owned);
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].item.as_deref(),
            Some("power-herb")
        );
        assert_eq!(
            b.participants(Side::Foes)[0].item.as_deref(),
            Some("normal-gem")
        );
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
    for bound_to_receiver in [false, true] {
        let mut user = mon("User", &[move_id]);
        user.item = Some("griseous-orb".into());
        let mut foe = mon("Foe", &["splash"]);
        foe.item = Some("power-herb".into());
        if bound_to_receiver {
            foe.species_id = 487;
        } else {
            user.species_id = 487;
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].item.as_deref(),
            Some("griseous-orb")
        );
        assert_eq!(
            b.participants(Side::Foes)[0].item.as_deref(),
            Some("power-herb")
        );
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn trick_swaps_existing_items_atomically_with_empty_slots_and_removal_restrictions() {
    check_item_exchange("trick");
}

#[test]
fn switcheroo_swaps_existing_items_atomically_with_empty_slots_and_removal_restrictions() {
    check_item_exchange("switcheroo");
}

#[test]
fn bestow_gives_through_protect_substitute_and_both_participants_sticky_hold() {
    let mut user = mon("User", &["bestow"]);
    user.item = Some("power-herb".into());
    user.ability = Some("sticky-hold".into());
    let mut foe = mon("Foe", &["protect"]);
    foe.substitute_hp = 50;
    foe.ability = Some("sticky-hold".into());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].item, None);
    assert_eq!(
        b.participants(Side::Foes)[0].item.as_deref(),
        Some("power-herb")
    );
    assert!(b.participants(Side::Allies)[0].consumed_items.is_empty());
}

#[test]
fn bestow_rejects_occupied_slots_and_species_binding_without_losing_the_gift() {
    for (mine, theirs, source_species, target_species) in [
        (Some("power-herb"), Some("normal-gem"), 0, 0),
        (Some("griseous-orb"), None, 487, 0),
        (Some("griseous-orb"), None, 0, 487),
        (None, None, 0, 0),
    ] {
        let mut user = mon("User", &["bestow"]);
        user.item = mine.map(str::to_owned);
        user.species_id = source_species;
        let mut foe = mon("Foe", &["splash"]);
        foe.item = theirs.map(str::to_owned);
        foe.species_id = target_species;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].item.as_deref(), mine);
        assert_eq!(b.participants(Side::Foes)[0].item.as_deref(), theirs);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn detect_and_protect_share_the_consecutive_failure_chance() {
    let mut b = game(
        mon("User", &["protect", "detect", "splash"]),
        vec![mon("Foe", &["tackle"])],
        0.5,
    );
    turn_with(&mut b, |p| choose_move(p, "protect", "tackle"));
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    turn_with(&mut b, |p| choose_move(p, "detect", "tackle"));
    assert!(b.participants(Side::Allies)[0].hp < 1000);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    let prior = b.participants(Side::Allies)[0].hp;
    turn_with(&mut b, |p| choose_move(p, "detect", "tackle"));
    assert_eq!(b.participants(Side::Allies)[0].hp, prior);
    turn_with(&mut b, |p| choose_move(p, "protect", "tackle"));
    assert!(b.participants(Side::Allies)[0].hp < prior);
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
    let prior = b.participants(Side::Allies)[0].hp;
    turn_with(&mut b, |p| choose_move(p, "detect", "tackle"));
    assert_eq!(b.participants(Side::Allies)[0].hp, prior);
}

#[test]
fn detect_fails_if_every_other_action_already_finished() {
    let mut user = mon("User", &["detect"]);
    user.speed = 10;
    let foe = mon("Foe", &["protect"]);
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn endure_survives_multiple_direct_attacks_at_one_hp_but_not_poison() {
    for poison in [false, true] {
        let mut user = mon("User", &["endure"]);
        user.hp = 50;
        if poison {
            user.status = Some(wilds_battle_engine::Status::Poisoned);
        }
        let mut b = game(
            user,
            vec![
                mon("First", &["seismic-toss"]),
                mon("Second", &["seismic-toss"]),
            ],
            0.5,
        );
        let events = turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if poison { 0 } else { 1 }
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e,BattleEvent::Message(m) if m=="User endured the hit!"))
                .count(),
            2
        );
    }
}

#[test]
fn endure_shares_protection_chance_and_does_not_block_status_moves() {
    use wilds_battle_engine::Stat;
    let mut b = game(
        mon("User", &["protect", "endure"]),
        vec![mon("Foe", &["growl"])],
        0.5,
    );
    turn_with(&mut b, |p| choose_move(p, "protect", "growl"));
    turn_with(&mut b, |p| choose_move(p, "endure", "growl"));
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), -1);
    turn_with(&mut b, |p| choose_move(p, "endure", "growl"));
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), -2);
}

fn check_foresight(move_id: &str) {
    use wilds_battle_engine::Stat;
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Ghost, PokemonType::Steel];
    foe.change_stage(Stat::Evasion, 6);
    let mut b = game(mon("User", &[move_id, "tackle"]), vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, move_id, "splash"));
    turn_with(&mut b, |p| choose_move(p, "tackle", "splash"));
    assert!(b.participants(Side::Foes)[0].hp < 1000);
    assert_eq!(
        b.participants(Side::Foes)[0].types,
        vec![PokemonType::Ghost, PokemonType::Steel]
    );
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Evasion), 6);
    turn_with(&mut b, |p| choose_move(p, move_id, "splash"));
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for (response, lands) in [("splash", true), ("protect", false)] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = 50;
        let mut b = game(mon("User", &[move_id]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0]
                .condition("foresight")
                .is_some(),
            lands
        );
    }
    let user = mon("User", &[move_id, "shadow-ball"]);
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Ghost];
    let expected = control_damage(80, "Ghost", "Special", user.clone(), foe.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, move_id, "splash"));
    turn_with(&mut b, |p| choose_move(p, "shadow-ball", "splash"));
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
}

#[test]
fn foresight_bypasses_ghost_immunities_and_positive_evasion_without_erasing_type() {
    check_foresight("foresight");
}

#[test]
fn foresight_preserves_negative_evasion_for_incoming_accuracy() {
    use wilds_battle_engine::Stat;
    let mut foe = mon("Foe", &["splash"]);
    foe.change_stage(Stat::Evasion, -6);
    let mut b = game(mon("User", &["foresight", "zap-cannon"]), vec![foe], 0.9);
    turn_with(&mut b, |p| choose_move(p, "foresight", "splash"));
    turn_with(&mut b, |p| choose_move(p, "zap-cannon", "splash"));
    assert!(b.participants(Side::Foes)[0].hp < 1000);
}

#[test]
fn odor_sleuth_identifies_ghost_types_and_bypasses_positive_evasion() {
    check_foresight("odor-sleuth");
}

#[test]
fn miracle_eye_enables_psychic_hits_against_dark_types_through_positive_evasion() {
    use wilds_battle_engine::Stat;
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Dark];
    foe.change_stage(Stat::Evasion, 6);
    let mut b = game(mon("User", &["miracle-eye", "psychic"]), vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, "miracle-eye", "splash"));
    turn_with(&mut b, |p| choose_move(p, "psychic", "splash"));
    assert!(b.participants(Side::Foes)[0].hp < 1000);
    assert_eq!(b.participants(Side::Foes)[0].types, vec![PokemonType::Dark]);
}
#[test]
fn miracle_eye_cannot_stack_with_foresight_and_respects_protect_but_not_substitute() {
    for first in ["foresight", "odor-sleuth", "miracle-eye"] {
        let mut b = game(
            mon("User", &["foresight", "odor-sleuth", "miracle-eye"]),
            vec![mon("Foe", &["splash"])],
            0.5,
        );
        turn_with(&mut b, |p| choose_move(p, first, "splash"));
        let second = if first == "miracle-eye" {
            "foresight"
        } else {
            "miracle-eye"
        };
        turn_with(&mut b, |p| choose_move(p, second, "splash"));
        assert!(b.participants(Side::Allies)[0].last_move_failed);
        assert_eq!(b.participants(Side::Foes)[0].conditions.len(), 1);
    }
    for (response, lands) in [("splash", true), ("protect", false)] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = 50;
        let mut b = game(mon("User", &["miracle-eye"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0]
                .condition("miracle_eye")
                .is_some(),
            lands
        );
    }
}

#[test]
fn tar_shot_lowers_speed_and_changes_fire_effectiveness_for_wonder_guard() {
    use wilds_battle_engine::Stat;
    for (kind, damaged) in [(PokemonType::Normal, true), (PokemonType::Water, false)] {
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![kind];
        foe.ability = Some("wonder-guard".into());
        let mut b = game(mon("User", &["tar-shot", "ember"]), vec![foe], 0.5);
        turn_with(&mut b, |p| choose_move(p, "ember", "splash"));
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
        turn_with(&mut b, |p| choose_move(p, "tar-shot", "splash"));
        assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Speed), -1);
        turn_with(&mut b, |p| choose_move(p, "ember", "splash"));
        assert_eq!(b.participants(Side::Foes)[0].hp < 1000, damaged);
    }
}

#[test]
fn tar_shot_does_not_stack_weakness_and_activates_a_resistance_berry() {
    use wilds_battle_engine::Stat;
    let user = mon("User", &["tar-shot", "ember"]);
    let foe = mon("Foe", &["splash"]);
    let normal = control_damage(40, "Fire", "Special", user.clone(), foe.clone());
    let mut foe = foe;
    foe.item = Some("occa-berry".into());
    let mut b = game(user, vec![foe], 0.5);
    for _ in 0..2 {
        turn_with(&mut b, |p| choose_move(p, "tar-shot", "splash"));
    }
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Speed), -2);
    turn_with(&mut b, |p| choose_move(p, "ember", "splash"));
    assert!(b.participants(Side::Foes)[0].item.is_none());
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, normal);
}

#[test]
fn tar_shot_cannot_apply_either_effect_through_protect_or_substitute() {
    use wilds_battle_engine::Stat;
    for (response, sub) in [("protect", 0), ("splash", 50)] {
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["tar-shot"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Speed), 0);
        assert!(
            b.participants(Side::Foes)[0]
                .condition("tar_shot")
                .is_none()
        );
    }
}

#[test]
fn magnitude_uses_the_seven_power_brackets_and_doubles_against_dig() {
    for (roll, power) in [
        (0, 10),
        (4, 10),
        (5, 30),
        (14, 30),
        (15, 50),
        (34, 50),
        (35, 70),
        (64, 70),
        (65, 90),
        (84, 90),
        (85, 110),
        (94, 110),
        (95, 150),
        (99, 150),
    ] {
        let user = mon("User", &["magnitude"]);
        let foe = mon("Foe", &["splash"]);
        let expected = control_damage(power, "Ground", "Physical", user.clone(), foe.clone());
        let mut b = Battle::with_rng(
            Rolls(roll, 0.5),
            [vec![user], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "roll {roll}"
        );
    }
    let user = mon("User", &["magnitude"]);
    let mut foe = mon("Foe", &["dig"]);
    foe.speed = 200;
    let expected = control_damage(20, "Ground", "Physical", user.clone(), foe.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
}

#[test]
fn magnitude_hits_adjacent_allies_and_foes_but_not_distant_slots_or_flying_types() {
    let mut user = mon("User", &["magnitude"]);
    user.speed = 200;
    let near = mon("Near", &["splash"]);
    let far = mon("Far", &["splash"]);
    let mut flying = mon("Flying", &["splash"]);
    flying.types = vec![PokemonType::Flying];
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![user, near, far],
            vec![
                mon("DistantFoe", &["splash"]),
                mon("NearFoe", &["splash"]),
                flying,
            ],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut b);
    assert!(b.participants(Side::Allies)[1].hp < 1000);
    assert_eq!(b.participants(Side::Allies)[2].hp, 1000);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Foes)[1].hp < 1000);
    assert_eq!(b.participants(Side::Foes)[2].hp, 1000);
}

#[test]
fn nightmare_drains_sleeping_targets_and_ends_when_they_wake() {
    let mut foe = mon("Foe", &["splash"]);
    foe.status = Some(wilds_battle_engine::Status::Asleep);
    foe.status_turns = 1;
    let mut b = game(mon("User", &["nightmare", "splash"]), vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
    turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
    assert!(
        b.participants(Side::Foes)[0]
            .condition("nightmare")
            .is_none()
    );
}

#[test]
fn cured_nightmare_does_not_resume_during_a_later_sleep() {
    let mut user = mon("User", &["nightmare", "purify", "splash"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["splash", "rest"]);
    foe.status = Some(wilds_battle_engine::Status::Asleep);
    foe.status_turns = 4;
    let mut b = game(user, vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, "nightmare", "splash"));
    turn_with(&mut b, |p| choose_move(p, "purify", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
    assert!(
        b.participants(Side::Foes)[0]
            .condition("nightmare")
            .is_none()
    );
    turn_with(&mut b, |p| choose_move(p, "splash", "rest"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert_eq!(
        b.participants(Side::Foes)[0].status,
        Some(wilds_battle_engine::Status::Asleep)
    );
}

#[test]
fn nightmare_respects_protect_and_substitute_and_accepts_comatose() {
    for (ability, response, sub, works) in [
        (None, "splash", 0, false),
        (Some("comatose"), "splash", 0, true),
        (Some("comatose"), "protect", 0, false),
        (Some("comatose"), "splash", 50, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.ability = ability.map(str::to_owned);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["nightmare"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if works { 750 } else { 1000 }
        );
    }
}

#[test]
fn attract_can_prevent_an_opposite_gender_targets_action_without_spending_its_pp() {
    use wilds_battle_engine::Gender;
    for (dice, blocked) in [(0.25, true), (0.75, false)] {
        let mut user = mon("User", &["attract"]);
        user.gender = Gender::Male;
        user.speed = 200;
        let mut foe = mon("Foe", &["tackle"]);
        foe.gender = Gender::Female;
        foe.substitute_hp = 50;
        let mut b = game(user, vec![foe], dice);
        let pp = b.participants(Side::Foes)[0].move_pp["tackle"];
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].condition("attract").is_some());
        assert_eq!(b.participants(Side::Allies)[0].hp == 1000, blocked);
        assert_eq!(
            b.participants(Side::Foes)[0].move_pp["tackle"],
            pp - u8::from(!blocked)
        );
    }
}

#[test]
fn attract_respects_gender_oblivious_and_protect() {
    use wilds_battle_engine::Gender;
    for (gender, ability, response) in [
        (Gender::Male, None, "splash"),
        (Gender::Genderless, None, "splash"),
        (Gender::Female, Some("oblivious"), "splash"),
        (Gender::Female, None, "protect"),
    ] {
        let mut user = mon("User", &["attract"]);
        user.gender = Gender::Male;
        let mut foe = mon("Foe", &[response]);
        foe.gender = gender;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].condition("attract").is_none());
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn attract_ends_when_its_source_switches_out() {
    use wilds_battle_engine::Gender;
    let mut user = mon("User", &["attract"]);
    user.gender = Gender::Male;
    user.speed = 200;
    let mut foe = mon("Foe", &["tackle"]);
    foe.gender = Gender::Female;
    let mut b = Battle::with_rng(
        Fixed(0.25),
        [vec![user, mon("Bench", &["splash"])], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Allies {
                    matches!(c, Choice::Switch { .. })
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert!(b.participants(Side::Foes)[0].condition("attract").is_none());
    assert!(b.participants(Side::Allies)[1].hp < 1000);
}

#[test]
fn taunt_interrupts_a_selected_status_move_and_restricts_the_next_three_actions() {
    use wilds_battle_engine::Stat;
    for fast in [true, false] {
        let mut user = mon("User", &["taunt", "splash"]);
        user.speed = if fast { 200 } else { 50 };
        let mut foe = mon("Foe", &["growl", "tackle"]);
        foe.substitute_hp = 50;
        let mut b = game(user, vec![foe], 0.5);
        let pp = b.participants(Side::Foes)[0].move_pp["growl"];
        turn_with(&mut b, |p| choose_move(p, "taunt", "growl"));
        assert_eq!(
            b.participants(Side::Foes)[0].move_pp["growl"],
            pp - u8::from(!fast)
        );
        assert_eq!(
            b.participants(Side::Allies)[0].stage(Stat::Attack),
            if fast { 0 } else { -1 }
        );
        let remaining = if fast { 2 } else { 3 };
        assert_eq!(
            b.participants(Side::Foes)[0]
                .condition("taunt")
                .unwrap()
                .turns_left,
            Some(remaining)
        );
        for _ in 0..remaining {
            turn_with(&mut b, |p| {
                if p.actor.side == Side::Foes {
                    assert!(
                        !p.choices
                            .iter()
                            .any(|c| matches!(c,Choice::UseMove{move_id,..} if move_id=="growl"))
                    );
                }
                choose_move(p, "splash", "tackle")
            });
        }
        assert!(b.participants(Side::Foes)[0].condition("taunt").is_none());
        turn_with(&mut b, |p| choose_move(p, "splash", "growl"));
        assert_eq!(
            b.participants(Side::Foes)[0].move_pp["growl"],
            pp - u8::from(!fast) - 1
        );
    }
}

#[test]
fn taunt_is_blocked_by_oblivious_and_protect() {
    for (ability, response) in [(Some("oblivious"), "splash"), (None, "protect")] {
        let mut foe = mon("Foe", &[response]);
        foe.ability = ability.map(str::to_owned);
        let mut b = game(mon("User", &["taunt"]), vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].condition("taunt").is_none());
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn disable_blocks_the_last_move_including_a_pending_repeat_and_expires_after_four_actions() {
    for fast in [true, false] {
        let mut user = mon("User", &["disable", "splash"]);
        user.speed = if fast { 200 } else { 50 };
        let mut foe = mon("Foe", &["tackle", "splash"]);
        foe.substitute_hp = 50;
        let mut b = game(user, vec![foe], 0.5);
        turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
        let pp = b.participants(Side::Foes)[0].move_pp["tackle"];
        turn_with(&mut b, |p| choose_move(p, "disable", "tackle"));
        assert_eq!(
            b.participants(Side::Foes)[0].move_pp["tackle"],
            pp - u8::from(!fast)
        );
        let remaining = if fast { 3 } else { 4 };
        assert_eq!(
            b.participants(Side::Foes)[0]
                .condition("disable")
                .unwrap()
                .turns_left,
            Some(remaining)
        );
        for _ in 0..remaining {
            turn_with(&mut b, |p| {
                if p.actor.side == Side::Foes {
                    assert!(
                        !p.choices
                            .iter()
                            .any(|c| matches!(c,Choice::UseMove{move_id,..} if move_id=="tackle"))
                    );
                }
                choose_move(p, "splash", "splash")
            });
        }
        assert!(b.participants(Side::Foes)[0].condition("disable").is_none());
        turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
    }
}

#[test]
fn disable_fails_without_a_previous_move_or_when_that_move_has_no_pp() {
    let mut user = mon("User", &["disable", "splash"]);
    user.speed = 200;
    let mut b = game(user.clone(), vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    let mut foe = mon("Foe", &["tackle", "splash"]);
    foe.move_pp.insert("tackle".into(), 1);
    let mut b = game(user, vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
    turn_with(&mut b, |p| choose_move(p, "disable", "splash"));
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert!(b.participants(Side::Foes)[0].condition("disable").is_none());
}

#[test]
fn torment_restricts_future_choices_without_canceling_an_already_selected_repeat() {
    let mut user = mon("User", &["torment", "splash"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["tackle", "splash"]);
    foe.substitute_hp = 50;
    let mut b = game(user, vec![foe], 0.5);
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
    let pp = b.participants(Side::Foes)[0].move_pp["tackle"];
    turn_with(&mut b, |p| choose_move(p, "torment", "tackle"));
    assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], pp - 1);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Foes {
            assert!(
                !p.choices
                    .iter()
                    .any(|c| matches!(c,Choice::UseMove{move_id,..} if move_id=="tackle"))
            );
        }
        choose_move(p, "splash", "splash")
    });
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
}

#[test]
fn torment_allows_struggle_when_the_only_learned_move_cannot_repeat() {
    let mut user = mon("User", &["torment", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["tackle"])], 0.5);
    turn_with(&mut b, |p| choose_move(p, "torment", "tackle"));
    turn_with(&mut b, |p| choose_move(p, "splash", "struggle"));
    assert!(b.participants(Side::Foes)[0].hp < 1000);
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
}

#[test]
fn imprison_blocks_shared_moves_immediately_and_unseals_them_when_the_user_leaves() {
    let mut user = mon("User", &["imprison", "tackle", "splash"]);
    user.speed = 200;
    let foe = mon("Foe", &["tackle", "splash", "ember"]);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, mon("Bench", &["splash"])], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    let pp = b.participants(Side::Foes)[0].move_pp["tackle"];
    turn_with(&mut b, |p| choose_move(p, "imprison", "tackle"));
    assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], pp);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies {
            p.choices
                .iter()
                .find(|c| matches!(c, Choice::Switch { .. }))
                .unwrap()
                .id()
        } else {
            assert!(!p.choices.iter().any(|c| matches!(c,Choice::UseMove{move_id,..} if move_id=="tackle" || move_id=="splash")));
            choose_move(p, "splash", "ember")
        }
    });
    turn_with(&mut b, |p| choose_move(p, "splash", "tackle"));
    assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], pp - 1);
}

#[test]
fn imprison_does_not_seal_allies_and_leaves_struggle_available() {
    let mut user = mon("User", &["imprison", "tackle", "splash"]);
    user.speed = 200;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![user, mon("Partner", &["splash"])],
            vec![mon("Foe", &["tackle"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn_with(&mut b, |p| {
        choose_move(
            p,
            if p.actor.index == 0 {
                "imprison"
            } else {
                "splash"
            },
            "tackle",
        )
    });
    turn_with(&mut b, |p| choose_move(p, "splash", "struggle"));
    assert!(b.participants(Side::Foes)[0].hp < 1000);
}

#[test]
fn curse_boosts_non_ghost_users_and_offers_a_self_target() {
    use wilds_battle_engine::Stat;
    let mut b = game(mon("User", &["curse"]), vec![mon("Foe", &["splash"])], 0.5);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies {
            assert!(
                p.choices
                    .iter()
                    .all(|c| matches!(c,Choice::UseMove{target,..} if *target==p.actor))
            );
        }
        p.choices[0].id()
    });
    let user = &b.participants(Side::Allies)[0];
    assert_eq!(user.stage(Stat::Attack), 1);
    assert_eq!(user.stage(Stat::Defense), 1);
    assert_eq!(user.stage(Stat::Speed), -1);
    assert_eq!(user.hp, 1000);
}

#[test]
fn ghost_curse_pays_half_max_hp_once_and_drains_through_protect_and_substitute() {
    let mut user = mon("User", &["curse"]);
    user.types = vec![PokemonType::Ghost];
    let mut foe = mon("Foe", &["protect"]);
    foe.substitute_hp = 50;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 500);
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
    assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 50);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 500);
    assert_eq!(b.participants(Side::Foes)[0].hp, 500);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn curse_rechecks_its_users_type_after_an_earlier_type_change() {
    use wilds_battle_engine::Stat;
    for ghost in [true, false] {
        let mut user = mon("User", &["curse"]);
        user.speed = 50;
        if ghost {
            user.types = vec![PokemonType::Ghost];
        }
        let mut foe = mon("Foe", &[if ghost { "soak" } else { "trick-or-treat" }]);
        foe.speed = 200;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if ghost { 1000 } else { 500 }
        );
        assert_eq!(
            b.participants(Side::Allies)[0].stage(Stat::Attack),
            if ghost { 1 } else { 0 }
        );
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if ghost { 1000 } else { 750 }
        );
    }
}

fn check_sport(move_id: &str, attack: &str, kind: &str) {
    let mut user = mon("User", &[move_id, "splash"]);
    user.speed = 200;
    let foe = mon("Foe", &[attack]);
    let weak = control_damage(13, kind, "Special", foe.clone(), user.clone());
    let normal = control_damage(40, kind, "Special", foe.clone(), user.clone());
    let mut b = game(user, vec![foe], 0.5);
    let mut expected = 1000;
    for index in 0..6 {
        turn_with(&mut b, |p| {
            choose_move(p, if index < 2 { move_id } else { "splash" }, attack)
        });
        expected -= if index < 5 { weak } else { normal };
        assert_eq!(b.participants(Side::Allies)[0].hp, expected, "turn {index}");
        if index == 1 {
            assert!(b.participants(Side::Allies)[0].last_move_failed);
        }
    }
}

#[test]
fn mud_sport_reduces_electric_base_power_for_five_turns_without_refreshing() {
    check_sport("mud-sport", "thunder_shock", "Electric");
}

#[test]
fn mud_sport_persists_after_its_user_switches_out() {
    let mut user = mon("User", &["mud-sport"]);
    user.speed = 200;
    let foe = mon("Foe", &["thunder_shock"]);
    let bench = mon("Bench", &["splash"]);
    let expected = control_damage(13, "Electric", "Special", foe.clone(), bench.clone());
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, bench], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Allies {
                    matches!(c, Choice::Switch { .. })
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert_eq!(1000 - b.participants(Side::Allies)[1].hp, expected);
}

#[test]
fn water_sport_reduces_fire_base_power_for_five_turns_without_refreshing() {
    check_sport("water-sport", "ember", "Fire");
}

#[test]
fn water_sport_does_not_reduce_other_attack_types() {
    let mut user = mon("User", &["water-sport"]);
    user.speed = 200;
    let foe = mon("Foe", &["thunder_shock"]);
    let expected = control_damage(40, "Electric", "Special", foe.clone(), user.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(1000 - b.participants(Side::Allies)[0].hp, expected);
}

#[test]
fn helping_hand_stacks_base_power_boosts_for_one_turn_through_substitute() {
    for helpers in [1, 2] {
        let mut helper = mon("Helper", &["helping-hand", "splash"]);
        helper.speed = 200;
        let mut attacker = mon("Attacker", &["tackle"]);
        attacker.substitute_hp = 50;
        let foe = mon("Foe", &["splash"]);
        let boosted = control_damage(
            if helpers == 1 { 60 } else { 90 },
            "Normal",
            "Physical",
            attacker.clone(),
            foe.clone(),
        );
        let normal = control_damage(40, "Normal", "Physical", attacker.clone(), foe.clone());
        let mut allies = vec![helper.clone(), attacker];
        if helpers == 2 {
            allies.push(helper);
        }
        let mut b = Battle::with_rng(
            Fixed(0.5),
            [allies, vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn_with(&mut b, |p| {
            if p.actor.side == Side::Allies && p.actor.index != 1 {
                p.choices.iter().find(|c| matches!(c,Choice::UseMove{move_id,target,..} if move_id=="helping-hand" && target.index==1)).unwrap().id()
            } else {
                p.choices[0].id()
            }
        });
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, boosted);
        turn_with(&mut b, |p| {
            choose_move(
                p,
                if p.actor.index == 1 {
                    "tackle"
                } else {
                    "splash"
                },
                "splash",
            )
        });
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, boosted + normal);
    }
}

#[test]
fn helping_hand_fails_in_singles_and_does_not_boost_fixed_damage() {
    let mut b = game(
        mon("User", &["helping-hand"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.participants(Side::Allies)[0].move_pp["helping-hand"], 19);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![
                mon("Helper", &["helping-hand"]),
                mon("Attacker", &["dragon_rage"]),
            ],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 960);
}

#[test]
fn hold_hands_succeeds_with_an_adjacent_ally_through_protect_and_substitute() {
    let mut ally = mon("Ally", &["protect"]);
    ally.substitute_hp = 50;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![mon("User", &["hold-hands"]), ally],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn(&mut b);
    assert!(!b.participants(Side::Allies)[0].last_move_failed);
    assert!(
        events
            .iter()
            .any(|e| matches!(e,BattleEvent::Message(m) if m=="User and Ally held hands!"))
    );
    assert_eq!(b.participants(Side::Allies)[1].substitute_hp, 50);
    assert!(b.participants(Side::Allies).iter().all(|p| p.hp == 1000));
}
#[test]
fn hold_hands_fails_without_an_ally_after_spending_pp() {
    let mut b = game(
        mon("User", &["hold-hands"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.participants(Side::Allies)[0].move_pp["hold-hands"], 39);
}

#[test]
fn happy_hour_doubles_prior_and_future_pay_day_money_without_stacking() {
    let mut user = mon("User", &["happy-hour", "pay-day"]);
    user.level = 10;
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    turn_with(&mut b, |p| choose_move(p, "pay-day", "splash"));
    assert_eq!(b.payout(Side::Allies), 50);
    for _ in 0..2 {
        turn_with(&mut b, |p| choose_move(p, "happy-hour", "splash"));
        assert_eq!(b.payout(Side::Allies), 100);
        assert_eq!(b.payout_multiplier(Side::Allies), 2);
        assert_eq!(b.payout_multiplier(Side::Foes), 1);
        assert!(!b.participants(Side::Allies)[0].last_move_failed);
    }
    turn_with(&mut b, |p| choose_move(p, "pay-day", "splash"));
    assert_eq!(b.payout(Side::Allies), 200);
}

#[test]
fn happy_hours_multiplier_persists_when_its_user_switches_out() {
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [
            vec![mon("User", &["happy-hour"]), mon("Bench", &["splash"])],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    turn_with(&mut b, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Allies {
                    matches!(c, Choice::Switch { .. })
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert_eq!(b.payout_multiplier(Side::Allies), 2);
}

#[test]
fn octolock_repeatedly_drops_both_defenses_respects_immunities_and_releases_with_source() {
    use wilds_battle_engine::Stat;
    for (ability, kind, expected) in [
        (None, PokemonType::Normal, -2),
        (Some("contrary"), PokemonType::Normal, 2),
        (Some("clear-body"), PokemonType::Normal, 0),
        (None, PokemonType::Ghost, 0),
    ] {
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = ability.map(str::to_owned);
        foe.types = vec![kind];
        let mut b = Battle::with_rng(
            Fixed(0.5),
            [
                vec![
                    mon("User", &["octolock", "splash"]),
                    mon("Bench", &["splash"]),
                ],
                vec![foe, mon("Foe bench", &["splash"])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
        assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Defense), expected);
        assert_eq!(
            b.participants(Side::Foes)[0].stage(Stat::SpDefense),
            expected
        );
        turn_with(&mut b, |p| {
            if p.actor.side == Side::Allies {
                p.choices
                    .iter()
                    .find(|c| matches!(c, Choice::Switch { .. }))
                    .unwrap()
                    .id()
            } else {
                choose_move(p, "splash", "splash")
            }
        });
        assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Defense), expected);
        assert!(
            !b.participants(Side::Foes)[0]
                .conditions
                .iter()
                .any(|c| c.name == "octolock")
        );
    }
    for response in ["protect", "substitute"] {
        let mut foe = mon("Foe", &[response]);
        foe.speed = 200;
        let mut b = game(mon("User", &["octolock"]), vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].conditions.is_empty());
    }
}

#[test]
fn yawn_sleeps_at_the_end_of_the_next_turn_and_cannot_be_refreshed() {
    use wilds_battle_engine::Status;
    let mut b = game(mon("User", &["yawn"]), vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].status, None);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert_eq!(b.participants(Side::Foes)[0].status, Some(Status::Asleep));
    for ability in ["insomnia", "vital-spirit", "comatose"] {
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = Some(ability.into());
        let mut b = game(mon("User", &["yawn"]), vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].conditions.is_empty());
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn yawn_is_cancelled_by_switching_and_rechecks_safeguard_at_expiry() {
    for switch in [false, true] {
        let mut b = game(
            mon("User", &["yawn", "splash"]),
            vec![
                mon("Foe", &["splash", "safeguard"]),
                mon("Bench", &["splash"]),
            ],
            0.5,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        turn_with(&mut b, |p| {
            if p.actor.side == Side::Foes && switch {
                p.choices
                    .iter()
                    .find(|c| matches!(c, Choice::Switch { .. }))
                    .unwrap()
                    .id()
            } else {
                choose_move(p, "splash", "safeguard")
            }
        });
        assert_eq!(b.participants(Side::Foes)[0].status, None);
        assert_eq!(b.participants(Side::Foes)[1].status, None);
        assert!(b.participants(Side::Foes)[0].conditions.is_empty());
    }
}

#[test]
fn perish_song_bypasses_protect_and_substitute_counts_three_turns_and_respects_soundproof() {
    let mut user = mon("User", &["perish-song", "splash"]);
    user.ability = Some("soundproof".into());
    let mut foe = mon("Foe", &["protect", "splash"]);
    foe.substitute_hp = 100;
    let mut b = game(user, vec![foe], 0.5);
    let events = turn(&mut b);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m.contains("count fell to 3")))
    );
    assert!(b.participants(Side::Allies)[0].conditions.is_empty());
    for _ in 0..2 {
        turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
    turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 0);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
}

#[test]
fn perish_song_does_not_refresh_cancels_on_switch_and_resolves_faster_pokemon_first() {
    let mut b = game(
        mon("User", &["perish-song", "splash"]),
        vec![mon("Foe", &["splash"]), mon("Bench", &["splash"])],
        0.5,
    );
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Foes {
            p.choices
                .iter()
                .find(|c| matches!(c, Choice::Switch { .. }))
                .unwrap()
                .id()
        } else {
            choose_move(p, "splash", "splash")
        }
    });
    turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
    assert_eq!(b.participants(Side::Allies)[0].hp, 0);
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    for fast_side in [Side::Allies, Side::Foes] {
        let mut user = mon("User", &["perish-song", "splash"]);
        let mut foe = mon("Foe", &["splash"]);
        if fast_side == Side::Allies {
            user.speed = 200;
        } else {
            foe.speed = 200;
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        let mut events = vec![];
        for _ in 0..3 {
            events = turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BattleEvent::End {winner: Some(w)} if *w != fast_side))
        );
    }
}

#[test]
fn trick_room_reverses_speed_for_five_turns_preserves_priority_and_toggles_off() {
    let mut user = mon("User", &["trick-room", "splash", "quick-attack"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["celebrate"])], 0.5);
    turn(&mut b);
    assert_eq!(
        used_names(&turn_with(&mut b, |p| choose_move(
            p,
            "splash",
            "celebrate"
        ))),
        ["Foe", "User"]
    );
    assert_eq!(
        used_names(&turn_with(&mut b, |p| choose_move(
            p,
            "quick-attack",
            "celebrate"
        ))),
        ["User", "Foe"]
    );
    for _ in 0..2 {
        turn_with(&mut b, |p| choose_move(p, "splash", "celebrate"));
    }
    assert_eq!(
        used_names(&turn_with(&mut b, |p| choose_move(
            p,
            "splash",
            "celebrate"
        ))),
        ["User", "Foe"]
    );
    turn(&mut b);
    turn(&mut b);
    assert_eq!(
        used_names(&turn_with(&mut b, |p| choose_move(
            p,
            "splash",
            "celebrate"
        ))),
        ["User", "Foe"]
    );
}

#[test]
fn trick_room_updates_remaining_equal_priority_actions_without_changing_damage_speed() {
    let mut fast = mon("Fast", &["trick-room"]);
    fast.speed = 300;
    let mut mid = mon("Mid", &["trick-room"]);
    mid.speed = 200;
    let slow = mon("Slow", &["trick-room"]);
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![fast, mid], vec![slow]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn(&mut b);
    let users: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let BattleEvent::Message(m) = e {
                m.split_once(" used ").map(|(who, _)| who)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(users, ["Fast", "Slow", "Mid"]);
}

#[test]
fn wonder_room_swaps_raw_defenses_without_swapping_boosts_and_toggles() {
    use wilds_battle_engine::Stat;
    let user = mon("User", &["wonder-room", "pound"]);
    let mut foe = mon("Foe", &["splash"]);
    foe.defense = 400;
    foe.sp_defense = 100;
    foe.change_stage(Stat::Defense, 2);
    foe.change_stage(Stat::SpDefense, -1);
    let normal = control_damage(40, "Normal", "Physical", user.clone(), foe.clone());
    let mut swapped = foe.clone();
    swapped.defense = 100;
    swapped.sp_defense = 400;
    let room = control_damage(40, "Normal", "Physical", user.clone(), swapped);
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "pound", "splash"));
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, room);
    assert_eq!(b.participants(Side::Foes)[0].defense, 400);
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "pound", "splash"));
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, room + normal);
}

#[test]
fn wonder_room_body_press_uses_original_defense_with_special_defense_boosts() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["wonder-room", "body-press"]);
    user.defense = 400;
    user.sp_defense = 100;
    user.change_stage(Stat::Defense, 2);
    user.change_stage(Stat::SpDefense, -1);
    let foe = mon("Foe", &["splash"]);
    let mut equivalent = user.clone();
    equivalent.attack = 400;
    equivalent.change_stage(Stat::Attack, -1);
    let expected = control_damage(80, "Fighting", "Physical", equivalent, foe.clone());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "body-press", "splash"));
    assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
}

#[test]
fn ally_switch_moves_slots_redirects_queued_attacks_and_preserves_volatile_state() {
    use wilds_battle_engine::{Stat, Status};
    let mut user = mon("User", &["ally-switch"]);
    user.status = Some(Status::Poisoned);
    user.change_stage(Stat::Attack, 2);
    let mut ally = mon("Ally", &["splash"]);
    ally.speed = 150;
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user, ally], vec![mon("Foe", &["pound"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn_with(&mut b, |p| {
        p.choices.iter().find(|c| matches!(c,Choice::UseMove {target,..} if p.actor.side == Side::Allies || target.index == 0)).unwrap().id()
    });
    assert_eq!(b.active_indices(Side::Allies), &[1, 0]);
    assert_eq!(b.participants(Side::Allies)[0].hp, 875); // one poison tick, no direct hit
    assert!(b.participants(Side::Allies)[1].hp < 1000);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 2);
    assert_eq!(used_names(&events), ["User", "Ally", "Foe"]);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::PositionsSwapped { .. }))
    );
    // Gen 8 has no consecutive-use failure chance. Aim at the user's new slot again.
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c| matches!(c,Choice::UseMove {target,..} if p.actor.side == Side::Allies || target.index == 0)).unwrap().id()
    });
    assert_eq!(b.active_indices(Side::Allies), &[0, 1]);
    assert_eq!(b.participants(Side::Allies)[0].hp, 750);
}

#[test]
fn ally_switch_fails_in_singles_and_the_middle_of_triples_but_swaps_the_edges() {
    let mut b = game(
        mon("User", &["ally-switch"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for index in [0, 1, 2] {
        let mut party = vec![
            mon("Left", &["splash"]),
            mon("Middle", &["splash"]),
            mon("Right", &["splash"]),
        ];
        party[index].moves = vec!["ally-switch".into()];
        let mut b = Battle::with_rng(
            Fixed(0.5),
            [party, vec![mon("Foe", &["splash"])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        turn(&mut b);
        assert_eq!(
            b.active_indices(Side::Allies),
            if index == 1 { &[0, 1, 2] } else { &[2, 1, 0] }
        );
        assert_eq!(
            b.participants(Side::Allies)[index].last_move_failed,
            index == 1
        );
    }
}

#[test]
fn ion_deluge_converts_normal_attacks_status_and_fixed_damage_for_one_turn() {
    use wilds_battle_engine::Status;
    for move_id in ["pound", "super-fang", "glare"] {
        let mut user = mon("User", &["ion-deluge", "splash"]);
        user.types = vec![PokemonType::Ground];
        let mut b = game(user, vec![mon("Foe", &[move_id])], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
        assert_eq!(b.participants(Side::Allies)[0].status, None);
        turn_with(&mut b, |p| choose_move(p, "splash", move_id));
        if move_id == "glare" {
            assert_eq!(
                b.participants(Side::Allies)[0].status,
                Some(Status::Paralyzed)
            );
        } else {
            assert!(b.participants(Side::Allies)[0].hp < 1000);
        }
    }
}

#[test]
fn electrify_converts_pending_moves_including_dynamic_types_and_expires_after_the_turn() {
    for attack in ["ember", "weather-ball", "super-fang"] {
        let mut user = mon("User", &["electrify", "splash"]);
        user.speed = 200;
        user.types = vec![PokemonType::Ground];
        let mut b = game(user, vec![mon("Foe", &[attack])], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
        assert!(!b.participants(Side::Allies)[0].last_move_failed);
        turn_with(&mut b, |p| choose_move(p, "splash", attack));
        assert!(b.participants(Side::Allies)[0].hp < 1000);
    }
    let mut user = mon("User", &["electrify"]);
    user.speed = 50;
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for response in ["protect", "substitute"] {
        let mut user = mon("User", &["electrify"]);
        user.speed = 50;
        let mut b = game(user, vec![mon("Foe", &[response])], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn magic_coat_reflects_status_once_even_against_mold_breaker_and_expires() {
    use wilds_battle_engine::Status;
    for ability in [None, Some("mold-breaker"), Some("magic-bounce")] {
        let mut foe = mon("Foe", &["toxic"]);
        foe.ability = ability.map(str::to_owned);
        let mut b = game(mon("User", &["magic-coat", "splash"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].status, None);
        assert_eq!(
            b.participants(Side::Foes)[0].status,
            Some(Status::BadlyPoisoned)
        );
        assert_eq!(b.participants(Side::Allies)[0].move_pp["magic-coat"], 14);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["toxic"], 9);
        turn_with(&mut b, |p| choose_move(p, "splash", "toxic"));
        assert_eq!(
            b.participants(Side::Allies)[0].status,
            Some(Status::BadlyPoisoned)
        );
    }
}

#[test]
fn magic_coat_reflects_side_hazards_without_stopping_attacks() {
    let mut b = game(
        mon("User", &["magic-coat"]),
        vec![mon("Foe", &["spikes"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.side_conditions(Side::Allies).is_empty());
    assert!(
        b.side_conditions(Side::Foes)
            .iter()
            .any(|c| c.name == "spikes")
    );
    let mut b = game(
        mon("User", &["magic-coat"]),
        vec![mon("Foe", &["pound"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].hp < 1000);
}

#[test]
fn powder_cancels_fire_moves_after_spending_pp_and_bypasses_substitute() {
    for ability in [None, Some("magic-guard")] {
        let mut foe = mon("Foe", &["ember"]);
        foe.substitute_hp = 100;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(mon("User", &["powder"]), vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if ability.is_some() { 1000 } else { 750 }
        );
        assert_eq!(b.participants(Side::Foes)[0].substitute_hp, 100);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["ember"], 24);
        assert!(b.participants(Side::Foes)[0].last_move_failed);
    }
    for (kind, ability) in [
        (PokemonType::Grass, None),
        (PokemonType::Normal, Some("overcoat")),
    ] {
        let mut foe = mon("Foe", &["ember"]);
        foe.types = vec![kind];
        foe.ability = ability.map(str::to_owned);
        let mut b = game(mon("User", &["powder"]), vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].hp < 1000);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
}

#[test]
fn powder_uses_weather_balls_actual_type_and_expires_after_the_turn() {
    let mut b = game(
        mon("User", &["sunny_day", "powder", "splash"]),
        vec![mon("Foe", &["splash", "weather-ball"])],
        0.5,
    );
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "powder", "weather-ball"));
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
    turn_with(&mut b, |p| choose_move(p, "splash", "weather-ball"));
    assert!(b.participants(Side::Allies)[0].hp < 1000);
    assert_eq!(b.participants(Side::Foes)[0].hp, 750);
}

#[test]
fn court_change_exchanges_screens_and_hazard_layers_without_resetting_durations() {
    let mut b = game(
        mon("User", &["reflect", "spikes", "court-change"]),
        vec![mon("Foe", &["light-screen", "spikes", "splash"])],
        0.5,
    );
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "spikes", "spikes"));
    turn_with(&mut b, |p| choose_move(p, "spikes", "splash"));
    turn_with(&mut b, |p| choose_move(p, "court-change", "splash"));
    let a = b.side_conditions(Side::Allies);
    let f = b.side_conditions(Side::Foes);
    assert_eq!(a.iter().find(|c| c.name == "spikes").unwrap().value, 2);
    assert_eq!(f.iter().find(|c| c.name == "spikes").unwrap().value, 1);
    assert_eq!(
        a.iter()
            .find(|c| c.name == "light_screen")
            .unwrap()
            .turns_left,
        Some(1)
    );
    assert_eq!(
        f.iter().find(|c| c.name == "reflect").unwrap().turns_left,
        Some(1)
    );
    assert!(!a.iter().any(|c| c.name == "reflect"));
    assert!(!f.iter().any(|c| c.name == "light_screen"));
    let mut b = game(
        mon("User", &["court-change"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

fn veil_damage(attack: &str, ability: Option<&str>, screen: &str, veil: bool) -> u16 {
    let mut user = mon(
        "User",
        &["hail", "aurora-veil", "reflect", "light-screen", "splash"],
    );
    user.speed = 200;
    user.types = vec![PokemonType::Ice];
    let mut foe = mon("Foe", &["splash", attack]);
    foe.types = vec![PokemonType::Ice];
    foe.ability = ability.map(str::to_owned);
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    turn_with(&mut b, |p| {
        choose_move(p, if veil { "aurora-veil" } else { "splash" }, "splash")
    });
    turn_with(&mut b, |p| choose_move(p, screen, attack));
    1000 - b.participants(Side::Allies)[0].hp
}

#[test]
fn aurora_veil_requires_hail_protects_both_categories_and_never_stacks_with_screens() {
    for (attack, screen) in [("pound", "reflect"), ("psychic", "light-screen")] {
        let normal = veil_damage(attack, None, "splash", false);
        let veil = veil_damage(attack, None, "splash", true);
        assert!(veil < normal);
        assert_eq!(veil_damage(attack, None, screen, true), veil);
        assert_eq!(
            veil_damage(attack, Some("infiltrator"), screen, true),
            normal
        );
    }
    assert_eq!(
        veil_damage("frost-breath", None, "splash", true),
        veil_damage("frost-breath", None, "splash", false)
    );
    let mut b = game(
        mon("User", &["aurora-veil"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert!(b.side_conditions(Side::Allies).is_empty());
}

#[test]
fn aurora_veil_keeps_working_after_hail_ends_but_expires_after_five_turns() {
    let mut b = game(
        mon("User", &["hail", "aurora-veil", "splash"]),
        vec![mon("Foe", &["splash", "rain-dance"])],
        0.5,
    );
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "aurora-veil", "splash"));
    turn_with(&mut b, |p| choose_move(p, "splash", "rain-dance"));
    assert!(
        b.side_conditions(Side::Allies)
            .iter()
            .any(|c| c.name == "aurora_veil")
    );
    for _ in 0..3 {
        turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
    }
    assert!(b.side_conditions(Side::Allies).is_empty());
}

#[test]
fn defog_clears_both_sides_hazards_and_only_targets_screens_while_substitute_blocks_the_drop() {
    use wilds_battle_engine::Stat;
    for sub in [0, 100] {
        let mut foe = mon("Foe", &["reflect", "spikes", "splash"]);
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &["reflect", "spikes", "defog"]), vec![foe], 0.5);
        turn(&mut b);
        turn_with(&mut b, |p| choose_move(p, "spikes", "spikes"));
        turn_with(&mut b, |p| choose_move(p, "defog", "splash"));
        assert!(b.side_conditions(Side::Foes).is_empty());
        assert_eq!(b.side_conditions(Side::Allies).len(), 1);
        assert_eq!(b.side_conditions(Side::Allies)[0].name, "reflect");
        assert_eq!(
            b.participants(Side::Foes)[0].stage(Stat::Evasion),
            if sub == 0 { -1 } else { 0 }
        );
    }
    let mut b = game(mon("User", &["defog"]), vec![mon("Foe", &["protect"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Evasion), 0);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

#[test]
fn heal_block_interrupts_queued_healing_and_disables_healing_and_drain_moves_for_five_turns() {
    let mut user = mon("User", &["heal-block", "splash"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["recover", "absorb", "splash"]);
    foe.hp = 600;
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 600);
    assert_eq!(b.participants(Side::Foes)[0].move_pp["recover"], 5);
    for _ in 0..4 {
        turn_with(&mut b, |p| {
            if p.actor.side == Side::Foes {
                assert!(p.choices.iter().all(|c| !matches!(c,Choice::UseMove {move_id,..} if move_id=="recover" || move_id=="absorb")));
            }
            choose_move(p, "splash", "splash")
        });
    }
    turn_with(&mut b, |p| choose_move(p, "splash", "recover"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
}

#[test]
fn heal_block_prevents_passive_recovery_and_retains_healing_berries_until_expiry() {
    let mut user = mon("User", &["splash", "heal-block", "super-fang"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["aqua-ring", "splash"]);
    foe.hp = 600;
    foe.item = Some("sitrus-berry".into());
    let mut b = game(user, vec![foe], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 662);
    turn_with(&mut b, |p| choose_move(p, "heal-block", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 662);
    turn_with(&mut b, |p| choose_move(p, "super-fang", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 331);
    assert!(b.participants(Side::Foes)[0].item.is_some());
    for _ in 0..3 {
        turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
    }
    assert!(b.participants(Side::Foes)[0].item.is_none());
    assert_eq!(b.participants(Side::Foes)[0].hp, 581);
}

#[test]
fn heal_block_does_not_prevent_pain_split_or_regenerator_on_switch_out() {
    let mut user = mon("User", &["heal-block", "splash"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["pain-split", "splash"]);
    foe.hp = 200;
    foe.ability = Some("regenerator".into());
    let mut b = game(user, vec![foe, mon("Bench", &["splash"])], 0.5);
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b);
    assert_eq!(b.participants(Side::Foes)[0].hp, 600);
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Foes {
            p.choices
                .iter()
                .find(|c| matches!(c, Choice::Switch { .. }))
                .unwrap()
                .id()
        } else {
            choose_move(p, "splash", "splash")
        }
    });
    assert_eq!(b.participants(Side::Foes)[0].hp, 933);
}

#[test]
fn magnet_rise_grants_five_turn_ground_immunity_and_gravity_removes_it() {
    let mut user = mon("User", &["magnet-rise", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["earthquake", "gravity"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    for _ in 0..4 {
        turn_with(&mut b, |p| choose_move(p, "splash", "earthquake"));
    }
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    turn_with(&mut b, |p| choose_move(p, "splash", "earthquake"));
    assert!(b.participants(Side::Allies)[0].hp < 1000);
    turn_with(&mut b, |p| choose_move(p, "magnet-rise", "gravity"));
    assert!(b.participants(Side::Allies)[0].conditions.is_empty());
    turn_with(&mut b, |p| {
        if p.actor.side == Side::Allies {
            assert!(
                !p.choices
                    .iter()
                    .any(|c| matches!(c,Choice::UseMove{move_id,..} if move_id=="magnet-rise"))
            );
        }
        choose_move(p, "splash", "earthquake")
    });
}

#[test]
fn magnet_rise_is_removed_by_grounding_attacks_and_thousand_arrows_hits_levitate() {
    for attack in ["smack-down", "thousand-arrows"] {
        let mut user = mon("User", &["magnet-rise", "splash"]);
        user.speed = 200;
        let mut b = game(user, vec![mon("Foe", &[attack, "earthquake"])], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Allies)[0].hp < 1000);
        assert!(
            !b.participants(Side::Allies)[0]
                .conditions
                .iter()
                .any(|c| c.name == "magnet_rise")
        );
        let before = b.participants(Side::Allies)[0].hp;
        turn_with(&mut b, |p| choose_move(p, "splash", "earthquake"));
        assert!(b.participants(Side::Allies)[0].hp < before);
    }
    let mut user = mon("User", &["splash"]);
    user.ability = Some("levitate".into());
    let mut b = game(user, vec![mon("Foe", &["thousand-arrows"])], 0.5);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].hp < 1000);
}

#[test]
fn telekinesis_guarantees_ordinary_hits_for_three_turns_but_not_ohko_hits() {
    let mut user = mon("User", &["telekinesis", "fire-blast", "sheer-cold"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["splash"])], 0.95);
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "fire-blast", "splash"));
    let after = b.participants(Side::Foes)[0].hp;
    assert!(after < 1000);
    turn_with(&mut b, |p| choose_move(p, "sheer-cold", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, after);
    assert!(
        !b.participants(Side::Foes)[0]
            .conditions
            .iter()
            .any(|c| c.name == "telekinesis")
    );
    turn_with(&mut b, |p| choose_move(p, "fire-blast", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, after);
}

#[test]
fn telekinesis_grants_ground_immunity_but_fails_on_grounded_species_and_protect() {
    let mut b = game(
        mon("User", &["telekinesis", "earthquake"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "earthquake", "splash"));
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    for id in [50, 51, 769, 770, 94] {
        let mut foe = mon("Foe", &["splash"]);
        foe.species_id = id;
        if id == 94 {
            foe.form = Some("mega".into());
        }
        let mut b = game(mon("User", &["telekinesis"]), vec![foe], 0.5);
        turn(&mut b);
        assert!(b.participants(Side::Foes)[0].conditions.is_empty());
        assert!(b.participants(Side::Allies)[0].last_move_failed);
    }
    let mut b = game(
        mon("User", &["telekinesis"]),
        vec![mon("Foe", &["protect"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Foes)[0].conditions.is_empty());
}

#[test]
fn electric_terrain_boosts_grounded_electric_power_and_defog_removes_it() {
    for flying in [false, true] {
        let mut user = mon("User", &["electric-terrain", "thunder_shock"]);
        user.types = vec![PokemonType::Electric];
        if flying {
            user.types.push(PokemonType::Flying);
        }
        let foe = mon("Foe", &["splash", "defog"]);
        let expected = control_damage(
            if flying { 40 } else { 52 },
            "Electric",
            "Special",
            user.clone(),
            foe.clone(),
        );
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        turn_with(&mut b, |p| choose_move(p, "thunder_shock", "splash"));
        assert_eq!(1000 - b.participants(Side::Foes)[0].hp, expected);
        turn_with(&mut b, |p| choose_move(p, "electric-terrain", "defog"));
        assert!(
            !b.field_conditions()
                .iter()
                .any(|c| c.name == "electric_terrain")
        );
    }
}

#[test]
fn electric_terrain_prevents_sleep_and_rest_only_on_grounded_pokemon_and_expires() {
    use wilds_battle_engine::Status;
    for flying in [false, true] {
        let mut user = mon("User", &["electric-terrain", "splash"]);
        user.speed = 200;
        let mut foe = mon("Foe", &["rest", "splash"]);
        foe.hp = 600;
        if flying {
            foe.types = vec![PokemonType::Flying];
        }
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].status,
            if flying { Some(Status::Asleep) } else { None }
        );
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if flying { 1000 } else { 600 }
        );
        for _ in 0..4 {
            turn_with(&mut b, |p| choose_move(p, "splash", "splash"));
        }
        assert!(
            !b.field_conditions()
                .iter()
                .any(|c| c.name == "electric_terrain")
        );
    }
    let mut user = mon("User", &["electric-terrain"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["spore"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].status, None);
}

#[test]
fn misty_terrain_prevents_status_and_confusion_for_grounded_pokemon() {
    for attack in ["toxic", "confuse_ray"] {
        for flying in [false, true] {
            let mut user = mon("User", &["misty-terrain"]);
            user.speed = 200;
            if flying {
                user.types = vec![PokemonType::Flying];
            }
            let mut b = game(user, vec![mon("Foe", &[attack])], 0.5);
            turn(&mut b);
            let user = &b.participants(Side::Allies)[0];
            if attack == "toxic" {
                assert_eq!(user.status.is_some(), flying);
            } else {
                assert_eq!(user.confused_turns.is_some(), flying);
            }
        }
    }
    let mut b = game(
        mon("User", &["misty-terrain", "outrage"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    for _ in 0..2 {
        turn_with(&mut b, |p| choose_move(p, "outrage", "splash"));
    }
    assert!(b.participants(Side::Allies)[0].confused_turns.is_none());
}

#[test]
fn misty_terrain_halves_dragon_damage_to_grounded_targets_and_replaces_other_terrain() {
    for flying in [false, true] {
        let mut user = mon("User", &["misty-terrain", "splash"]);
        user.speed = 200;
        if flying {
            user.types = vec![PokemonType::Flying];
        }
        let foe = mon("Foe", &["splash", "dragon-breath"]);
        let normal = control_damage(
            if flying { 60 } else { 30 },
            "Dragon",
            "Special",
            foe.clone(),
            user.clone(),
        );
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        turn_with(&mut b, |p| choose_move(p, "splash", "dragon-breath"));
        let damage = 1000 - b.participants(Side::Allies)[0].hp;
        assert_eq!(damage, normal);
    }
    let mut b = game(
        mon("User", &["electric-terrain", "misty-terrain"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    turn_with(&mut b, |p| choose_move(p, "misty-terrain", "splash"));
    assert_eq!(b.field_conditions().len(), 1);
    assert_eq!(b.field_conditions()[0].name, "misty_terrain");
}

#[test]
fn protective_terrain_does_not_stop_yawn_from_resolving_while_the_target_is_underground() {
    use wilds_battle_engine::Status;
    for terrain in ["electric-terrain", "misty-terrain"] {
        let mut user = mon("User", &["yawn", terrain]);
        user.speed = 200;
        let mut b = game(user, vec![mon("Foe", &["splash", "dig"])], 0.5);
        turn(&mut b);
        turn_with(&mut b, |p| choose_move(p, terrain, "dig"));
        assert_eq!(b.participants(Side::Foes)[0].status, Some(Status::Asleep));
    }
}
