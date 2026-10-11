//! Checks the shipped Lua files, independently of the API examples in sheet_moves.lua.
use std::collections::HashSet;
use std::path::Path;
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, Gender, MoveCatalog,
    ParticipantId, Pokemon, PokemonType, Side, Stat, Status,
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
    let mut pokemon = Pokemon::new(name, moves.iter().map(|id| id.to_string()).collect());
    pokemon.hp = 1000;
    pokemon.max_hp = 1000;
    pokemon
}

struct Game(Battle);
impl Game {
    fn new(allies: Vec<Pokemon>, foes: Vec<Pokemon>, dice: f32) -> Self {
        Self(
            Battle::with_rng(Fixed(dice), [allies, foes], MoveCatalog::builtin().unwrap()).unwrap(),
        )
    }
    fn duel(moves: &[&str], foe: Pokemon, dice: f32) -> Self {
        Self::new(vec![mon("Ally", moves)], vec![foe], dice)
    }
    fn turn(&mut self, ally: &str, foe: &str) -> Vec<BattleEvent> {
        self.turn_with(ally, foe, None)
    }
    fn turn_with(&mut self, ally: &str, foe: &str, partner: Option<&str>) -> Vec<BattleEvent> {
        for _ in 0..30 {
            let result = self.0.advance().unwrap();
            if !result.events.is_empty() {
                return result.events;
            }
            let AdvanceStatus::Awaiting(prompt) = result.status else {
                return result.events;
            };
            let selected = if prompt.actor.side == Side::Allies && prompt.actor.index == 0 {
                ally
            } else if prompt.actor.side == Side::Allies {
                partner.unwrap_or(foe)
            } else {
                foe
            };
            let choice = prompt
                .choices
                .iter()
                .find(|choice| {
                    let Choice::UseMove { move_id, .. } = choice;
                    move_id == selected
                })
                .unwrap_or_else(|| panic!("missing {selected}: {:?}", prompt.choices));
            self.0
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: choice.id(),
                })
                .unwrap();
        }
        panic!("turn did not advance");
    }
    fn ally(&self) -> &Pokemon {
        &self.0.participants(Side::Allies)[0]
    }
    fn foe(&self) -> &Pokemon {
        &self.0.participants(Side::Foes)[0]
    }
}

fn said(events: &[BattleEvent], text: &str) -> bool {
    events
        .iter()
        .any(|event| matches!(event, BattleEvent::Message(message) if message.contains(text)))
}

#[test]
fn power_herb_completes_charge_moves_immediately_and_pays_pp_once() {
    for move_id in [
        "razor-wind",
        "skull-bash",
        "sky-attack",
        "shadow-force",
        "freeze-shock",
        "ice-burn",
        "phantom-force",
        "geomancy",
        "solar-blade",
        "meteor-beam",
        "fly",
        "bounce",
        "dig",
        "solar_beam",
        "dive",
    ] {
        let mut ally = mon("Ally", &[move_id, "splash"]);
        ally.item = Some("power-herb".into());
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![PokemonType::Psychic];
        let mut game = Game::new(vec![ally], vec![foe], 0.5);
        let pp = game.ally().move_pp[move_id];
        let events = game.turn(move_id, "splash");
        assert!(said(&events, "Power Herb"), "{move_id}");
        assert_eq!(game.ally().item, None, "{move_id}");
        assert_eq!(
            game.ally().last_consumed_item.as_deref(),
            Some("power-herb")
        );
        assert!(game.ally().consumed_items.contains("power-herb"));
        assert_eq!(game.ally().move_pp[move_id], pp - 1, "{move_id}");
        assert!(game.ally().hidden().is_none(), "{move_id}");
        if move_id == "geomancy" {
            assert_eq!(game.ally().stage(Stat::SpAttack), 2);
        } else {
            assert!(game.foe().hp < 1000, "{move_id}");
        }
        if move_id == "meteor-beam" {
            assert_eq!(game.ally().stage(Stat::SpAttack), 1);
        }
        if move_id == "skull-bash" {
            assert_eq!(game.ally().stage(Stat::Defense), 1);
        }
        let hp = game.foe().hp;
        game.turn("splash", "splash");
        assert_eq!(game.foe().hp, hp, "{move_id} still forced a second turn");
    }
}

#[test]
fn power_herb_preserves_weather_shortcuts_and_does_not_skip_lock_ins() {
    for move_id in ["solar_beam", "solar-blade"] {
        let mut ally = mon("Ally", &["sunny_day", move_id]);
        ally.item = Some("power_herb".into());
        let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
        game.turn("sunny_day", "splash");
        game.turn(move_id, "splash");
        assert!(game.foe().hp < 1000);
        assert_eq!(game.ally().item.as_deref(), Some("power_herb"));
        assert!(game.ally().consumed_items.is_empty());
    }
    let mut ally = mon("Ally", &["thrash"]);
    ally.item = Some("power-herb".into());
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
    game.turn("thrash", "splash");
    assert_eq!(game.ally().item.as_deref(), Some("power-herb"));
    let mut ally = mon("Ally", &["meteor-beam"]);
    ally.item = Some("oran-berry".into());
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
    game.turn("meteor-beam", "splash");
    assert_eq!(game.foe().hp, 1000);
    assert_eq!(game.ally().item.as_deref(), Some("oran-berry"));
    game.turn("meteor-beam", "splash");
    assert!(game.foe().hp < 1000);
}

