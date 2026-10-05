// What the "All blocks" page lists: the three starting blocks, then every block of the palette in
// the palette's order. The explanation of a block is its tooltip (definitions.ts) and the lines it
// writes come from the generator, so this file only adds what neither can say: fuller examples for
// blocks that hold other blocks, and details too long for a tooltip.
import { HAT_TYPES } from './definitions.ts';
import type { BlockState } from './library.ts';
import { CATEGORIES, COLORS } from './toolbox.ts';

export interface ReferenceEntry {
  type: string;
  /** What to build for the picture and the lines. The entry shows the first `type` block in it. */
  state: BlockState;
  /** Details that do not fit the tooltip. */
  note?: string;
  /** A second look at a block that is already listed: the explanation is not repeated. */
  variant?: boolean;
}

export interface ReferenceSection {
  id: string;
  name: string;
  color: string;
  about: string;
  entries: ReferenceEntry[];
}

const number = (value: number): { shadow: BlockState } => ({ shadow: { type: 'mlab_number', fields: { NUM: value } } });
const kind = (type: string): { shadow: BlockState } => ({ shadow: { type: 'mlab_type', fields: { TYPE: type } } });
const damage = (power: number | BlockState): BlockState => ({
  type: 'mlab_damage',
  inputs: { POWER: typeof power === 'number' ? number(power) : { ...number(40), block: power } },
});
const when = (condition: BlockState, then: BlockState): BlockState => ({ type: 'mlab_if', inputs: { COND: { block: condition }, DO: { block: then } } });
const sunny: BlockState = { type: 'mlab_weather_is', fields: { OP: 'IS', WEATHER: 'Sun' } };
const userHpUnder50: BlockState = {
  type: 'mlab_compare',
  fields: { OP: 'LT' },
  inputs: { A: { ...number(0), block: { type: 'mlab_hp', fields: { WHO: 'user' } } }, B: number(50) },
};
const tripleKick: BlockState = {
  type: 'mlab_repeat',
  inputs: {
    TIMES: number(3),
    DO: { block: damage({ type: 'mlab_arith', fields: { OP: 'MULTIPLY' }, inputs: { A: number(10), B: { ...number(1), block: { type: 'mlab_repeat_count' } } } }) },
  },
};
const sleepTarget: BlockState = { type: 'mlab_status', fields: { WHO: 'target', STATUS: 'Asleep', CHANCE: 100 } };
const reflect: BlockState = {
  type: 'mlab_effect',
  fields: { TURNS: 5, SCOPE: 'user_side', NAME: 'reflect' },
  inputs: { RULES: { block: { type: 'mlab_rule_damage_taken', fields: { FACTOR: 0.5, CATEGORY: 'Physical' } } } },
};
const haze: BlockState = {
  type: 'mlab_sure',
  inputs: {
    DO: { block: { type: 'mlab_for_each', fields: { GROUP: 'everyone' }, inputs: { DO: { block: { type: 'mlab_reset_stats', fields: { WHO: 'each' } } } } } },
  },
};
const furyCount: BlockState = {
  type: 'mlab_mark',
  fields: { SCOPE: 'user', NAME: 'fury' },
  inputs: {
    VALUE: {
      ...number(1),
      block: { type: 'mlab_arith', fields: { OP: 'ADD' }, inputs: { A: { ...number(0), block: { type: 'mlab_mark_value', fields: { SCOPE: 'user', NAME: 'fury' } } }, B: number(1) } },
    },
  },
};

