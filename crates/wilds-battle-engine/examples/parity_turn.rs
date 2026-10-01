use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, Pokemon,
    Side,
};

struct Draws {
    chances: VecDeque<f32>,
    damage_roll: Option<f32>,
}

struct RecordedRng(Rc<RefCell<Draws>>);

impl BattleRng for RecordedRng {
    fn next_u64(&mut self) -> u64 {
        // Speed decides order in these fixtures, but the battle still draws
        // two tie breakers, as the C# engine does.
        0
    }

    fn fraction(&mut self) -> f32 {
        panic!("unexpected raw random fraction")
    }

    fn chance(&mut self, probability: f32) -> bool {
        let draw = self
            .0
            .borrow_mut()
            .chances
            .pop_front()
            .expect("missing chance draw");
        draw <= probability
    }

    fn damage_roll(&mut self) -> f32 {
        self.0
            .borrow_mut()
            .damage_roll
            .take()
            .expect("missing damage roll")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let case_id = args.next().expect("case id");
    let move_id = args.next().expect("move id");
    assert!(["dragon_rage", "tackle", "toxic", "recover"].contains(&move_id.as_str()));
    let chances = args
        .next()
        .expect("comma-separated chance draws")
        .split(',')
        .map(str::parse)
        .collect::<Result<VecDeque<f32>, _>>()?;
    let damage_roll = match args.next().expect("damage roll or none").as_str() {
        "none" => None,
        value => Some(value.parse()?),
    };
    assert!(args.next().is_none());
    let draws = Rc::new(RefCell::new(Draws {
        chances,
        damage_roll,
    }));

    let mut ally = Pokemon::new("Ally", vec![move_id.clone()]);
    ally.speed = 200;
    if case_id == "recover" {
        ally.hp = 50;
    }
    let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
    foe.speed = 100;
    if case_id == "dragon_rage_ko" {
        foe.hp = 35;
    }
    let mut battle = Battle::with_rng(
        RecordedRng(Rc::clone(&draws)),
        [vec![ally], vec![foe]],
        MoveCatalog::builtin()?,
    )?;

    for selected in [move_id.as_str(), "splash"] {
        let AdvanceStatus::Awaiting(prompt) = battle.advance()?.status else {
            panic!("expected a move prompt");
        };
        let choice = prompt
            .choices
            .iter()
            .find(|choice| matches!(choice, Choice::UseMove { move_id: id, .. } if id == selected))
            .expect("fixture move is available");
        battle.set_response(ActionSelection {
            prompt_id: prompt.id,
            choice_id: choice.id(),
        })?;
    }

    let events: Vec<_> = battle
        .advance()?
        .events
        .into_iter()
        .filter_map(|event| match event {
            BattleEvent::Message(message) => Some(message),
            BattleEvent::Damage { target, amount, .. } => {
                Some(format!("damage:{:?}:{amount}", target.side))
            }
            BattleEvent::Heal { target, amount, .. } => {
                Some(format!("heal:{:?}:{amount}", target.side))
            }
            BattleEvent::Status { target, status } => Some(format!(
                "status:{:?}:{}",
                target.side,
                status.map_or("None".into(), |s| format!("{s:?}"))
            )),
            BattleEvent::End { winner } => Some(format!(
                "win:{}",
                winner.map_or("None".into(), |side| format!("{side:?}"))
            )),
            _ => None,
        })
        .collect();
    let draws = draws.borrow();
    assert!(draws.chances.is_empty(), "unused chance draws");
    assert!(draws.damage_roll.is_none(), "unused damage roll");
    let foe = &battle.participants(Side::Foes)[0];
    println!("PARITY_RUST_{case_id}_EVENTS={}", events.join("|"));
    println!(
        "PARITY_RUST_{case_id}_HP={},{}",
        battle.participants(Side::Allies)[0].hp,
        foe.hp
    );
    println!(
        "PARITY_RUST_{case_id}_STATUS={},{}",
        foe.status.map_or("None".into(), |s| format!("{s:?}")),
        foe.status_turns
    );
    Ok(())
}