#[test]
fn beak_blast_reacts_to_zero_damage_and_knockout_contact_but_not_immunity() {
    for attack in ["hold-back", "false_swipe", "body-slam", "tackle"] {
        let mut ally = mon("Ally", &["beak-blast"]);
        ally.hp = 1;
        let mut game = Game::new(vec![ally], vec![mon("Foe", &[attack])], 0.5);
        game.turn("beak-blast", attack);
        assert_eq!(game.foe().status, Some(Status::Burned), "{attack}");
        let leaves_one_hp = matches!(attack, "hold-back" | "false_swipe");
        assert_eq!(game.ally().hp, if leaves_one_hp { 1 } else { 0 });
        if leaves_one_hp {
            assert!(game.ally().last_hit.is_none());
            assert_eq!(game.ally().damage_taken, 0);
        }
    }
    let mut ally = mon("Ally", &["beak-blast"]);
    ally.types = vec![PokemonType::Ghost];
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["body-slam"])], 0.5);
    game.turn("beak-blast", "body-slam");
    assert_eq!(game.foe().status, None);
    assert_eq!(game.ally().hp, 1000);
}

#[test]
fn pledges_combine_in_both_orders_promote_the_partner_and_apply_the_right_effect() {
    let pairs = [
        (
            "fire-pledge",
            "water-pledge",
            PokemonType::Water,
            "rainbow",
            Side::Allies,
        ),
        (
            "fire-pledge",
            "grass-pledge",
            PokemonType::Fire,
            "sea_of_fire",
            Side::Foes,
        ),
        (
            "water-pledge",
            "grass-pledge",
            PokemonType::Grass,
            "swamp",
            Side::Foes,
        ),
    ];
    for (a, b, kind, effect, side) in pairs {
        let mut damages = Vec::new();
        for (first, second) in [(a, b), (b, a)] {
            let mut leader = mon("Leader", &[first]);
            leader.speed = 300;
            let mut foe = mon("Foe", &["splash"]);
            foe.speed = 200;
            let mut game = Game::new(vec![leader, mon("Partner", &[second])], vec![foe], 0.5);
            let events = game.turn_with(first, "splash", Some(second));
            let uses: Vec<&str> = events
                .iter()
                .filter_map(|event| match event {
                    BattleEvent::Message(text) if text.contains(" used ") => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert!(uses[0].starts_with("Leader"));
            assert!(uses[1].starts_with("Partner"));
            assert!(uses[2].starts_with("Foe"));
            let hits: Vec<u16> = events
                .iter()
                .filter_map(|event| match event {
                    BattleEvent::Damage { target, amount, .. } if target.side == Side::Foes => {
                        Some(*amount)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(hits.len(), if effect == "sea_of_fire" { 2 } else { 1 });
            assert_eq!(game.foe().last_hit.as_ref().unwrap().move_type, kind);
            damages.push(hits[0]);
            let condition = game
                .0
                .side_conditions(side)
                .iter()
                .find(|c| c.name == effect)
                .unwrap();
            assert_eq!(condition.turns_left, Some(3));
            if effect == "swamp" {
                assert!(condition.rules.iter().any(|r| matches!(r, wilds_battle_engine::Rule::StatMultiplier { factor, .. } if *factor == 0.25)));
            }
        }
        assert_eq!(damages[0], damages[1]);
    }
}

#[test]
fn pledges_do_not_combine_identical_moves_or_leave_effects_after_a_block() {
    for protected in [false, true] {
        let first = "fire-pledge";
        let second = if protected { "water-pledge" } else { first };
        let foe_move = if protected { "protect" } else { "splash" };
        let mut game = Game::new(
            vec![mon("Leader", &[first]), mon("Partner", &[second])],
            vec![mon("Foe", &[foe_move])],
            0.5,
        );
        let events = game.turn_with(first, foe_move, Some(second));
        assert!(game.0.side_conditions(Side::Allies).is_empty());
        assert!(game.0.side_conditions(Side::Foes).is_empty());
        let hits = events
            .iter()
            .filter(|e| matches!(e, BattleEvent::Damage { .. }))
            .count();
        assert_eq!(hits, if protected { 0 } else { 2 });
    }
    let mut partner = mon("Partner", &["water-pledge"]);
    partner.status = Some(Status::Asleep);
    partner.status_turns = 5;
    let mut leader = mon("Leader", &["fire-pledge"]);
    leader.speed = 200;
    let mut game = Game::new(vec![leader, partner], vec![mon("Foe", &["splash"])], 0.5);
    let events = game.turn_with("fire-pledge", "splash", Some("water-pledge"));
    assert!(said(&events, "is waiting for Partner"));
    assert_eq!(game.foe().hp, 1000);
    assert!(game.0.side_conditions(Side::Allies).is_empty());
    assert!(
        game.0.participants(Side::Allies)[1]
            .condition("pledge_partner")
            .is_none()
    );
}

#[test]
fn grassy_glide_priority_depends_on_terrain_and_grounding_and_expires() {
    let source = format!("return {{ (function() {} end)(), (function() {} end)(),
        {{id='grass', name='Grass', type=Type.Grass, category=Category.Status, pp=40,
          script=function(ctx) ctx:start_effect(ctx.field, 'grassy_terrain', {{turns=2, group='terrain'}}) end}},
        {{id='wait', name='Wait', type=Type.Normal, category=Category.Status, pp=40, script=function(_) end}}
    }}", include_str!("../../../moves/grassy-glide.lua"), include_str!("../../../moves/tackle.lua"));
    for flying in [false, true] {
        let mut ally = mon("Ally", &["grassy-glide", "grass"]);
        if flying {
            ally.types = vec![PokemonType::Flying];
        }
        let mut foe = mon("Foe", &["tackle", "wait"]);
        foe.speed = 200;
        let mut game = Game(
            Battle::with_rng(
                Fixed(0.5),
                [vec![ally], vec![foe]],
                MoveCatalog::from_lua(&source).unwrap(),
            )
            .unwrap(),
        );
        game.turn("grass", "wait");
        let first_move = |events: Vec<BattleEvent>| {
            events
                .into_iter()
                .find_map(|e| match e {
                    BattleEvent::Message(text) if text.contains(" used ") => Some(text),
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(
            first_move(game.turn("grassy-glide", "tackle")),
            if flying {
                "Foe used Tackle!"
            } else {
                "Ally used Grassy Glide!"
            }
        );
        assert_eq!(
            first_move(game.turn("grassy-glide", "tackle")),
            "Foe used Tackle!"
        );
    }
}

#[test]
fn shell_trap_is_promoted_after_the_triggering_attacker_only_once() {
    let mut first = mon("First", &["tackle"]);
    first.speed = 300;
    let mut second = mon("Second", &["tackle"]);
    second.speed = 200;
    let mut game = Game::new(vec![mon("Ally", &["shell-trap"])], vec![first, second], 0.5);
    let events = game.turn("shell-trap", "tackle");
    let uses: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::Message(text) if text.contains(" used ") => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        uses,
        [
            "First used Tackle!",
            "Ally used Shell Trap!",
            "Second used Tackle!"
        ]
    );

    let mut game = Game::duel(&["shell-trap"], mon("Foe", &["water-gun"]), 0.5);
    let events = game.turn("shell-trap", "water-gun");
    assert!(said(&events, "shell trap didn't work"));
    assert_eq!(game.foe().hp, 1000);
}

#[test]
fn sheer_cold_uses_one_ohko_check_with_type_and_level_restrictions() {
    for (ice_user, dice, level, target_ice, guaranteed, hits) in [
        (false, 0.25, 100, false, false, false),
        (true, 0.25, 100, false, false, true),
        (false, 0.19, 100, false, false, true),
        (false, 0.99, 100, false, true, true),
        (true, 0.0, 99, false, true, false),
        (true, 0.0, 100, true, true, false),
    ] {
        let mut ally = mon("Ally", &["sheer-cold"]);
        ally.level = level;
        if ice_user {
            ally.types = vec![PokemonType::Ice];
        }
        if guaranteed {
            ally.conditions.push(wilds_battle_engine::Condition {
                name: "aim".into(),
                value: 1,
                turns_left: Some(2),
                group: None,
                end_message: None,
                rules: vec![wilds_battle_engine::Rule::AlwaysHit { against: None }],
            });
        }
        let mut foe = mon("Foe", &["splash"]);
        if target_ice {
            foe.types = vec![PokemonType::Ice];
        }
        let mut game = Game::new(vec![ally], vec![foe], dice);
        game.turn("sheer-cold", "splash");
        assert_eq!(
            game.foe().hp == 0,
            hits,
            "ice={ice_user} dice={dice} level={level} immune={target_ice} guaranteed={guaranteed}"
        );
    }
}

#[test]
fn attack_screens_use_the_battle_format_and_critical_hits_bypass_them() {
    for (screen, attack) in [("baddy-bad", "tackle"), ("glitzy-glow", "water-gun")] {
        let damage = |with_screen: bool, multi: bool, critical: bool| {
            let mut ally = mon("Ally", &[screen, "splash"]);
            ally.speed = 200;
            let mut allies = vec![ally];
            if multi {
                let mut fainted_partner = mon("Partner", &["splash"]);
                fainted_partner.hp = 0;
                allies.push(fainted_partner);
            }
            let mut game = Game::new(
                allies,
                vec![mon("Foe", &[attack])],
                if critical { 0.0 } else { 0.5 },
            );
            game.turn(if with_screen { screen } else { "splash" }, attack);
            1000 - game.ally().hp
        };
        let base = damage(false, false, false);
        let single = damage(true, false, false);
        let multi = damage(true, true, false);
        assert!((single as i32 * 2 - base as i32).abs() <= 2, "{screen}");
        assert!((multi as i32 * 3 - base as i32 * 2).abs() <= 3, "{screen}");
        for multi in [false, true] {
            assert_eq!(damage(true, multi, true), damage(false, multi, true));
        }
    }
}

#[test]
fn shell_side_arm_makes_contact_only_for_its_physical_hit() {
    for physical in [true, false] {
        let mut ally = mon("Ally", &["shell-side-arm"]);
        ally.attack = if physical { 200 } else { 10 };
        ally.sp_attack = if physical { 10 } else { 200 };
        let mut game = Game::new(vec![ally], vec![mon("Foe", &["beak-blast"])], 0.5);
        game.turn("shell-side-arm", "beak-blast");
        let hit = game.foe().last_hit.as_ref().unwrap();
        assert_eq!(hit.contact, physical);
        assert_eq!(
            hit.category,
            if physical {
                wilds_battle_engine::Category::Physical
            } else {
                wilds_battle_engine::Category::Special
            }
        );
        assert_eq!(
            game.ally().status,
            if physical { Some(Status::Burned) } else { None }
        );
    }
}

#[test]
fn focus_punch_loses_focus_from_damage_without_being_interrupted_by_flinch() {
    let mut foe = mon("Foe", &["headbutt"]);
    foe.speed = 200;
    let mut game = Game::duel(&["focus-punch", "tackle"], foe, 0.2);
    let events = game.turn("focus-punch", "headbutt");
    assert!(said(&events, "lost its focus"));
    assert!(!said(&events, "flinched"));
    assert_eq!(game.foe().hp, 1000);
    assert!(game.ally().condition("focus_punch").is_none());
    let events = game.turn("tackle", "headbutt");
    // Immunity expired with Focus Punch.
    assert!(said(&events, "flinched"));
}

#[test]
fn lash_out_uses_actual_drops_this_turn_and_forgets_previous_turns() {
    let damage = |capped: bool, lowered: bool| {
        let mut ally = mon("Ally", &["lash-out"]);
        if capped {
            ally.change_stage(Stat::Defense, -6);
        }
        let mut foe = mon("Foe", &["leer", "splash"]);
        foe.speed = 200;
        let mut game = Game::new(vec![ally], vec![foe], 0.5);
        game.turn("lash-out", if lowered { "leer" } else { "splash" });
        let first = 1000 - game.foe().hp;
        game.turn("lash-out", "splash");
        let second = 1000 - first - game.foe().hp;
        assert!(!game.ally().stats_lowered_this_turn);
        (first, second)
    };
    let (base, _) = damage(false, false);
    let (boosted, next) = damage(false, true);
    assert!(boosted >= base * 19 / 10 && boosted <= base * 21 / 10);
    assert_eq!(next, base);
    assert_eq!(damage(true, true).0, base);
}

#[test]
fn burning_jealousy_checks_each_targets_actual_stat_increases() {
    let mut boosted = mon("Boosted", &["stockpile", "splash"]);
    boosted.speed = 200;
    let mut capped = boosted.clone();
    capped.name = "Capped".into();
    capped.change_stage(Stat::Defense, 6);
    capped.change_stage(Stat::SpDefense, 6);
    let mut game = Game::new(
        vec![mon("Ally", &["burning-jealousy"])],
        vec![boosted, capped],
        0.5,
    );
    game.turn("burning-jealousy", "stockpile");
    assert_eq!(
        game.0.participants(Side::Foes)[0].status,
        Some(Status::Burned)
    );
    assert_eq!(game.0.participants(Side::Foes)[1].status, None);

    let mut foe = mon("Foe", &["stockpile", "splash"]);
    foe.speed = 200;
    let mut game = Game::duel(&["splash", "burning-jealousy"], foe, 0.5);
    game.turn("splash", "stockpile");
    game.turn("burning-jealousy", "splash");
    assert_eq!(game.foe().status, None);
}

#[test]
fn eerie_spell_can_exhaust_a_selected_move_before_it_executes() {
    let mut ally = mon("Ally", &["splash", "eerie-spell"]);
    ally.speed = 200;
    let mut foe = mon("Foe", &["tackle"]);
    foe.move_pp.insert("tackle".into(), 4);
    let mut game = Game::new(vec![ally], vec![foe], 0.5);
    game.turn("splash", "tackle");
    let hp = game.ally().hp;
    let events = game.turn("eerie-spell", "tackle");
    assert!(said(&events, "no PP left for Tackle"));
    assert!(!said(&events, "Foe used Tackle"));
    assert_eq!(game.ally().hp, hp);
    assert_eq!(game.foe().move_pp["tackle"], 0);
    assert!(said(&game.turn("splash", "struggle"), "used Struggle"));
}

#[test]
fn eerie_spell_reduces_last_move_pp_only_after_a_surviving_hit() {
    let mut foe = mon("Foe", &["splash", "protect"]);
    foe.speed = 200;
    foe.move_pp.insert("splash".into(), 3);
    let mut game = Game::duel(&["eerie-spell"], foe, 0.5);
    let events = game.turn("eerie-spell", "splash");
    assert!(said(&events, "lost 2 PP"));
    assert_eq!(game.foe().move_pp["splash"], 0);
    assert!(game.foe().hp < 1000);
    let pp = game.foe().move_pp["protect"];
    let hp = game.foe().hp;
    game.turn("eerie-spell", "protect");
    assert_eq!(game.foe().hp, hp);
    assert_eq!(game.foe().move_pp["protect"], pp - 1);

    let mut ally = mon("Ally", &["eerie-spell"]);
    ally.speed = 200;
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
    let pp = game.foe().move_pp["splash"];
    game.turn("eerie-spell", "splash");
    assert_eq!(game.foe().move_pp["splash"], pp - 1); // no prior move
    game.turn("eerie-spell", "splash");
    assert_eq!(game.foe().move_pp["splash"], pp - 5);

    let mut foe = mon("Foe", &["splash"]);
    foe.speed = 200;
    foe.hp = 1;
    let mut game = Game::duel(&["eerie-spell"], foe, 0.5);
    let pp = game.foe().move_pp["splash"];
    game.turn("eerie-spell", "splash");
    assert_eq!(game.foe().hp, 0);
    assert_eq!(game.foe().move_pp["splash"], pp - 1);
}

#[test]
fn stockpile_caps_at_three_and_swallow_heals_each_tier() {
    for layers in 1..=3 {
        let mut ally = mon("Ally", &["stockpile", "swallow"]);
        ally.hp = 100;
        let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
        assert!(said(&game.turn("swallow", "splash"), "failed"));
        for _ in 0..layers {
            game.turn("stockpile", "splash");
        }
        assert_eq!(game.ally().condition("stockpile").unwrap().value, layers);
        assert_eq!(game.ally().stage(Stat::Defense), layers as i8);
        assert_eq!(game.ally().stage(Stat::SpDefense), layers as i8);
        if layers == 3 {
            assert!(said(&game.turn("stockpile", "splash"), "failed"));
            assert_eq!(game.ally().stage(Stat::Defense), 3);
            assert_eq!(game.ally().condition("stockpile").unwrap().value, 3);
        }
        game.turn("swallow", "splash");
        assert_eq!(game.ally().hp, [350, 600, 1000][layers as usize - 1]);
        assert_eq!(game.ally().stage(Stat::Defense), 0);
        assert_eq!(game.ally().stage(Stat::SpDefense), 0);
        for mark in ["stockpile", "stockpile_defense", "stockpile_sp_defense"] {
            assert!(game.ally().condition(mark).is_none());
        }
    }
}

#[test]
fn stockpile_tracks_only_actual_boosts_and_swallow_consumes_at_full_hp() {
    let mut ally = mon("Ally", &["stockpile", "swallow"]);
    ally.change_stage(Stat::Defense, 5);
    ally.change_stage(Stat::SpDefense, 6);
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["splash"])], 0.5);
    for _ in 0..3 {
        game.turn("stockpile", "splash");
    }
    assert_eq!(game.ally().condition("stockpile_defense").unwrap().value, 1);
    assert_eq!(
        game.ally().condition("stockpile_sp_defense").unwrap().value,
        0
    );
    game.turn("swallow", "splash");
    assert_eq!(game.ally().hp, 1000);
    assert_eq!(game.ally().stage(Stat::Defense), 5);
    assert_eq!(game.ally().stage(Stat::SpDefense), 6);
    assert!(game.ally().condition("stockpile").is_none());
}

#[test]
fn swallow_preserves_other_stat_changes() {
    let mut ally = mon("Ally", &["stockpile", "swallow"]);
    ally.speed = 200;
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["screech", "splash"])], 0.5);
    game.turn("stockpile", "screech");
    game.turn("stockpile", "splash");
    game.turn("swallow", "splash");
    assert_eq!(game.ally().stage(Stat::Defense), -2);
    assert_eq!(game.ally().stage(Stat::SpDefense), 0);
}

