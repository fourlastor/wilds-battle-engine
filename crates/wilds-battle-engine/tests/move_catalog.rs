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
fn all_sheet_ids_have_unique_canonical_files_and_matching_display_names() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../moves/catalog.json")).unwrap();
    let entries = manifest["moves"].as_array().unwrap();
    assert_eq!(entries.len(), 572);
    let builtin = MoveCatalog::builtin().unwrap();
    let disk = MoveCatalog::from_directory(root.join("moves")).unwrap();
    assert_eq!(builtin.len(), 581);
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
    let files: Vec<_> = std::fs::read_dir(root.join("moves"))
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "lua"))
        .collect();
    assert_eq!(files.len(), 580);
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