/** Examples that say more than the bare block from the palette would. */
const EXAMPLES: Record<string, BlockState> = {
  mlab_if: when(sunny, damage(120)),
  mlab_if_else: { type: 'mlab_if_else', inputs: { COND: { block: userHpUnder50 }, DO: { block: damage(80) }, ELSE: { block: damage(40) } } },
  mlab_repeat: tripleKick,
  mlab_repeat_count: tripleKick,
  mlab_compare: userHpUnder50,
  mlab_and_or: { type: 'mlab_and_or', fields: { OP: 'AND' }, inputs: { A: { block: sunny }, B: { block: { type: 'mlab_first_turn' } } } },
  mlab_not: { type: 'mlab_not', inputs: { A: { block: { type: 'mlab_hit_landed' } } } },
  mlab_for_each: {
    type: 'mlab_for_each',
    fields: { GROUP: 'foes' },
    inputs: { DO: { block: { type: 'mlab_stats', fields: { WHO: 'each', STAT: 'Speed', STAGES: '-1' } } } },
  },
  mlab_chance: when({ type: 'mlab_chance', fields: { PERCENT: 30 } }, { type: 'mlab_confuse_target' }),
  mlab_as_type: { type: 'mlab_as_type', inputs: { TYPE: kind('Fire'), DO: { block: damage(80) } } },
  mlab_effect: reflect,
  mlab_rule_damage_taken: reflect,
  mlab_mark_value: furyCount,
  mlab_sure: haze,
  mlab_if_reaches: when({ type: 'mlab_if_reaches', fields: { WHO: 'target' } }, sleepTarget),
  mlab_repeat_works: when({ type: 'mlab_not', inputs: { A: { block: { type: 'mlab_repeat_works' } } } }, { type: 'mlab_fail' }),
};

const NOTES: Record<string, string> = {
  mlab_on_use:
    '“a chosen foe”: the player picks one. “a chosen foe or ally”: the player picks anyone but the user. “the user”: the move acts on the Pokémon using it. “all foes”, “everyone else”, “all allies”, “the user and its allies”, “everyone”: “deal damage” hits each of them; the other blocks still act on one Pokémon, so put them in a “for each Pokémon the move is aimed at” block. Those five need a script. “a random foe”: picked again every time the move is used. “the field”: for moves aimed at no one, like weather. The choice is written as target = … at the top of the file, and left out for “a chosen foe”.',
  mlab_on_turn_start:
    'Focus Punch tightens its focus here, and Beak Blast starts heating up. The stack runs for every Pokémon that chose the move, fastest first, before the first move of the turn.',
  mlab_damage:
    'A plain move checks its accuracy once for the whole move. In a script every “deal damage” block checks it again, so one can hit and the next one miss. A power with a fraction, such as 150 × HP ÷ max HP, loses what comes after the decimal point.',
  mlab_exact_damage:
    'It is still an attack: protection, hiding and a miss stop it, and a Pokémon watching for hits reacts to it. To take HP away without any of that, use “loses”.',
  mlab_ohko:
    'With that rule the move lands about 30% of the time, a little more for each level the user has over the target, and never on a target of a higher level.',
  mlab_as_type:
    'Pick the type in the menu, or plug in a block that works one out, such as “the user’s first type”.',
  mlab_heal:
    'The plain line heals whoever the move is used on, so Move Lab writes it only when the move is used on “the user”. For any other target the move becomes a script.',
  mlab_status:
    'Some types cannot get some conditions: Fire types burns, Ice types freezing, Poison and Steel types poison, Electric types paralysis. Sleep lasts 1 to 3 turns, and deep sleep (the one Rest uses) lasts 2.',
  mlab_protect: 'In a plain move it protects whoever the move is used on, so the move has to be used on “the user”.',
  mlab_weather:
    'Harsh sunlight makes Fire moves 1.5 times as strong and Water moves half as strong; rain does the opposite. A sandstorm takes 1/16 of max HP at the end of each turn from every Pokémon that is not a Rock, Ground or Steel type, and raises the Sp. Def of Rock types. Hail does the same to every Pokémon that is not an Ice type, and raises the Defense of Ice types.',
  mlab_effect:
    'Where the effect goes decides who is under it: one Pokémon, everyone on a side, or everyone in the battle. “for 5 turns” counts this turn, so the effect ends at the end of the fifth. An effect on a Pokémon ends when it faints. The name is yours to choose: other moves find the effect by it, with “end the effect” and “the effect … is active”. An effect with no blocks inside does nothing by itself, but other moves can still ask whether it is active.',
  mlab_mark:
    'Marks are how moves talk to each other. Minimize leaves a mark on its user, and Stomp doubles its power on a target that has it. Fury Cutter counts its uses in a mark, as in the picture next to “mark” below. The name is yours to choose.',
  mlab_rule_drain: 'Leech Seed: the seeded Pokémon is under the effect, and the menu says who gets the HP.',
  mlab_rule_block_status: 'Safeguard, and the sleeplessness of an Uproar. Rest still works, since that is something a Pokémon does to itself.',
  mlab_rule_always_hit: 'Lock-On: give the user this effect for 2 turns, against “the target”.',
  mlab_force_move:
    'Running it again on a later turn stops the battle with an error, so put it inside “if first turn of this move”. With “a random foe”, each later turn is aimed at a foe picked at random; the first turn still goes to whoever the starting block says.',
  mlab_hide:
    'Fly, Dig and Dive: on the first turn hide, show a message, make the move take 2 turns and stop; on the second turn attack. A “deal damage” block reaches a hiding place with the option “reach hidden targets”, and a whole move with “Reaches Pokémon hiding” under More.',
  mlab_if_reaches:
    'In a script, Move Lab already writes this check around every block that does something to another Pokémon, so a status or a stat drop waits for the move to get through. Use the block yourself to put several blocks under one check, or to say what happens when the move does not get through. A “deal damage” block right after the check follows its answer instead of checking again.',
  mlab_sure:
    'Haze, in the picture, resets everyone’s stat changes whatever they are doing. “deal damage” blocks inside cannot miss either.',
  mlab_on_hit:
    'King’s Shield and Beak Blast use it to act on whoever touched the user: in this stack that Pokémon is “the target”.',
  mlab_hit_was: '“made contact” is true for moves that have “Makes contact” ticked under More.',
  mlab_text_is:
    'Items, abilities and species are only names here: the engine keeps them and lets moves read, take, give and suppress them, but does not know what an item or an ability does. Set them in the battle setup to test.',
  mlab_announce: 'It also writes manual_announce = true at the top of the file, which turns the usual announcement off.',
};