#[test]
fn gravity_boosts_plain_and_scripted_accuracy_until_expiry_without_changing_stages() {
    for move_id in ["fire-blast", "thunder"] {
        let mut game = Game::duel(&["gravity", move_id, "growl"], mon("Foe", &["growl"]), 0.9);
        game.turn(move_id, "growl");
        assert_eq!(game.foe().hp, 1000);
        game.turn("gravity", "growl");
        game.turn(move_id, "growl");
        assert!(game.foe().hp < 1000, "{move_id}");
        assert_eq!(game.ally().stage(Stat::Accuracy), 0);
        assert_eq!(game.foe().stage(Stat::Evasion), 0);
        for _ in 0..3 {
            game.turn("growl", "growl");
        }
        assert!(game.0.field_conditions().is_empty());
        let hp = game.foe().hp;
        game.turn(move_id, "growl");
        assert_eq!(game.foe().hp, hp, "{move_id}");
    }
}

#[test]
fn gravity_does_not_boost_one_hit_knockout_accuracy() {
    let mut game = Game::duel(&["gravity", "fissure"], mon("Foe", &["growl"]), 0.4);
    game.turn("gravity", "growl");
    game.turn("fissure", "growl");
    assert_eq!(game.foe().hp, 1000);
}

#[test]
fn gravity_grounds_flyers_and_expires_without_refreshing_on_failure() {
    let mut bird = mon("Bird", &["fly", "growl"]);
    bird.types = vec![PokemonType::Flying];
    bird.speed = 200;
    let mut game = Game::duel(&["gravity", "earthquake", "growl"], bird, 0.5);
    let events = game.turn("gravity", "fly");
    assert!(said(&events, "couldn't stay airborne"));
    assert!(game.foe().hidden().is_none());
    assert_eq!(game.0.field_conditions()[0].turns_left, Some(4));
    assert!(said(&game.turn("gravity", "growl"), "failed"));
    assert_eq!(game.0.field_conditions()[0].turns_left, Some(3));
    game.turn("earthquake", "growl");
    assert!(game.foe().hp < 1000);
    assert!(game.foe().hidden().is_none());
    game.turn("growl", "growl");
    let events = game.turn("growl", "growl");
    assert!(said(&events, "Gravity returned to normal"));
    assert!(game.0.field_conditions().is_empty());
    let hp = game.foe().hp;
    game.turn("earthquake", "growl");
    assert_eq!(game.foe().hp, hp);
    game.turn("growl", "fly");
    assert!(game.foe().hidden().is_some());
}

