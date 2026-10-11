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
