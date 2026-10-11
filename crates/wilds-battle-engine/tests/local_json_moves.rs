//! Behavior regressions for the local JSON moves outside the saved sheet.
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
        if battle.turn() > start {
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
fn sonic_boom_deals_twenty_regardless_of_stats_and_resistance() {
    for kind in [PokemonType::Normal, PokemonType::Rock] {
        let mut user = mon("User", &["sonic-boom"]);
        user.level = 1;
        user.sp_attack = 1;
        let mut foe = mon("Foe", &["splash"]);
        foe.sp_defense = 999;
        foe.types = vec![kind];
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 980);
    }
}

#[test]
fn sonic_boom_respects_immunity_accuracy_and_protect() {
    for (kind, response, dice) in [
        (PokemonType::Ghost, "splash", 0.5),
        (PokemonType::Normal, "splash", 0.95),
        (PokemonType::Normal, "protect", 0.5),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        let mut battle = game(mon("User", &["sonic-boom"]), vec![foe], dice);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    }
}

#[test]
fn seismic_toss_uses_user_level_without_type_or_stat_scaling() {
    for (level, kind) in [(7, PokemonType::Rock), (73, PokemonType::Flying)] {
        let mut user = mon("User", &["seismic-toss"]);
        user.level = level;
        user.attack = 1;
        user.types = vec![PokemonType::Fighting];
        let mut foe = mon("Foe", &["splash"]);
        foe.defense = 999;
        foe.types = vec![kind];
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(
            battle.participants(Side::Foes)[0].hp,
            1000 - u16::from(level)
        );
    }
}

#[test]
fn seismic_toss_respects_ghost_immunity_and_substitute() {
    for ghost in [false, true] {
        let mut user = mon("User", &["seismic-toss"]);
        user.level = 50;
        let mut foe = mon("Foe", &["splash"]);
        if ghost {
            foe.types = vec![PokemonType::Ghost];
        } else {
            foe.substitute_hp = 20;
        }
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
        assert_eq!(battle.participants(Side::Foes)[0].substitute_hp, 0);
    }
}

#[test]
fn dragon_darts_splits_two_hits_without_spread_damage_reduction() {
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    let double_damage = 1000 - single.participants(Side::Foes)[0].hp;
    assert!(double_damage > 0);
    let mut double = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Left", &["splash"]), mon("Right", &["splash"])],
        0.5,
    );
    turn(&mut double);
    for foe in double.participants(Side::Foes) {
        assert_eq!(2 * (1000 - foe.hp), double_damage);
    }
    assert_eq!(
        double.participants(Side::Allies)[0].move_pp["dragon-darts"],
        9
    );
}