#[test]
fn gravity_preserves_non_airborne_charge_moves() {
    for move_id in ["dig", "dive", "phantom-force"] {
        let mut foe = mon("Foe", &[move_id]);
        foe.speed = 200;
        let mut ally = mon("Ally", &["gravity", "growl"]);
        ally.types = vec![PokemonType::Grass];
        let mut game = Game::new(vec![ally], vec![foe], 0.5);
        game.turn("gravity", move_id);
        assert!(game.foe().hidden().is_some(), "{move_id} was interrupted");
        game.turn("growl", move_id);
        assert!(game.foe().hidden().is_none(), "{move_id} did not finish");
        assert!(game.ally().hp < 1000, "{move_id} did not hit");
    }
}

#[test]
fn gravity_blocks_airborne_moves_without_crash_damage_and_boosts_grav_apple() {
    for move_id in [
        "fly",
        "bounce",
        "jump-kick",
        "high-jump-kick",
        "flying-press",
        "splash",
        "sky-drop",
    ] {
        let mut foe = mon("Foe", &["gravity"]);
        foe.speed = 200;
        let mut game = Game::duel(&[move_id], foe, 0.5);
        let pp = game.ally().move_pp[move_id];
        assert!(
            said(&game.turn(move_id, "gravity"), "because of gravity"),
            "{move_id}"
        );
        assert_eq!(game.ally().move_pp[move_id], pp);
        assert_eq!((game.ally().hp, game.foe().hp), (1000, 1000), "{move_id}");
        assert!(game.ally().hidden().is_none());
    }
    let damage = |opener| {
        let mut game = Game::duel(
            &["gravity", "grav-apple", "growl"],
            mon("Foe", &["growl"]),
            0.5,
        );
        game.turn(opener, "growl");
        game.turn("grav-apple", "growl");
        1000 - game.foe().hp
    };
    let normal = damage("growl");
    let boosted = damage("gravity");
    assert!(boosted >= normal * 14 / 10 && boosted <= normal * 16 / 10);
}