function entry(state: BlockState): ReferenceEntry {
  return { type: state.type, state: EXAMPLES[state.type] ?? state, note: NOTES[state.type] };
}

const ABOUT: Record<string, string> = {
  damage: 'Take HP away.',
  heal: 'Give HP back.',
  status: 'Conditions that stick.',
  stats: 'Raise and lower battle stats.',
  field: 'Change the weather.',
  effects: 'Things that last: screens, terrains, seeds, and notes that moves leave for each other.',
  turns: 'Moves that take more than one turn, hide, or wait for their moment.',
  logic: 'Decide and repeat.',
  info: 'Read what is happening. These plug into other blocks.',
  traits: 'Items, abilities, types, weight and money.',
  text: 'Tell the player what happened.',
};

/** Blocks shown a second time with other settings, after the palette's own entry. */
const VARIANTS: Record<string, ReferenceEntry[]> = {
  mlab_stats: [
    {
      type: 'mlab_stats',
      state: { type: 'mlab_stats', fields: { WHO: 'target', STAT: 'Attack', STAGES: '-1' } },
      note: 'On another Pokémon, the script line waits for the move to get through to it.',
      variant: true,
    },
  ],
};

export const REFERENCE: ReferenceSection[] = [
  { id: 'starts', name: 'Starts', color: COLORS.hat, about: 'Every stack begins with one.', entries: HAT_TYPES.map((type) => entry({ type })) },
  ...CATEGORIES.map((category) => ({
    id: category.id,
    name: category.name,
    color: category.color,
    about: ABOUT[category.id] ?? '',
    entries: category.blocks
      .filter((block) => !HAT_TYPES.includes(block.type))
      .flatMap(({ kind: _kind, ...state }) => [entry(state as BlockState), ...(VARIANTS[state.type] ?? [])]),
  })),
];

/** Values for the rows "+ option" adds, where the defaults would make a poor example. */
export const OPTION_FIELDS: Record<string, Record<string, unknown>> = {
  // "another stat" starts on Attack again.
  'mlab_stats/stat': { STAT_1: 'Defense' },
  // "out of sight" is what the block means without a place, and writes nothing more.
  'mlab_two_turn/hidden': { PLACE: 'Underground' },
};