#[test]
fn dragon_darts_concentrates_on_the_eligible_opponent() {
    use wilds_battle_engine::Stat;
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    let expected = single.participants(Side::Foes)[0].hp;
    for block in ["fairy", "protect", "evasion", "wonderguard", "fly"] {
        let mut blocked = mon(
            "Blocked",
            &[if block == "protect" {
                "protect"
            } else if block == "fly" {
                "fly"
            } else {
                "splash"
            }],
        );
        match block {
            "fairy" => blocked.types = vec![PokemonType::Fairy],
            "evasion" => {
                blocked.stages.insert(Stat::Evasion, 6);
            }
            "wonderguard" => blocked.ability = Some("wonder-guard".into()),
            "fly" => blocked.speed = 999,
            _ => {}
        }
        let mut battle = game(
            mon("User", &["dragon-darts"]),
            vec![blocked, mon("Other", &["splash"])],
            0.5,
        );
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000, "{block}");
        assert_eq!(battle.participants(Side::Foes)[1].hp, expected, "{block}");
        assert!(!battle.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn dragon_darts_breaks_substitute_and_fails_if_every_target_is_immune() {
    let mut foe = mon("Foe", &["splash"]);
    foe.substitute_hp = 1;
    let mut battle = game(mon("User", &["dragon-darts"]), vec![foe], 0.5);
    turn(&mut battle);
    assert_eq!(battle.participants(Side::Foes)[0].substitute_hp, 0);
    assert!(battle.participants(Side::Foes)[0].hp < 1000);
    let foes = (0..2)
        .map(|_| {
            let mut p = mon("Fairy", &["splash"]);
            p.types = vec![PokemonType::Fairy];
            p
        })
        .collect();
    let mut battle = game(mon("User", &["dragon-darts"]), foes, 0.5);
    turn(&mut battle);
    assert!(battle.participants(Side::Allies)[0].last_move_failed);
    assert!(battle.participants(Side::Foes).iter().all(|p| p.hp == 1000));
}

#[test]
fn dragon_darts_respects_redirection_and_does_not_target_the_bench() {
    use wilds_battle_engine::{Condition, ParticipantId, Rule};
    let mut selected = mon("Selected", &["splash"]);
    selected.conditions.push(Condition {
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
            priority: 1,
        }],
    });
    let mut battle = game(
        mon("User", &["dragon-darts"]),
        vec![
            selected,
            mon("Redirector", &["splash"]),
            mon("Bench", &["splash"]),
        ],
        0.5,
    );
    battle.set_active([vec![0], vec![0, 1]]).unwrap();
    turn(&mut battle);
    let foes = battle.participants(Side::Foes);
    assert_eq!(foes[0].hp, 1000);
    assert!(foes[1].hp < 1000);
    assert_eq!(foes[2].hp, 1000);
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    assert_eq!(foes[1].hp, single.participants(Side::Foes)[0].hp);
}

fn order_damage(move_id: &str, speed: u16, recharge: bool) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.speed = speed;
    let mut foe = mon("Foe", &["splash"]);
    foe.recharging = recharge;
    let mut battle = game(user, vec![foe], 0.5);
    turn(&mut battle);
    1000 - battle.participants(Side::Foes)[0].hp
}