#[test]
fn gravity_interrupts_a_queued_airborne_second_turn() {
    for move_id in ["fly", "bounce"] {
        let mut ally = mon("Ally", &["gravity", "growl"]);
        ally.speed = 200;
        let mut game = Game::new(vec![ally], vec![mon("Foe", &[move_id, "growl"])], 0.5);
        game.turn("growl", move_id);
        assert!(game.foe().hidden().is_some());
        let pp = game.foe().move_pp[move_id];
        game.turn("gravity", move_id);
        assert!(game.foe().hidden().is_none());
        assert_eq!(game.ally().hp, 1000);
        assert_eq!(game.foe().move_pp[move_id], pp);
        game.turn("growl", "growl");
        assert_eq!(game.ally().hp, 1000);
    }
}

#[test]
fn gravity_filters_choices_and_allows_struggle_when_every_move_is_blocked() {
    let blocked = [
        "fly",
        "bounce",
        "jump-kick",
        "high-jump-kick",
        "flying-press",
        "splash",
        "sky-drop",
    ];
    let mut moves = blocked.to_vec();
    moves.extend(["gravity", "growl"]);
    let mut game = Game::duel(&moves, mon("Foe", &["growl"]), 0.5);
    game.turn("gravity", "growl");
    let AdvanceStatus::Awaiting(prompt) = game.0.advance().unwrap().status else {
        panic!("expected a choice")
    };
    assert_eq!(prompt.choices.len(), 2);
    for choice in prompt.choices {
        let Choice::UseMove { move_id, .. } = choice;
        assert!(!blocked.contains(&move_id.as_str()));
    }
    let mut foe = mon("Foe", &["gravity"]);
    foe.speed = 200;
    let mut game = Game::duel(&["splash"], foe, 0.5);
    game.turn("splash", "gravity");
    let events = game.turn("struggle", "gravity");
    assert!(said(&events, "used Struggle"));
    assert!(game.foe().hp < 1000);
}

