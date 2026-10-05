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
const damage = (power: number | BlockState): BlockState => ({
  type: 'mlab_damage',
  inputs: { POWER: typeof power === 'number' ? number(power) : { ...number(40), block: power } },
});
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

/** Examples that say more than the bare block from the palette would. */
const EXAMPLES: Record<string, BlockState> = {
  mlab_if: { type: 'mlab_if', inputs: { COND: { block: sunny }, DO: { block: damage(120) } } },
  mlab_if_else: { type: 'mlab_if_else', inputs: { COND: { block: userHpUnder50 }, DO: { block: damage(80) }, ELSE: { block: damage(40) } } },
  mlab_repeat: tripleKick,
  mlab_repeat_count: tripleKick,
  mlab_compare: userHpUnder50,
  mlab_and_or: { type: 'mlab_and_or', fields: { OP: 'AND' }, inputs: { A: { block: sunny }, B: { block: { type: 'mlab_first_turn' } } } },
  mlab_not: { type: 'mlab_not', inputs: { A: { block: { type: 'mlab_hit_landed' } } } },
};

const NOTES: Record<string, string> = {
  mlab_on_use:
    '“a chosen foe”: the player picks one. “the user”: the move acts on the Pokémon using it. “everyone else”: “deal damage” hits every other Pokémon, allies included, which takes a script; the other blocks still act on one foe. “a random foe”: picked again every time the move is used. “the field”: for moves aimed at no one, like weather. The choice is written as target = … at the top of the file, and left out for “a chosen foe”.',
  mlab_damage:
    'A plain move checks its accuracy once for the whole move. In a script every “deal damage” block checks it again, so one can hit and the next one miss.',
  mlab_ohko:
    'With that rule the move lands about 30% of the time, a little more for each level the user has over the target, and never on a target of a higher level.',
  mlab_heal:
    'The plain line heals whoever the move is used on, so Move Lab writes it only when the move is used on “the user”. For any other target the move becomes a script.',
  mlab_status:
    'Some types cannot get some conditions: Fire types burns, Ice types freezing, Poison and Steel types poison, Electric types paralysis. Sleep lasts 1 to 3 turns, and deep sleep (the one Rest uses) lasts 2.',
  mlab_protect: 'It protects whoever the move is used on, so the move has to be used on “the user”.',
  mlab_weather:
    'Harsh sunlight makes Fire moves 1.5 times as strong and Water moves half as strong. A sandstorm takes 1/16 of max HP at the end of each turn from every Pokémon that is not a Rock, Ground or Steel type, and raises the Sp. Def of Rock types.',
  mlab_force_move:
    'Running it again on a later turn stops the battle with an error, so put it inside “if first turn of this move”. With “a random foe”, each later turn is aimed at a foe picked at random; the first turn still goes to whoever the starting block says.',
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
  turns: 'Moves that take more than one turn.',
  logic: 'Decide and repeat.',
  info: 'Read what is happening. These plug into other blocks.',
  text: 'Tell the player what happened.',
};

/** Blocks shown a second time with other settings, after the palette's own entry. */
const VARIANTS: Record<string, ReferenceEntry[]> = {
  mlab_stats: [
    {
      type: 'mlab_stats',
      state: { type: 'mlab_stats', fields: { WHO: 'target', STAT: 'Attack', STAGES: '-1' } },
      note: 'Changing the target’s stats has no script line yet.',
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
};