fn switched_damage(move_id: &str) -> u16 {
    let mut battle = game(
        mon("User", &[move_id]),
        vec![mon("Outgoing", &["splash"]), mon("Incoming", &["splash"])],
        0.5,
    );
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn_with(&mut battle, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Foes {
                    matches!(c, Choice::Switch { to, .. } if to.index == 1)
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    1000 - battle.participants(Side::Foes)[1].hp
}

#[test]
fn bolt_beak_doubles_before_a_move_or_after_a_switch_but_not_recharge() {
    let base = order_damage("bolt-beak", 10, false);
    let boosted = order_damage("bolt-beak", 200, false);
    assert!(base > 0 && boosted > base * 18 / 10 && boosted <= base * 2);
    assert_eq!(order_damage("bolt-beak", 200, true), base);
    assert_eq!(switched_damage("bolt-beak"), boosted);
}

#[test]
fn fishious_rend_doubles_before_a_move_or_after_a_switch_but_not_recharge() {
    let base = order_damage("fishious-rend", 10, false);
    let boosted = order_damage("fishious-rend", 200, false);
    assert!(base > 0 && boosted > base * 18 / 10 && boosted <= base * 2);
    assert_eq!(order_damage("fishious-rend", 200, true), base);
    assert_eq!(switched_damage("fishious-rend"), boosted);
}

#[test]
fn fishious_rend_is_absorbed_even_when_its_power_is_doubled() {
    let mut user = mon("User", &["fishious-rend"]);
    user.speed = 200;
    let mut foe = mon("Foe", &["splash"]);
    foe.hp = 500;
    foe.ability = Some("water-absorb".into());
    let mut battle = game(user, vec![foe], 0.5);
    turn(&mut battle);
    assert_eq!(battle.participants(Side::Foes)[0].hp, 750);
}

const FIVE_STATS: [wilds_battle_engine::Stat; 5] = [
    wilds_battle_engine::Stat::Attack,
    wilds_battle_engine::Stat::Defense,
    wilds_battle_engine::Stat::SpAttack,
    wilds_battle_engine::Stat::SpDefense,
    wilds_battle_engine::Stat::Speed,
];

#[test]
fn clangorous_soul_pays_hp_once_and_obeys_stage_abilities() {
    for (ability, stage) in [(None, 1), (Some("simple"), 2), (Some("contrary"), -1)] {
        let mut user = mon("User", &["clangorous-soul"]);
        user.ability = ability.map(str::to_owned);
        let mut battle = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut battle);
        let user = &battle.participants(Side::Allies)[0];
        assert_eq!(user.hp, 670);
        for stat in FIVE_STATS {
            assert_eq!(user.stage(stat), stage);
        }
    }
}

#[test]
fn clangorous_soul_checks_hp_and_actual_stage_changes_before_payment() {
    for (hp, max_hp, initial_stage, ability, expected_hp, success) in [
        (330, 1000, 0, None, 330, false),
        (331, 1000, 0, None, 1, true),
        (1000, 1000, 6, None, 1000, false),
        (1000, 1000, -6, Some("contrary"), 1000, false),
        (1, 1, 0, None, 1, false),
        (100, 101, 5, None, 67, true),
    ] {
        let mut user = mon("User", &["clangorous-soul"]);
        user.hp = hp;
        user.max_hp = max_hp;
        user.ability = ability.map(str::to_owned);
        for stat in FIVE_STATS {
            user.stages.insert(stat, initial_stage);
        }
        let mut battle = game(user, vec![mon("Foe", &["splash"])], 0.5);
        turn(&mut battle);
        let user = &battle.participants(Side::Allies)[0];
        assert_eq!(user.hp, expected_hp);
        assert_eq!(user.last_move_failed, !success);
        if !success {
            for stat in FIVE_STATS {
                assert_eq!(user.stage(stat), initial_stage);
            }
        }
    }
}

fn body_press_damage(
    attack: u16,
    defense: u16,
    stages: (i8, i8),
    ability: Option<&str>,
    burn: bool,
    dice: f32,
) -> u16 {
    use wilds_battle_engine::{Stat, Status};
    let mut user = mon("User", &["body-press"]);
    user.attack = attack;
    user.defense = defense;
    user.stages.insert(Stat::Attack, stages.0);
    user.stages.insert(Stat::Defense, stages.1);
    user.ability = ability.map(str::to_owned);
    if burn {
        user.status = Some(Status::Burned);
    }
    let mut battle = game(user, vec![mon("Foe", &["splash"])], dice);
    turn(&mut battle);
    1000 - battle.participants(Side::Foes)[0].hp
}

#[test]
fn body_press_uses_defense_and_its_stages_but_attack_modifiers() {
    let base = body_press_damage(1, 100, (0, 0), None, false, 0.5);
    assert!(base > 0);
    assert_eq!(body_press_damage(999, 100, (6, 0), None, false, 0.5), base);
    assert!(body_press_damage(1, 200, (0, 0), None, false, 0.5) > base);
    assert!(body_press_damage(1, 100, (0, 2), None, false, 0.5) > base);
    assert_eq!(
        body_press_damage(1, 100, (0, 0), Some("fur-coat"), false, 0.5),
        base
    );
    assert!(body_press_damage(1, 100, (0, 0), Some("huge-power"), false, 0.5) > base);
    let burned = body_press_damage(1, 100, (0, 0), None, true, 0.5);
    assert!(burned < base);
    assert_eq!(
        body_press_damage(1, 100, (0, 0), Some("marvel-scale"), true, 0.5),
        burned
    );
}

#[test]
fn body_press_critical_hits_ignore_negative_defense_stages() {
    let critical = body_press_damage(1, 100, (0, 0), None, false, 0.0);
    assert_eq!(
        body_press_damage(1, 100, (6, -6), None, false, 0.0),
        critical
    );
}

#[test]
fn decorate_can_boost_an_ally_through_protect_without_reflection() {
    use wilds_battle_engine::Stat;
    let mut partner = mon("Partner", &["protect"]);
    partner.ability = Some("magic-bounce".into());
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [
            vec![mon("User", &["decorate"]), partner],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn_with(&mut battle, |p| {
        if p.actor.side == Side::Allies && p.actor.index == 0 {
            assert!(
                !p.choices
                    .iter()
                    .any(|c| matches!(c, Choice::UseMove { target, .. } if *target == p.actor))
            );
            p.choices.iter().find(|c| matches!(c, Choice::UseMove { target, .. } if target.side == Side::Allies && target.index == 1)).unwrap().id()
        } else {
            p.choices[0].id()
        }
    });
    let allies = battle.participants(Side::Allies);
    assert_eq!(allies[0].stage(Stat::Attack), 0);
    assert_eq!(allies[1].stage(Stat::Attack), 2);
    assert_eq!(allies[1].stage(Stat::SpAttack), 2);
}

#[test]
fn decorate_respects_substitute_caps_and_contrary() {
    use wilds_battle_engine::Stat;
    for (sub, stage, ability, expected, failed) in [
        (20, 0, None, 0, true),
        (0, 6, None, 6, true),
        (0, 5, None, 6, false),
        (0, 0, Some("contrary"), -2, false),
    ] {
        let mut foe = mon("Foe", &["splash"]);
        foe.substitute_hp = sub;
        foe.stages.insert(Stat::Attack, stage);
        foe.stages.insert(Stat::SpAttack, stage);
        foe.ability = ability.map(str::to_owned);
        let mut battle = game(mon("User", &["decorate"]), vec![foe], 0.99);
        turn(&mut battle);
        let foe = &battle.participants(Side::Foes)[0];
        assert_eq!(foe.stage(Stat::Attack), expected);
        assert_eq!(foe.stage(Stat::SpAttack), expected);
        assert_eq!(
            battle.participants(Side::Allies)[0].last_move_failed,
            failed
        );
    }
}

fn steel_damage(move_id: &str, kind: PokemonType, response: &str, dice: f32) -> Battle {
    let mut user = mon("User", &[move_id]);
    user.speed = 200;
    let mut foe = mon("Foe", &[response]);
    foe.types = vec![kind];
    let mut battle = game(user, vec![foe], dice);
    turn(&mut battle);
    battle
}

#[test]
fn behemoth_blade_has_ordinary_steel_damage_without_form_or_item_requirements() {
    for kind in [PokemonType::Normal, PokemonType::Fairy, PokemonType::Fire] {
        let blade = steel_damage("behemoth-blade", kind, "splash", 0.5);
        let same_power = steel_damage("iron-tail", kind, "splash", 0.5);
        assert_eq!(
            blade.participants(Side::Foes)[0].hp,
            same_power.participants(Side::Foes)[0].hp
        );
        assert_eq!(
            blade.participants(Side::Allies)[0].move_pp["behemoth-blade"],
            4
        );
    }
    assert!(
        steel_damage("behemoth-blade", PokemonType::Normal, "splash", 0.99)
            .participants(Side::Foes)[0]
            .hp
            < 1000
    );
    assert_eq!(
        steel_damage("behemoth-blade", PokemonType::Normal, "protect", 0.5)
            .participants(Side::Foes)[0]
            .hp,
        1000
    );
}

#[test]
fn behemoth_blade_makes_contact() {
    let battle = steel_damage("behemoth-blade", PokemonType::Normal, "beak-blast", 0.5);
    assert_eq!(
        battle.participants(Side::Allies)[0].status,
        Some(wilds_battle_engine::Status::Burned)
    );
}

#[test]
fn behemoth_bash_has_ordinary_steel_damage_without_form_or_item_requirements() {
    for kind in [PokemonType::Normal, PokemonType::Fairy, PokemonType::Fire] {
        let bash = steel_damage("behemoth-bash", kind, "splash", 0.5);
        let same_power = steel_damage("iron-tail", kind, "splash", 0.5);
        assert_eq!(
            bash.participants(Side::Foes)[0].hp,
            same_power.participants(Side::Foes)[0].hp
        );
        assert_eq!(
            bash.participants(Side::Allies)[0].move_pp["behemoth-bash"],
            4
        );
    }
    assert!(
        steel_damage("behemoth-bash", PokemonType::Normal, "splash", 0.99).participants(Side::Foes)
            [0]
        .hp < 1000
    );
    assert_eq!(
        steel_damage("behemoth-bash", PokemonType::Normal, "protect", 0.5).participants(Side::Foes)
            [0]
        .hp,
        1000
    );
}

#[test]
fn behemoth_bash_makes_contact_and_hits_substitute() {
    let battle = steel_damage("behemoth-bash", PokemonType::Normal, "beak-blast", 0.5);
    assert_eq!(
        battle.participants(Side::Allies)[0].status,
        Some(wilds_battle_engine::Status::Burned)
    );
    let mut foe = mon("Foe", &["splash"]);
    foe.substitute_hp = 1;
    let mut battle = game(mon("User", &["behemoth-bash"]), vec![foe], 0.5);
    turn(&mut battle);
    assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    assert_eq!(battle.participants(Side::Foes)[0].substitute_hp, 0);
}

#[test]
fn aura_wheel_requires_morpeko_and_uses_its_current_form() {
    use wilds_battle_engine::Stat;
    for (species_id, species, form, hits, boost) in [
        (0, "ditto", None, false, 0),
        (877, "morpeko", None, false, 0),
        (877, "morpeko", Some("morpeko-hangry"), true, 1),
        (0, "morpeko-hangry", None, true, 1),
    ] {
        let mut user = mon("User", &["aura-wheel"]);
        user.species_id = species_id;
        user.species = species.into();
        user.form = form.map(str::to_owned);
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![PokemonType::Ground];
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp < 1000, hits);
        assert_eq!(
            battle.participants(Side::Allies)[0].stage(Stat::Speed),
            boost
        );
    }
}

#[test]
fn aura_wheel_boosts_after_substitute_or_ko_but_not_protect_or_absorption() {
    use wilds_battle_engine::Stat;
    for outcome in ["hit", "substitute", "ko", "protect", "absorb"] {
        let mut user = mon("User", &["aura-wheel"]);
        user.species_id = 877;
        let mut foe = mon(
            "Foe",
            &[if outcome == "protect" {
                "protect"
            } else {
                "splash"
            }],
        );
        if outcome == "substitute" {
            foe.substitute_hp = 1;
        }
        if outcome == "ko" {
            foe.hp = 1;
        }
        if outcome == "absorb" {
            foe.ability = Some("volt-absorb".into());
        }
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(
            battle.participants(Side::Allies)[0].stage(Stat::Speed),
            if matches!(outcome, "protect" | "absorb") {
                0
            } else {
                1
            },
            "{outcome}"
        );
    }
}

#[test]
fn overdrive_hits_both_opponents_through_substitutes_and_spares_allies() {
    let mut left = mon("Left", &["splash"]);
    left.substitute_hp = 250;
    let mut battle = Battle::with_rng(
        Fixed(0.5),
        [
            vec![mon("User", &["overdrive"]), mon("Partner", &["splash"])],
            vec![left, mon("Right", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    turn(&mut battle);
    assert!(
        battle
            .participants(Side::Allies)
            .iter()
            .all(|p| p.hp == 1000)
    );
    let foes = battle.participants(Side::Foes);
    assert!(foes[0].hp < 1000);
    assert_eq!(foes[0].hp, foes[1].hp);
    assert_eq!(foes[0].substitute_hp, 250);
}

#[test]
fn overdrive_respects_soundproof_ground_immunity_and_protect() {
    for block in ["soundproof", "ground", "protect"] {
        let mut blocked = mon(
            "Blocked",
            &[if block == "protect" {
                "protect"
            } else {
                "splash"
            }],
        );
        if block == "soundproof" {
            blocked.ability = Some("soundproof".into());
        }
        if block == "ground" {
            blocked.types = vec![PokemonType::Ground];
        }
        let mut battle = game(
            mon("User", &["overdrive"]),
            vec![blocked, mon("Other", &["splash"])],
            0.5,
        );
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000, "{block}");
        assert!(battle.participants(Side::Foes)[1].hp < 1000, "{block}");
    }
}