#[test]
fn catalog_tracks_included_and_excluded_moves_with_canonical_names() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../moves/catalog.json")).unwrap();
    let entries = manifest["moves"].as_array().unwrap();
    assert_eq!(entries.len(), 540);
    let excluded = manifest["excluded_moves"].as_array().unwrap();
    assert_eq!(excluded.len(), 34);
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry["sheet_row"].is_number())
            .count()
            + excluded.len(),
        572
    );
    let builtin = MoveCatalog::builtin().unwrap();
    let disk = MoveCatalog::from_directory(root.join("moves")).unwrap();
    assert_eq!(builtin.len(), 549);
    assert_eq!(builtin.len(), disk.len());
    let mut ids = HashSet::new();
    let mut slugs = HashSet::new();
    for entry in entries {
        assert!(ids.insert(entry["id"].as_u64().unwrap()));
        let slug = entry["slug"].as_str().unwrap();
        assert!(slugs.insert(slug));
        assert!(
            slug.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        );
        let runtime_id = entry["runtime_id"].as_str().unwrap();
        let spec = builtin
            .get(runtime_id)
            .unwrap_or_else(|| panic!("missing {runtime_id}"));
        assert_eq!(spec.name, entry["name"].as_str().unwrap());
        assert_eq!(spec.name, disk.get(runtime_id).unwrap().name);
        if entry["status"] != "system" {
            assert_eq!(entry["file"], format!("moves/{slug}.lua"));
            assert!(root.join(entry["file"].as_str().unwrap()).is_file());
        }
        assert_eq!(
            spec.has_flag("incomplete"),
            entry["status"] == "partial",
            "{slug}"
        );
    }
    for entry in excluded {
        assert!(ids.insert(entry["id"].as_u64().unwrap()));
        let slug = entry["slug"].as_str().unwrap();
        assert!(slugs.insert(slug));
        assert!(
            builtin.get(slug).is_none(),
            "excluded move {slug} still embedded"
        );
        assert!(
            disk.get(slug).is_none(),
            "excluded move {slug} still on disk"
        );
        assert!(!root.join(format!("moves/{slug}.lua")).exists());
    }
    // This ordinary attack does not require the user to Dynamax.
    assert!(builtin.get("dynamax-cannon").is_some());
    let files: Vec<_> = std::fs::read_dir(root.join("moves"))
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "lua"))
        .collect();
    assert_eq!(files.len(), 548);
    assert!(
        files
            .iter()
            .all(|entry| !entry.file_name().to_string_lossy().contains('_'))
    );
}

