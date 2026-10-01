mod engine;
mod lua_symbols;
mod model;
mod moves;
mod rng;
mod type_chart;

pub use engine::Battle;
pub use model::{
    ActionSelection, AdvanceResult, AdvanceStatus, AppliedStatus, BattleError, BattleEvent, Bound,
    Choice, CombatStat, ContinuationTarget, LockedMove, NextMovePowerBoost, ParticipantId, Pokemon,
    PokemonType, Prompt, ScriptContinuation, Side, Stat, Status, Weather, WeatherKind,
};
pub use moves::{Accuracy, Category, Effect, MoveCatalog, MoveSpec, Target};
pub use rng::{BattleRng, SeededRng};

#[cfg(test)]
mod tests {
    use super::*;

    fn id(side: Side) -> ParticipantId {
        ParticipantId { side, index: 0 }
    }
    fn one_on_one(seed: u64, ally: &str, foe: &str) -> Battle {
        Battle::new(
            seed,
            [
                vec![Pokemon::new("Ally", vec![ally.into()])],
                vec![Pokemon::new("Foe", vec![foe.into()])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap()
    }

    struct FixedRng(f32);

    impl BattleRng for FixedRng {
        fn next_u64(&mut self) -> u64 {
            0
        }

        fn fraction(&mut self) -> f32 {
            self.0
        }
    }

    #[test]
    fn supplied_rng_controls_random_move_effects() {
        let run = |fraction| {
            let mut battle = Battle::with_rng(
                FixedRng(fraction),
                [
                    vec![Pokemon::new("Ally", vec!["splash".into()])],
                    vec![Pokemon::new("Foe", vec!["splash".into()])],
                ],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "splash");
            pick(&mut battle, "splash");
            battle.advance().unwrap();
            [
                battle.participants(Side::Allies)[0].hp,
                battle.participants(Side::Foes)[0].hp,
            ]
        };
        assert_eq!(run(0.5), [100, 100]);
        assert!(run(0.0).into_iter().all(|hp| hp < 100));
    }

    #[test]
    fn rage_reacts_to_each_hit_and_expires_at_the_next_action() {
        let mut ally = Pokemon::new("Ally", vec!["rage".into(), "splash".into()]);
        ally.hp = 300;
        ally.max_hp = 300;
        ally.speed = 200;
        let mut battle = Battle::with_rng(
            FixedRng(0.5),
            [
                vec![ally],
                vec![Pokemon::new("Foe", vec!["double_kick".into()])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();

        pick(&mut battle, "rage");
        pick(&mut battle, "double_kick");
        let first = battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 2);
        assert_eq!(
            first
                .events
                .iter()
                .filter(|event| matches!(event, BattleEvent::StatChange { target, stat: Stat::Attack, stages: 1 } if *target == id(Side::Allies)))
                .count(),
            2
        );

        pick(&mut battle, "splash");
        pick(&mut battle, "double_kick");
        battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 2);
    }

    #[test]
    fn rage_stays_active_until_a_slower_user_acts_again() {
        let mut ally = Pokemon::new("Ally", vec!["rage".into()]);
        ally.hp = 500;
        ally.max_hp = 500;
        ally.speed = 50;
        let mut foe = Pokemon::new("Foe", vec!["double_kick".into()]);
        foe.speed = 200;
        let mut battle = Battle::with_rng(
            FixedRng(0.5),
            [vec![ally], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "rage");
        pick(&mut battle, "double_kick");
        let first = battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 0);

        pick(&mut battle, "rage");
        pick(&mut battle, "double_kick");
        let second = battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 2);
        let foe_damage = |events: &[BattleEvent]| {
            events.iter().find_map(|event| match event {
                BattleEvent::Damage { target, amount, .. } if *target == id(Side::Foes) => {
                    Some(*amount)
                }
                _ => None,
            })
        };
        assert!(foe_damage(&second.events) > foe_damage(&first.events));

        pick(&mut battle, "rage");
        pick(&mut battle, "double_kick");
        battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 4);
    }

    #[test]
    fn rage_does_not_react_to_residual_status_damage() {
        let mut ally = Pokemon::new("Ally", vec!["rage".into()]);
        ally.status = Some(Status::Poisoned);
        let mut battle = Battle::with_rng(
            FixedRng(0.5),
            [vec![ally], vec![Pokemon::new("Foe", vec!["splash".into()])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "rage");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        assert!(battle.participants(Side::Allies)[0].hp < 100);
        assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    }
    fn pick(battle: &mut Battle, move_id: &str) {
        let result = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            panic!("expected prompt")
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { move_id: id, .. } if id == move_id))
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id(),
            })
            .unwrap();
    }
    #[test]
    fn loads_original_and_scripted_moves() {
        let catalog = MoveCatalog::builtin().unwrap();
        assert_eq!(catalog.len(), 42);
        let from_files =
            MoveCatalog::from_directory(concat!(env!("CARGO_MANIFEST_DIR"), "/../../moves"))
                .unwrap();
        assert_eq!(from_files.len(), catalog.len());
        for id in catalog.ids() {
            assert!(from_files.get(id).is_some(), "missing {id}");
        }
        assert_eq!(
            catalog.get("struggle").unwrap().move_type,
            PokemonType::Normal
        );
        let kiss = catalog.get("draining_kiss").unwrap();
        assert_eq!(kiss.category, Category::Physical);
        assert_eq!(catalog.get("protect").unwrap().priority, 4);
        for id in [
            "dream_eater",
            "false_swipe",
            "triple_kick",
            "explosion",
            "hyper_beam",
            "solar_beam",
            "thrash",
            "charge",
            "rage",
        ] {
            assert!(catalog.get(id).unwrap().script.is_some());
        }
    }
    #[test]
    fn rejects_wrong_lua_enum_families() {
        let base = r#"return {{ id = 'test', name = 'Test', type = Type.Normal, category = Category.Status, pp = 1, effects = {{ kind = Effect.Status, status = Status.Poisoned }} }}"#;
        for (source, expected) in [
            (
                base.replace("type = Type.Normal", "type = Stat.Attack"),
                "type requires Type value",
            ),
            (
                base.replace("status = Status.Poisoned", "status = Weather.Sun"),
                "status requires Status value",
            ),
            (
                base.replace("kind = Effect.Status", "kind = Category.Status"),
                "kind requires Effect value",
            ),
            (
                base.replace(
                    "kind = Effect.Status, status = Status.Poisoned",
                    "kind = Effect.Stats, stages = {[Type.Normal] = 1}",
                ),
                "stage requires Stat value",
            ),
        ] {
            let error = MoveCatalog::from_lua(&source).unwrap_err();
            assert!(matches!(error, BattleError::InvalidSetup(_)));
            assert_eq!(error.to_string(), expected);
        }
    }
    #[test]
    fn prompts_both_sides_and_rejects_stale_responses() {
        let mut battle = one_on_one(10, "tackle", "tackle");
        let first = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("expected prompt")
        };
        assert_eq!(prompt.actor, id(Side::Allies));
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        assert!(
            battle
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: 0
                })
                .is_err()
        );
        let second = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = second.status else {
            panic!("expected prompt")
        };
        assert_eq!(prompt.actor, id(Side::Foes));
    }
    #[test]
    fn damage_and_recoil_resolve_in_order() {
        let mut battle = one_on_one(3, "double_edge", "splash");
        pick(&mut battle, "double_edge");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(
            result.events.iter().any(
                |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Foes))
            )
        );
        assert!(result.events.iter().any(
            |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Allies))
        ));
    }

    #[test]
    fn every_builtin_move_executes_a_turn() {
        let ids: Vec<String> = MoveCatalog::builtin()
            .unwrap()
            .ids()
            .map(str::to_owned)
            .collect();
        for move_id in ids.into_iter().filter(|id| id != "struggle") {
            let mut battle = one_on_one(31, &move_id, "splash");
            pick(&mut battle, &move_id);
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            assert!(!result.events.is_empty(), "{move_id}");
        }
    }

    #[test]
    fn protect_runs_before_a_faster_attack() {
        let mut battle = one_on_one(1, "protect", "tackle");
        pick(&mut battle, "protect");
        pick(&mut battle, "tackle");
        let result = battle.advance().unwrap();
        assert!(!result.events.iter().any(
            |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Allies))
        ));
    }

    #[test]
    fn fixed_damage_and_multi_hit_keep_their_csharp_values() {
        let mut fixed = one_on_one(9, "dragon_rage", "splash");
        pick(&mut fixed, "dragon_rage");
        pick(&mut fixed, "splash");
        let result = fixed.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 40, .. } if *target == id(Side::Foes))));

        let mut multi = one_on_one(9, "double_kick", "splash");
        pick(&mut multi, "double_kick");
        pick(&mut multi, "splash");
        let result = multi.advance().unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .filter(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes)))
                .count(),
            2
        );
    }

    #[test]
    fn toxic_damages_at_turn_end_and_rest_replaces_status() {
        let mut battle = one_on_one(7, "toxic", "splash");
        pick(&mut battle, "toxic");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(
            battle.participants(Side::Foes)[0].status,
            Some(Status::BadlyPoisoned)
        );
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));

        let mut ally = Pokemon::new("Ally", vec!["rest".into()]);
        ally.hp = 35;
        ally.status = Some(Status::Poisoned);
        let mut rest = Battle::new(
            9,
            [vec![ally], vec![Pokemon::new("Foe", vec!["splash".into()])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut rest, "rest");
        pick(&mut rest, "splash");
        rest.advance().unwrap();
        let ally = &rest.participants(Side::Allies)[0];
        assert_eq!(ally.status, Some(Status::Asleep));
        assert_eq!(ally.hp, ally.max_hp);
        assert_eq!(ally.status_turns, 2);
    }

    #[test]
    fn solar_beam_charges_then_acts_without_a_second_choice() {
        let mut battle = one_on_one(5, "solar_beam", "splash");
        pick(&mut battle, "solar_beam");
        pick(&mut battle, "splash");
        let first = battle.advance().unwrap();
        assert!(
            first
                .events
                .iter()
                .any(|event| matches!(event, BattleEvent::Message(message)
            if message.contains("took in sunlight")))
        );
        assert!(
            battle.participants(Side::Allies)[0]
                .script_continuation
                .is_some()
        );
        assert_eq!(
            battle.participants(Side::Allies)[0].move_pp["solar_beam"],
            9
        );
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("foe should choose")
        };
        assert_eq!(prompt.actor, id(Side::Foes));
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        let second = battle.advance().unwrap();
        assert!(second.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
        assert_eq!(
            battle.participants(Side::Allies)[0].move_pp["solar_beam"],
            9
        );
        assert!(
            battle.participants(Side::Allies)[0]
                .script_continuation
                .is_none()
        );
    }

    fn thrash_battle(seed: u64) -> Battle {
        let ally = Pokemon::new("Ally", vec!["thrash".into(), "splash".into()]);
        let mut foe = Pokemon::new("Foe", vec!["splash".into(), "protect".into()]);
        foe.hp = 1000;
        foe.max_hp = 1000;
        Battle::new(
            seed,
            [vec![ally], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn thrash_forces_two_or_three_turns_and_spends_one_pp() {
        let mut battle = thrash_battle(12);
        pick(&mut battle, "thrash");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        let lock = battle.participants(Side::Allies)[0]
            .script_continuation
            .as_ref()
            .unwrap();
        let duration = lock.total_turns;
        assert!((2..=3).contains(&duration));
        assert_eq!(lock.turn, 2);
        assert_eq!(battle.participants(Side::Allies)[0].move_pp["thrash"], 9);
        for turn in 2..=duration {
            pick(&mut battle, "splash");
            battle.advance().unwrap();
            let ally = &battle.participants(Side::Allies)[0];
            assert_eq!(ally.move_pp["thrash"], 9);
            if turn < duration {
                assert_eq!(ally.script_continuation.as_ref().unwrap().turn, turn + 1);
                assert!(ally.confused_turns.is_none());
            } else {
                assert!(ally.script_continuation.is_none());
                assert!(ally.confused_turns.is_some());
            }
        }
    }

    #[test]
    fn thrash_protect_disrupts_early_but_confuses_on_final_turn() {
        let mut early = thrash_battle(12);
        pick(&mut early, "thrash");
        pick(&mut early, "protect");
        early.advance().unwrap();
        let ally = &early.participants(Side::Allies)[0];
        assert!(ally.script_continuation.is_none());
        assert!(ally.confused_turns.is_none());
        assert_eq!(ally.move_pp["thrash"], 9);

        let mut final_turn = thrash_battle(12);
        pick(&mut final_turn, "thrash");
        pick(&mut final_turn, "splash");
        final_turn.advance().unwrap();
        let duration = final_turn.participants(Side::Allies)[0]
            .script_continuation
            .as_ref()
            .unwrap()
            .total_turns;
        for turn in 2..=duration {
            pick(
                &mut final_turn,
                if turn == duration {
                    "protect"
                } else {
                    "splash"
                },
            );
            final_turn.advance().unwrap();
        }
        let ally = &final_turn.participants(Side::Allies)[0];
        assert!(ally.script_continuation.is_none());
        assert!(ally.confused_turns.is_some());
        assert_eq!(ally.move_pp["thrash"], 9);
    }

    #[test]
    fn thrash_stops_on_ghost_immunity() {
        let ally = Pokemon::new("Ally", vec!["thrash".into()]);
        let mut foe = Pokemon::new("Ghost", vec!["splash".into()]);
        foe.types = vec![PokemonType::Ghost];
        let mut battle =
            Battle::new(9, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "thrash");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        let ally = &battle.participants(Side::Allies)[0];
        assert!(ally.script_continuation.is_none());
        assert!(ally.confused_turns.is_none());
        assert_eq!(ally.move_pp["thrash"], 9);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 100);
    }

    #[test]
    fn final_turn_paralysis_still_confuses_after_interruption() {
        let mut found = false;
        for seed in 1..=32 {
            let mut ally = Pokemon::new("Ally", vec!["thrash".into()]);
            ally.status = Some(Status::Paralyzed);
            ally.move_pp.insert("thrash".into(), 9);
            ally.script_continuation = Some(ScriptContinuation {
                move_id: "thrash".into(),
                target: id(Side::Foes),
                turn: 2,
                total_turns: 2,
                target_policy: ContinuationTarget::RandomOpponent,
            });
            let foe = Pokemon::new("Foe", vec!["splash".into()]);
            let mut battle = Battle::new(
                seed,
                [vec![ally], vec![foe]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            if result.events.iter().any(|event| matches!(event, BattleEvent::Message(message) if message.contains("can't move"))) {
                let ally = &battle.participants(Side::Allies)[0];
                assert!(ally.script_continuation.is_none());
                assert!(ally.confused_turns.is_some());
                assert_eq!(ally.move_pp["thrash"], 9);
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn thrash_can_choose_each_opponent_on_forced_turns() {
        let mut targets = std::collections::BTreeSet::new();
        for seed in 1..=32 {
            let ally = Pokemon::new("Ally", vec!["thrash".into()]);
            let mut first = Pokemon::new("First", vec!["splash".into()]);
            first.hp = 1000;
            first.max_hp = 1000;
            let mut second = Pokemon::new("Second", vec!["splash".into()]);
            second.hp = 1000;
            second.max_hp = 1000;
            let mut battle = Battle::new(
                seed,
                [vec![ally], vec![first, second]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "thrash");
            pick(&mut battle, "splash");
            pick(&mut battle, "splash");
            battle.advance().unwrap();
            pick(&mut battle, "splash");
            pick(&mut battle, "splash");
            for event in battle.advance().unwrap().events {
                if let BattleEvent::Damage { target, .. } = event
                    && target.side == Side::Foes
                {
                    targets.insert(target.index);
                }
            }
        }
        assert_eq!(targets, [0, 1].into());
    }

    fn exhausted_battle(seed: u64, ally_type: PokemonType, foe_move: &str) -> Battle {
        let mut ally = Pokemon::new("Ally", vec!["splash".into()]);
        ally.max_hp = 202;
        ally.hp = 202;
        ally.types = vec![ally_type];
        ally.move_pp.insert("splash".into(), 0);
        let mut foe = Pokemon::new("Foe", vec![foe_move.into()]);
        foe.max_hp = 1000;
        foe.hp = 1000;
        foe.types = vec![PokemonType::Ghost];
        Battle::new(
            seed,
            [vec![ally], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn exhausted_pokemon_uses_typeless_struggle_with_max_hp_recoil() {
        let mut normal = exhausted_battle(5, PokemonType::Normal, "splash");
        pick(&mut normal, "splash");
        let result = normal.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Message(message) if message == "Ally used Struggle!")));
        assert_eq!(normal.participants(Side::Allies)[0].hp, 151);
        assert_eq!(normal.participants(Side::Allies)[0].move_pp["splash"], 0);
        let damage = 1000 - normal.participants(Side::Foes)[0].hp;
        assert!(damage > 0, "Struggle should hit Ghost types");

        let mut fire = exhausted_battle(5, PokemonType::Fire, "splash");
        pick(&mut fire, "splash");
        fire.advance().unwrap();
        assert_eq!(fire.participants(Side::Foes)[0].hp, 1000 - damage);
    }

    #[test]
    fn protected_target_prevents_struggle_recoil() {
        let mut battle = exhausted_battle(8, PokemonType::Normal, "protect");
        pick(&mut battle, "protect");
        battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].hp, 202);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    }

    #[test]
    fn struggle_is_not_learnable() {
        let result = Battle::new(
            1,
            [
                vec![Pokemon::new("Ally", vec!["struggle".into()])],
                vec![Pokemon::new("Foe", vec!["splash".into()])],
            ],
            MoveCatalog::builtin().unwrap(),
        );
        assert!(
            matches!(result, Err(BattleError::InvalidSetup(message)) if message.contains("cannot be learned"))
        );
    }

    #[test]
    fn both_exhausted_pokemon_finish_with_all_turn_events() {
        let mut first = Pokemon::new("First", vec!["splash".into()]);
        first.move_pp.insert("splash".into(), 0);
        let mut second = Pokemon::new("Second", vec!["splash".into()]);
        second.move_pp.insert("splash".into(), 0);
        let mut battle = Battle::new(
            2,
            [vec![first], vec![second]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        let result = battle.advance().unwrap();
        assert!(matches!(result.status, AdvanceStatus::End { .. }));
        assert!(result.events.iter().filter(|event| matches!(event, BattleEvent::Message(message) if message.contains("used Struggle"))).count() >= 2);
    }

    #[test]
    fn advance_preserves_events_through_automatic_turns_until_prompt() {
        let mut found_three_turns = false;
        for seed in 1..=64 {
            let mut ally = Pokemon::new("Ally", vec!["thrash".into(), "splash".into()]);
            ally.max_hp = 1000;
            ally.hp = 1000;
            let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
            foe.max_hp = 1000;
            foe.hp = 1000;
            foe.move_pp.insert("splash".into(), 1);
            let mut battle = Battle::new(
                seed,
                [vec![ally], vec![foe]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "thrash");
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            if battle.turn() != 3 {
                continue;
            }
            found_three_turns = true;
            let AdvanceStatus::Awaiting(prompt) = result.status else {
                panic!("expected ally prompt after Thrash")
            };
            assert_eq!(prompt.actor, id(Side::Allies));
            let messages: Vec<&str> = result
                .events
                .iter()
                .filter_map(|event| match event {
                    BattleEvent::Message(message) => Some(message.as_str()),
                    _ => None,
                })
                .collect();
            let thrash_positions: Vec<_> = messages
                .iter()
                .enumerate()
                .filter_map(|(i, message)| (*message == "Ally used Thrash!").then_some(i))
                .collect();
            let struggle_positions: Vec<_> = messages
                .iter()
                .enumerate()
                .filter_map(|(i, message)| (*message == "Foe used Struggle!").then_some(i))
                .collect();
            let recoil_positions: Vec<_> = messages
                .iter()
                .enumerate()
                .filter_map(|(i, message)| {
                    (*message == "Foe was damaged by the recoil!").then_some(i)
                })
                .collect();
            assert_eq!(thrash_positions.len(), 3);
            assert_eq!(struggle_positions.len(), 2);
            assert_eq!(recoil_positions.len(), 2);
            assert!(thrash_positions[0] < struggle_positions[0]);
            assert!(struggle_positions[0] < recoil_positions[0]);
            assert!(recoil_positions[0] < struggle_positions[1]);
            assert!(struggle_positions[1] < recoil_positions[1]);
            break;
        }
        assert!(found_three_turns, "expected a seeded three-turn Thrash");
    }

    #[test]
    fn charge_raises_special_defense_and_sets_a_typed_next_move_boost() {
        let mut battle = one_on_one(7, "charge", "splash");
        pick(&mut battle, "charge");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        let ally = &battle.participants(Side::Allies)[0];
        assert_eq!(ally.stage(Stat::SpDefense), 1);
        assert_eq!(
            ally.next_move_power_boost,
            Some(NextMovePowerBoost {
                move_type: PokemonType::Electric,
                multiplier: 2.0,
            })
        );
        assert!(
            result
                .events
                .iter()
                .any(|event| matches!(event, BattleEvent::StatChange {
            target, stat: Stat::SpDefense, stages: 1,
        } if *target == id(Side::Allies)))
        );
    }

    fn electric_attack_damage(seed: u64, boosted: bool) -> (u16, Option<NextMovePowerBoost>) {
        let mut ally = Pokemon::new("Ally", vec!["thunder_shock".into()]);
        if boosted {
            ally.next_move_power_boost = Some(NextMovePowerBoost {
                move_type: PokemonType::Electric,
                multiplier: 2.0,
            });
        }
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.hp = 1000;
        foe.max_hp = 1000;
        let mut battle = Battle::new(
            seed,
            [vec![ally], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "thunder_shock");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        (
            1000 - battle.participants(Side::Foes)[0].hp,
            battle.participants(Side::Allies)[0].next_move_power_boost,
        )
    }

    #[test]
    fn charge_boosts_electric_power_once() {
        let (normal, _) = electric_attack_damage(14, false);
        let (charged, remaining) = electric_attack_damage(14, true);
        assert!(
            charged > normal + normal / 2,
            "expected roughly double power: {normal} -> {charged}"
        );
        assert_eq!(remaining, None);
    }

    #[test]
    fn charge_expires_after_a_non_electric_move() {
        let ally = Pokemon::new(
            "Ally",
            vec!["charge".into(), "tackle".into(), "thunder_shock".into()],
        );
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.hp = 1000;
        foe.max_hp = 1000;
        let mut battle =
            Battle::new(17, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "charge");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        assert!(
            battle.participants(Side::Allies)[0]
                .next_move_power_boost
                .is_some()
        );
        pick(&mut battle, "tackle");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        assert!(
            battle.participants(Side::Allies)[0]
                .next_move_power_boost
                .is_none()
        );
    }

    #[test]
    fn thrash_continues_after_spending_its_last_pp() {
        let mut ally = Pokemon::new("Ally", vec!["thrash".into()]);
        ally.move_pp.insert("thrash".into(), 1);
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.max_hp = 1000;
        foe.hp = 1000;
        let mut battle =
            Battle::new(12, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "thrash");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].move_pp["thrash"], 0);
        assert!(
            battle.participants(Side::Allies)[0]
                .script_continuation
                .is_some()
        );
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(
            |event| matches!(event, BattleEvent::Message(message) if message == "Ally used Thrash!")
        ));
    }

    #[test]
    fn sandstorm_damages_unprotected_types_at_turn_end() {
        let mut battle = one_on_one(4, "sandstorm", "splash");
        pick(&mut battle, "sandstorm");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(battle.weather().unwrap().kind, WeatherKind::Sandstorm);
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 6, .. } if *target == id(Side::Allies))));
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 6, .. } if *target == id(Side::Foes))));
    }

    #[test]
    fn self_stat_effect_survives_a_target_ko() {
        let mut found = false;
        for seed in 1..2000 {
            let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
            foe.hp = 1;
            let mut battle = Battle::new(
                seed,
                [
                    vec![Pokemon::new("Ally", vec!["ancient_power".into()])],
                    vec![foe],
                ],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "ancient_power");
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            if result.events.iter().any(
                |event| matches!(event, BattleEvent::StatChange { target, .. } if *target == id(Side::Allies)),
            ) {
                assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 1);
                assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Speed), 1);
                assert_eq!(battle.participants(Side::Foes)[0].hp, 0);
                found = true;
                break;
            }
        }
        assert!(found, "expected at least one seeded secondary effect");
    }

    #[test]
    fn ohko_respects_level_and_full_health_recovery_fails() {
        let mut ally = Pokemon::new("Ally", vec!["fissure".into()]);
        ally.level = 10;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.level = 50;
        let mut battle =
            Battle::new(1, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "fissure");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(!result.events.iter().any(
            |event| matches!(event, BattleEvent::Damage { target, .. } if *target == id(Side::Foes))
        ));

        let mut battle = one_on_one(1, "recover", "splash");
        pick(&mut battle, "recover");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(
            |event| matches!(event, BattleEvent::Message(message) if message == "But it failed!")
        ));
    }

    #[test]
    fn dive_avoids_an_attack_while_charging_and_hits_next_turn() {
        let mut user = Pokemon::new("Ally", vec!["dive".into()]);
        user.speed = 200;
        let mut battle = Battle::new(
            13,
            [vec![user], vec![Pokemon::new("Foe", vec!["tackle".into()])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "dive");
        pick(&mut battle, "tackle");
        let first = battle.advance().unwrap();
        assert!(battle.participants(Side::Allies)[0].semi_invulnerable);
        assert!(!first.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Allies))));
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("foe should choose")
        };
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        let second = battle.advance().unwrap();
        assert!(second.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
        assert!(!battle.participants(Side::Allies)[0].semi_invulnerable);
    }

    #[test]
    fn sunny_day_skips_solar_beam_charge() {
        let ally = Pokemon::new("Ally", vec!["sunny_day".into(), "solar_beam".into()]);
        let foe = Pokemon::new("Foe", vec!["splash".into()]);
        let mut battle =
            Battle::new(9, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "sunny_day");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        pick(&mut battle, "solar_beam");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
        assert!(battle.participants(Side::Allies)[0].locked_move.is_none());
    }

    #[test]
    fn bind_applies_residual_damage() {
        let mut battle = one_on_one(8, "bind", "splash");
        pick(&mut battle, "bind");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(battle.participants(Side::Foes)[0].bound.is_some());
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 12, .. } if *target == id(Side::Foes))));
    }

    #[test]
    fn drain_heals_and_storm_throw_always_crits() {
        let mut ally = Pokemon::new("Ally", vec!["draining_kiss".into()]);
        ally.hp = 50;
        let foe = Pokemon::new("Foe", vec!["splash".into()]);
        let mut battle =
            Battle::new(3, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "draining_kiss");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Heal { target, .. } if *target == id(Side::Allies))));

        let mut battle = one_on_one(3, "storm_throw", "splash");
        pick(&mut battle, "storm_throw");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Message(message) if message == "A critical hit!")));
    }

    #[test]
    fn recoil_can_faint_both_and_uses_current_csharp_tie_outcome() {
        let mut ally = Pokemon::new("Ally", vec!["double_edge".into()]);
        ally.hp = 1;
        ally.speed = 200;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.hp = 1;
        let mut battle =
            Battle::new(2, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "double_edge");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(
            result.status,
            AdvanceStatus::End {
                winner: Some(Side::Allies)
            }
        );
        assert_eq!(battle.participants(Side::Allies)[0].hp, 0);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 0);
    }

    #[test]
    fn dream_eater_checks_sleep_and_drains_damage() {
        let mut awake = one_on_one(7, "dream_eater", "splash");
        pick(&mut awake, "dream_eater");
        pick(&mut awake, "splash");
        let result = awake.advance().unwrap();
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "But it failed!"))
        );
        assert_eq!(awake.participants(Side::Foes)[0].hp, 100);

        let mut user = Pokemon::new("Ally", vec!["dream_eater".into()]);
        user.hp = 50;
        user.speed = 200;
        let mut target = Pokemon::new("Foe", vec!["splash".into()]);
        target.status = Some(Status::Asleep);
        target.status_turns = 2;
        let mut battle = Battle::new(
            7,
            [vec![user], vec![target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "dream_eater");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        assert!(battle.participants(Side::Foes)[0].hp < 100);
        assert!(battle.participants(Side::Allies)[0].hp > 50);
    }

    #[test]
    fn false_swipe_leaves_one_hp() {
        let mut battle = one_on_one(3, "false_swipe", "splash");
        pick(&mut battle, "false_swipe");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        let remaining = battle.participants(Side::Foes)[0].hp;
        assert!(remaining > 0 && remaining < 100);
        // Subsequent uses can bring the target to one HP but cannot faint it.
        for _ in 0..10 {
            pick(&mut battle, "false_swipe");
            pick(&mut battle, "splash");
            battle.advance().unwrap();
        }
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1);
    }

    #[test]
    fn explosion_faints_user_even_when_protected() {
        let mut battle = one_on_one(5, "explosion", "protect");
        pick(&mut battle, "explosion");
        pick(&mut battle, "protect");
        let result = battle.advance().unwrap();
        assert_eq!(battle.participants(Side::Allies)[0].hp, 0);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 100);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Fainted(target) if *target == id(Side::Allies)))
        );
    }

    #[test]
    fn explosion_hits_every_other_participant() {
        let mut user = Pokemon::new("User", vec!["explosion".into()]);
        user.speed = 200;
        let mut ally = Pokemon::new("Ally", vec!["splash".into()]);
        ally.max_hp = 1000;
        ally.hp = 1000;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.max_hp = 1000;
        foe.hp = 1000;
        let mut battle = Battle::new(
            1,
            [vec![user, ally], vec![foe]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        let first = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("expected prompt")
        };
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        pick(&mut battle, "splash");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        for target in [
            ParticipantId {
                side: Side::Allies,
                index: 1,
            },
            id(Side::Foes),
        ] {
            assert!(
                result.events.iter().any(
                    |e| matches!(e, BattleEvent::Damage { target: got, .. } if *got == target)
                )
            );
        }
        assert_eq!(battle.participants(Side::Allies)[0].hp, 0);
    }

    #[test]
    fn triple_kick_checks_each_hit_and_increases_power() {
        let mut complete = false;
        let mut stopped = false;
        for seed in 1..100 {
            let mut ally = Pokemon::new("Ally", vec!["triple_kick".into()]);
            ally.speed = 200;
            let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
            foe.hp = 1000;
            foe.max_hp = 1000;
            let mut battle = Battle::new(
                seed,
                [vec![ally], vec![foe]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "triple_kick");
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            let amounts: Vec<_> = result
                .events
                .iter()
                .filter_map(|e| match e {
                    BattleEvent::Damage { target, amount, .. } if *target == id(Side::Foes) => {
                        Some(*amount)
                    }
                    _ => None,
                })
                .collect();
            if amounts.len() == 3 {
                assert!(amounts[0] < amounts[1] && amounts[1] < amounts[2]);
                complete = true;
            }
            if amounts.len() < 3 {
                stopped = true;
            }
            if complete && stopped {
                break;
            }
        }
        assert!(complete && stopped);
    }

    #[test]
    fn hyper_beam_recharges_after_a_hit_without_a_prompt() {
        let mut ally = Pokemon::new("Ally", vec!["hyper_beam".into()]);
        ally.speed = 200;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.hp = 1000;
        foe.max_hp = 1000;
        let mut battle =
            Battle::new(2, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "hyper_beam");
        pick(&mut battle, "splash");
        let first = battle.advance().unwrap();
        assert!(
            first.events.iter().any(
                |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Foes))
            )
        );
        assert!(battle.participants(Side::Allies)[0].recharging);
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("expected next turn prompt")
        };
        assert_eq!(prompt.actor, id(Side::Foes));
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        let second = battle.advance().unwrap();
        assert!(
            second
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "Ally must recharge!"))
        );
        assert!(!battle.participants(Side::Allies)[0].recharging);
    }

    #[test]
    fn missed_hyper_beam_does_not_recharge() {
        let mut missed = false;
        for seed in 1..100 {
            let mut battle = one_on_one(seed, "hyper_beam", "splash");
            pick(&mut battle, "hyper_beam");
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            if result
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "Ally's attack missed!"))
            {
                assert!(!battle.participants(Side::Allies)[0].recharging);
                missed = true;
                break;
            }
        }
        assert!(missed);
    }

    #[test]
    fn scripted_moves_only_accept_api_operations() {
        let source = r#"return {{id='test', name='Test', type=Type.Normal, category=Category.Physical, pp=1,
            script=function(_) coroutine.yield({kind='damage', power=40}) end}}"#;
        let mut battle = Battle::new(
            1,
            [
                vec![Pokemon::new("Ally", vec!["test".into()])],
                vec![Pokemon::new("Foe", vec!["test".into()])],
            ],
            MoveCatalog::from_lua(source).unwrap(),
        )
        .unwrap();
        pick(&mut battle, "test");
        pick(&mut battle, "test");
        assert!(
            matches!(battle.advance(), Err(BattleError::InvalidSetup(message)) if message.contains("outside the battle API"))
        );
    }

    #[test]
    fn scripted_damage_rejects_unknown_options() {
        let source = r#"return {{id='test', name='Test', type=Type.Normal, category=Category.Physical, pp=1,
            script=function(ctx) ctx:damage(40, {min_target_hpp=1}) end}}"#;
        let mut battle = Battle::new(
            1,
            [
                vec![Pokemon::new("Ally", vec!["test".into()])],
                vec![Pokemon::new("Foe", vec!["test".into()])],
            ],
            MoveCatalog::from_lua(source).unwrap(),
        )
        .unwrap();
        pick(&mut battle, "test");
        pick(&mut battle, "test");
        assert!(
            matches!(battle.advance(), Err(BattleError::Script(error)) if error.to_string().contains("unknown damage option min_target_hpp"))
        );
    }

    #[test]
    fn scripted_moves_have_an_instruction_limit() {
        let source = r#"return {{id='test', name='Test', type=Type.Normal, category=Category.Physical, pp=1,
            script=function(_) while true do end end}}"#;
        let mut battle = Battle::new(
            1,
            [
                vec![Pokemon::new("Ally", vec!["test".into()])],
                vec![Pokemon::new("Foe", vec!["test".into()])],
            ],
            MoveCatalog::from_lua(source).unwrap(),
        )
        .unwrap();
        pick(&mut battle, "test");
        pick(&mut battle, "test");
        assert!(matches!(battle.advance(), Err(BattleError::Script(_))));
    }

    #[test]
    fn facade_uses_status_to_double_power() {
        let run = |status| {
            let mut ally = Pokemon::new("Ally", vec!["facade".into()]);
            ally.status = status;
            let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
            foe.hp = 500;
            foe.max_hp = 500;
            let mut battle = Battle::with_rng(
                FixedRng(0.5),
                [vec![ally], vec![foe]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "facade");
            pick(&mut battle, "splash");
            battle.advance().unwrap();
            500 - battle.participants(Side::Foes)[0].hp
        };
        assert!(run(Some(Status::Paralyzed)) > run(None));
    }

    #[test]
    fn snore_only_acts_during_sleep() {
        let run = |asleep| {
            let mut ally = Pokemon::new("Ally", vec!["snore".into()]);
            ally.speed = 200;
            if asleep {
                ally.status = Some(Status::Asleep);
                ally.status_turns = 3;
            }
            let mut battle = Battle::with_rng(
                FixedRng(0.5),
                [vec![ally], vec![Pokemon::new("Foe", vec!["splash".into()])]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "snore");
            pick(&mut battle, "splash");
            battle.advance().unwrap();
            battle.participants(Side::Foes)[0].hp
        };
        assert!(run(true) < run(false));
        assert_eq!(run(false), 100);
    }

    #[test]
    fn morning_sun_heals_more_in_sun() {
        let run = |sun| {
            let mut ally = Pokemon::new("Ally", vec!["morning_sun".into(), "splash".into()]);
            ally.hp = 10;
            ally.speed = 200;
            let foe_move = if sun { "sunny_day" } else { "splash" };
            let foe = Pokemon::new("Foe", vec![foe_move.into()]);
            let mut battle = Battle::with_rng(
                FixedRng(0.5),
                [vec![ally], vec![foe]],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            if sun {
                pick(&mut battle, "splash");
                pick(&mut battle, "sunny_day");
                battle.advance().unwrap();
            }
            pick(&mut battle, "morning_sun");
            pick(&mut battle, foe_move);
            battle.advance().unwrap();
            battle.participants(Side::Allies)[0].hp
        };
        assert!(run(true) > run(false));
    }
}
