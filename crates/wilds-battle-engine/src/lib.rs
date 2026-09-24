mod engine;
mod lua_symbols;
mod model;
mod moves;
mod type_chart;

pub use engine::Battle;
pub use model::{
    ActionSelection, AdvanceResult, AdvanceStatus, AppliedStatus, BattleError, BattleEvent, Bound,
    Choice, CombatStat, LockedMove, ParticipantId, Pokemon, PokemonType, Prompt, Side, Stat,
    Status, Weather, WeatherKind,
};
pub use moves::{Accuracy, Category, Effect, MoveCatalog, MoveSpec, Target};

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
    fn pick(battle: &mut Battle, move_id: &str) {
        let result = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            panic!("expected prompt")
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| c.move_id == move_id)
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id,
            })
            .unwrap();
    }
    #[test]
    fn loads_original_and_scripted_moves() {
        let catalog = MoveCatalog::builtin().unwrap();
        assert_eq!(catalog.len(), 34);
        let kiss = catalog.get("draining_kiss").unwrap();
        assert_eq!(kiss.category, Category::Physical);
        assert_eq!(catalog.get("protect").unwrap().priority, 4);
        for id in [
            "dream_eater",
            "false_swipe",
            "triple_kick",
            "explosion",
            "hyper_beam",
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
        for move_id in ids {
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
        assert!(battle.participants(Side::Allies)[0].locked_move.is_some());
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
}