#[test]
fn every_shipped_move_runs_with_items_statuses_and_multiple_targets() {
    let catalog = MoveCatalog::builtin().unwrap();
    for move_id in catalog.ids().filter(|id| *id != "struggle") {
        for seed in 1..=3 {
            let mut ally = mon("Ally", &[move_id]);
            ally.hp = 400;
            ally.gender = Gender::Female;
            ally.species = if seed == 1 {
                "darkrai"
            } else {
                "hoopa-unbound"
            }
            .into();
            ally.item = Some("flame-plate".into());
            ally.ability = Some("plus".into());
            ally.ivs = Some([30, 31, 30, 31, 30, 31]);
            let mut foe = mon("Foe", &["scratch", "protect", "fly"]);
            foe.gender = Gender::Male;
            foe.item = Some("oran-berry".into());
            foe.status = Some(if seed == 1 {
                Status::Poisoned
            } else {
                Status::Asleep
            });
            foe.status_turns = 3;
            let mut other = mon("Other", &["ember"]);
            other.types = vec![PokemonType::Grass, PokemonType::Flying];
            let mut battle = Battle::new(
                seed,
                [vec![ally, mon("Partner", &["splash"])], vec![foe, other]],
                catalog.clone(),
            )
            .unwrap();
            battle.set_environment(Some("cave".into()));
            for step in 0..100 {
                let result = battle
                    .advance()
                    .unwrap_or_else(|error| panic!("{move_id}, seed {seed}: {error}"));
                if battle.turn() > 5 {
                    break;
                }
                let AdvanceStatus::Awaiting(prompt) = result.status else {
                    break;
                };
                let choices = &prompt.choices;
                let choice = &choices[(seed as usize + step) % choices.len()];
                battle
                    .set_response(ActionSelection {
                        prompt_id: prompt.id,
                        choice_id: choice.id(),
                    })
                    .unwrap();
            }
            for side in Side::ALL {
                for pokemon in battle.participants(side) {
                    assert!(pokemon.hp <= pokemon.max_hp, "{move_id}");
                }
            }
        }
    }
}

#[test]
fn spread_hits_do_not_apply_secondaries_to_immune_targets() {
    let mut immune = mon("Ground", &["splash"]);
    immune.types = vec![PokemonType::Ground];
    let mut game = Game::new(
        vec![mon("Ally", &["discharge"])],
        vec![mon("Foe", &["splash"]), immune],
        0.1,
    );
    game.turn("discharge", "splash");
    assert!(game.foe().hp < 1000);
    assert_eq!(game.foe().status, Some(Status::Paralyzed));
    let immune = &game.0.participants(Side::Foes)[1];
    assert_eq!(immune.hp, 1000);
    assert_eq!(immune.status, None);
}

#[test]
fn damage_stat_changes_go_to_the_correct_pokemon() {
    let mut game = Game::duel(&["superpower", "overheat"], mon("Foe", &["splash"]), 0.5);
    game.turn("superpower", "splash");
    assert_eq!(
        (
            game.ally().stage(Stat::Attack),
            game.ally().stage(Stat::Defense)
        ),
        (-1, -1)
    );
    assert_eq!(game.foe().stage(Stat::Attack), 0);
    game.turn("overheat", "splash");
    assert_eq!(game.ally().stage(Stat::SpAttack), -2);
    assert_eq!(game.foe().stage(Stat::SpAttack), 0);
}

#[test]
fn no_retreat_boosts_and_traps_once_without_damage() {
    let mut game = Game::duel(&["no-retreat"], mon("Foe", &["splash"]), 0.5);
    game.turn("no-retreat", "splash");
    for stat in [
        Stat::Attack,
        Stat::Defense,
        Stat::SpAttack,
        Stat::SpDefense,
        Stat::Speed,
    ] {
        assert_eq!(game.ally().stage(stat), 1);
    }
    assert!(game.0.trapped(ParticipantId {
        side: Side::Allies,
        index: 0
    }));
    assert_eq!(game.foe().hp, 1000);
    assert!(said(&game.turn("no-retreat", "splash"), "failed"));
    assert_eq!(game.ally().stage(Stat::Attack), 1);
}

#[test]
fn anchoring_moves_leave_ghosts_free_and_record_other_traps() {
    for move_id in ["anchor-shot", "spirit-shackle"] {
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![PokemonType::Grass];
        let mut game = Game::duel(&[move_id], foe.clone(), 0.5);
        game.turn(move_id, "splash");
        assert!(
            game.0.trapped(ParticipantId {
                side: Side::Foes,
                index: 0
            }),
            "{move_id}"
        );
        foe.types = vec![PokemonType::Ghost];
        let mut game = Game::duel(&[move_id], foe, 0.5);
        game.turn(move_id, "splash");
        assert!(game.foe().hp < 1000, "{move_id}");
        assert!(
            !game.0.trapped(ParticipantId {
                side: Side::Foes,
                index: 0
            }),
            "{move_id}"
        );
    }
}

#[test]
fn secret_sword_uses_defense_and_ignores_special_defense() {
    let damage = |defense, sp_defense| {
        let mut foe = mon("Foe", &["splash"]);
        foe.defense = defense;
        foe.sp_defense = sp_defense;
        let mut game = Game::duel(&["secret-sword"], foe, 0.5);
        game.turn("secret-sword", "splash");
        1000 - game.foe().hp
    };
    assert_eq!(damage(100, 30), damage(100, 300));
    assert!(damage(30, 100) > damage(300, 100));
}

#[test]
fn echoed_voice_shares_one_counter_per_turn_and_resets_after_a_gap() {
    let mut game = Game::duel(
        &["echoed-voice", "splash"],
        mon("Foe", &["echoed-voice", "splash"]),
        0.5,
    );
    game.turn("echoed-voice", "echoed-voice");
    let base = 1000 - game.ally().hp;
    assert_eq!(base, 1000 - game.foe().hp);
    let before = game.ally().hp;
    game.turn("splash", "echoed-voice");
    assert!(before - game.ally().hp > base);
    game.turn("splash", "splash");
    let before = game.ally().hp;
    game.turn("splash", "echoed-voice");
    assert_eq!(before - game.ally().hp, base);
}

#[test]
fn sheer_cold_respects_ice_immunity_and_level() {
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Ice];
    let mut game = Game::duel(&["sheer-cold"], foe.clone(), 0.1);
    game.turn("sheer-cold", "splash");
    assert_eq!(game.foe().hp, 1000);
    foe.types = vec![PokemonType::Normal];
    foe.level = 100;
    let mut user = mon("Ally", &["sheer-cold"]);
    user.level = 50;
    let mut game = Game::new(vec![user], vec![foe.clone()], 0.1);
    game.turn("sheer-cold", "splash");
    assert_eq!(game.foe().hp, 1000);
    foe.level = 1;
    let mut game = Game::duel(&["sheer-cold"], foe, 0.1);
    game.turn("sheer-cold", "splash");
    assert_eq!(game.foe().hp, 0);
}

#[test]
fn guaranteed_critical_moves_still_respect_accuracy() {
    // These rolls are above the ordinary and increased critical-hit thresholds.
    for (move_id, roll, lands) in [
        ("frost-breath", 0.5, true),
        ("frost-breath", 0.85, true),
        ("frost-breath", 0.95, false),
        ("wicked-blow", 0.95, true),
    ] {
        let mut game = Game::duel(&[move_id], mon("Foe", &["splash"]), roll);
        let events = game.turn(move_id, "splash");
        assert_eq!(game.foe().hp < 1000, lands, "{move_id}, roll {roll}");
        assert_eq!(said(&events, "critical"), lands, "{move_id}, roll {roll}");
    }
}

#[test]
fn snipe_shot_is_water_damage_with_increased_crit_and_no_screen() {
    let mut game = Game::duel(&["snipe-shot"], mon("Foe", &["splash"]), 0.1);
    let events = game.turn("snipe-shot", "splash");
    assert!(said(&events, "critical"));
    assert!(game.foe().hp < 1000);
    let catalog = MoveCatalog::builtin().unwrap();
    assert_eq!(
        catalog.get("snipe-shot").unwrap().move_type,
        PokemonType::Water
    );
    assert!(game.0.side_conditions(Side::Allies).is_empty());
}

#[test]
fn powder_does_not_affect_grass_and_poison_fang_badly_poisons() {
    let mut grass = mon("Grass", &["splash"]);
    grass.types = vec![PokemonType::Grass];
    let mut game = Game::duel(&["spore", "poison-fang"], grass, 0.1);
    game.turn("spore", "splash");
    assert_eq!(game.foe().status, None);
    game.turn("poison-fang", "splash");
    assert_eq!(game.foe().status, Some(Status::BadlyPoisoned));
}

#[test]
fn triple_axel_checks_each_hit_and_surging_strikes_always_crits() {
    let mut game = Game::duel(
        &["triple-axel", "surging-strikes"],
        mon("Foe", &["splash"]),
        0.5,
    );
    let events = game.turn("triple-axel", "splash");
    let hits: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::Damage { target, amount, .. } if target.side == Side::Foes => {
                Some(*amount)
            }
            _ => None,
        })
        .collect();
    assert_eq!(hits.len(), 3);
    assert!(hits[0] < hits[1] && hits[1] < hits[2]);
    let events = game.turn("surging-strikes", "splash");
    assert!(said(&events, "critical"));
    assert_eq!(events.iter().filter(|event| matches!(event, BattleEvent::Damage { target, .. } if target.side == Side::Foes)).count(), 3);
    let mut misses = Game::duel(&["triple-axel"], mon("Foe", &["splash"]), 0.95);
    misses.turn("triple-axel", "splash");
    assert_eq!(misses.foe().hp, 1000);
}

#[test]
fn special_damage_and_stat_changes_have_concrete_effects() {
    let mut game = Game::duel(&["scale-shot", "meteor-beam"], mon("Foe", &["splash"]), 0.5);
    game.turn("scale-shot", "splash");
    assert_eq!(
        (
            game.ally().stage(Stat::Speed),
            game.ally().stage(Stat::Defense)
        ),
        (1, -1)
    );
    let before = game.foe().hp;
    game.turn("meteor-beam", "splash");
    assert_eq!(game.foe().hp, before);
    assert_eq!(game.ally().stage(Stat::SpAttack), 1);
    game.turn("meteor-beam", "splash");
    assert!(game.foe().hp < before);
}

#[test]
fn berry_moves_report_consumption_and_unlock_belch() {
    let mut foe = mon("Foe", &["splash"]);
    foe.item = Some("oran-berry".into());
    let mut game = Game::duel(&["belch", "bug-bite"], foe, 0.5);
    assert!(said(&game.turn("belch", "splash"), "failed"));
    assert_eq!(game.foe().hp, 1000);
    game.turn("bug-bite", "splash");
    assert!(game.foe().item.is_none());
    let before = game.foe().hp;
    game.turn("belch", "splash");
    assert!(game.foe().hp < before);
}

#[test]
fn unavailable_moves_fail_explicitly_without_fake_effects() {
    for id in ["sky-drop", "stuff-cheeks"] {
        let mut game = Game::duel(&[id], mon("Foe", &["splash"]), 0.5);
        let events = game.turn(id, "splash");
        assert!(said(&events, "unavailable"), "{id}");
        assert_eq!(game.foe().hp, 1000);
        assert_eq!(game.ally().stage(Stat::Defense), 0);
    }
}

#[test]
fn naturally_grounded_pokemon_receive_rototiller_but_flying_ones_do_not() {
    let mut grass = mon("Grass", &["rototiller"]);
    grass.types = vec![PokemonType::Grass];
    let mut flyer = mon("Flying grass", &["splash"]);
    flyer.types = vec![PokemonType::Grass, PokemonType::Flying];
    let mut game = Game::new(vec![grass], vec![flyer], 0.5);
    game.turn("rototiller", "splash");
    assert_eq!(game.ally().stage(Stat::Attack), 1);
    assert_eq!(game.foe().stage(Stat::Attack), 0);
}
