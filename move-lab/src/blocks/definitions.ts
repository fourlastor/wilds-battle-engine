// Block definitions. Every block maps to one piece of the engine's move API:
// either a `ctx:` call in a script, or an entry in a move's `effects` list.
import * as Blockly from 'blockly/core';
import { TYPES } from '../model.ts';

type Menu = [string, string][];

export const TARGET_MENU: Menu = [
  ['a chosen foe', 'Selected'],
  ['the user', 'User'],
  ['everyone else', 'AllOthers'],
  ['a random foe', 'RandomOpponent'],
  ['the field', 'Field'],
];
const STAT_MENU: Menu = [
  ['Attack', 'Attack'], ['Defense', 'Defense'], ['Sp. Atk', 'SpAttack'], ['Sp. Def', 'SpDefense'],
  ['Speed', 'Speed'], ['Accuracy', 'Accuracy'], ['Evasion', 'Evasion'],
];
const STAGE_MENU: Menu = [
  ['+1', '1'], ['+2', '2'], ['+3', '3'], ['+4', '4'], ['+5', '5'], ['+6', '6'],
  ['−1', '-1'], ['−2', '-2'], ['−3', '-3'], ['−4', '-4'], ['−5', '-5'], ['−6', '-6'],
];
const TYPE_MENU: Menu = TYPES.map((type) => [type, type]);
const GIVE_STATUS_MENU: Menu = [
  ['poison', 'Poisoned'], ['bad poison', 'BadlyPoisoned'], ['a burn', 'Burned'], ['paralysis', 'Paralyzed'],
  ['sleep', 'Asleep'], ['deep sleep (Rest)', 'RestSleep'], ['freeze', 'Frozen'],
];
const STATUS_IS_MENU: Menu = [
  ['asleep', 'Asleep'], ['poisoned', 'Poisoned'], ['badly poisoned', 'BadlyPoisoned'], ['burned', 'Burned'],
  ['paralyzed', 'Paralyzed'], ['frozen', 'Frozen'], ['healthy', 'None'],
];
const WEATHER_MENU: Menu = [['harsh sunlight', 'Sun'], ['a sandstorm', 'Sandstorm']];
const WEATHER_IS_MENU: Menu = [['sunny', 'Sun'], ['a sandstorm', 'Sandstorm'], ['clear', 'None']];
const WHO_MENU: Menu = [['the user', 'user'], ['the target', 'target']];
const WHOSE_MENU: Menu = [['the user’s', 'user'], ['the target’s', 'target']];
const IS_MENU: Menu = [['is', 'IS'], ['is not', 'NOT']];

// ---------------------------------------------------------------------------
// Optional rows: the "+ option" menu on a block adds a row, the × removes it.

export interface OptionSpec {
  key: string;
  /** Text shown in the "+ option" menu. */
  menu: string;
  /** How many of this row a block may have (default 1). */
  repeat?: number;
  build: (input: Blockly.Input, suffix: string) => void;
}

export interface OptionBlock extends Blockly.Block {
  mlabOptions: string[];
  mlabSpecs: OptionSpec[];
}

const REMOVE_ICON =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">' +
      '<circle cx="8" cy="8" r="8" fill="rgba(0,0,0,0.3)"/>' +
      '<path d="M5.2 5.2l5.6 5.6M10.8 5.2l-5.6 5.6" stroke="#fff" stroke-width="1.8" stroke-linecap="round"/></svg>',
  );

/** `drain` or, for repeatable rows, `stat#2`. */
export function optionKey(id: string): string {
  return id.split('#')[0];
}
export function optionSuffix(id: string): string {
  const part = id.split('#')[1];
  return part ? `_${part}` : '';
}
export function optionIds(block: Blockly.Block, key?: string): string[] {
  const ids = (block as unknown as { mlabOptions?: string[] }).mlabOptions ?? [];
  return key ? ids.filter((id) => optionKey(id) === key) : ids;
}
export function hasOption(block: Blockly.Block, key: string): boolean {
  return optionIds(block, key).length > 0;
}

function mutate(block: Blockly.Block, change: () => void): void {
  const before = JSON.stringify(block.saveExtraState?.() ?? null);
  Blockly.Events.setGroup(true);
  try {
    change();
    const after = JSON.stringify(block.saveExtraState?.() ?? null);
    const Change = Blockly.Events.get(Blockly.Events.BLOCK_CHANGE) as unknown as new (
      ...args: unknown[]
    ) => Blockly.Events.Abstract;
    Blockly.Events.fire(new Change(block, 'mutation', null, before, after));
  } finally {
    Blockly.Events.setGroup(false);
  }
}

function addRow(block: OptionBlock, id: string): void {
  const spec = block.mlabSpecs.find((candidate) => candidate.key === optionKey(id));
  if (!spec) return;
  block.appendEndRowInput(`BR_${id}`);
  const input = block.appendDummyInput(`OPT_${id}`);
  spec.build(input, optionSuffix(id));
  input.appendField(
    new Blockly.FieldImage(REMOVE_ICON, 16, 16, 'Remove', () => {
      if (!block.isInFlyout) mutate(block, () => removeOption(block, id));
    }),
  );
  if (block.getInput('BR_ADD')) {
    block.moveInputBefore(`BR_${id}`, 'BR_ADD');
    block.moveInputBefore(`OPT_${id}`, 'BR_ADD');
  }
}

function removeOption(block: OptionBlock, id: string): void {
  block.removeInput(`OPT_${id}`, true);
  block.removeInput(`BR_${id}`, true);
  block.mlabOptions = block.mlabOptions.filter((other) => other !== id);
}

function addOption(block: OptionBlock, key: string): void {
  const spec = block.mlabSpecs.find((candidate) => candidate.key === key);
  if (!spec) return;
  let id = key;
  if ((spec.repeat ?? 1) > 1) {
    const used = optionIds(block, key).map((other) => Number(other.split('#')[1]));
    id = `${key}#${Math.max(0, ...used) + 1}`;
  } else if (block.mlabOptions.includes(key)) {
    return;
  }
  block.mlabOptions.push(id);
  addRow(block, id);
}

function optionMenu(block: OptionBlock | null): Menu {
  const menu: Menu = [['+ option', '']];
  if (!block) return menu;
  for (const spec of block.mlabSpecs) {
    if (optionIds(block, spec.key).length < (spec.repeat ?? 1)) menu.push([spec.menu, spec.key]);
  }
  return menu;
}

function installOptions(block: OptionBlock, specs: OptionSpec[], ownRow: boolean): void {
  block.mlabOptions = [];
  block.mlabSpecs = specs;
  const dropdown = new Blockly.FieldDropdown(
    function (this: Blockly.FieldDropdown) {
      return optionMenu(this.getSourceBlock() as OptionBlock | null);
    },
    function (this: Blockly.FieldDropdown, value: string) {
      const source = this.getSourceBlock() as OptionBlock | null;
      // Adding the row changes the block, so do it after the menu has closed.
      if (source && value && !source.isInFlyout) setTimeout(() => mutate(source, () => addOption(source, value)), 0);
      return null;
    },
  );
  // Long blocks keep "+ option" on a line of its own so the palette stays narrow.
  if (ownRow) block.appendEndRowInput('BR_ADD');
  block.appendDummyInput('ADD').appendField(dropdown, 'ADD');
  block.saveExtraState = function (this: OptionBlock) {
    return this.mlabOptions.length ? { options: [...this.mlabOptions] } : null;
  };
  block.loadExtraState = function (this: OptionBlock, state: { options?: string[] } | null) {
    for (const id of [...this.mlabOptions]) removeOption(this, id);
    for (const id of state?.options ?? []) {
      if (this.mlabOptions.includes(id)) continue;
      this.mlabOptions.push(id);
      addRow(this, id);
    }
  };
}

const percent = (value: number) => new Blockly.FieldNumber(value, 0, 100, 0.01);

const OPTIONS: Record<string, OptionSpec[]> = {
  mlab_damage: [
    {
      key: 'drain',
      menu: 'heal the user from it',
      build: (input) => input.appendField('and heal the user by').appendField(percent(50), 'DRAIN').appendField('% of it'),
    },
    {
      key: 'min_hp',
      menu: 'leave the target some HP',
      build: (input) =>
        input.appendField('but leave the target at least').appendField(new Blockly.FieldNumber(1, 0, 999, 1), 'MIN_HP').appendField('HP'),
    },
    {
      key: 'accuracy',
      menu: 'give this hit its own accuracy',
      build: (input) => input.appendField('landing').appendField(percent(90), 'ACC').appendField('% of the time on its own'),
    },
    { key: 'typeless', menu: 'ignore type matchups', build: (input) => input.appendField('ignoring type matchups') },
    {
      key: 'recoil',
      menu: 'recoil from the damage dealt',
      build: (input) => input.appendField('and the user takes').appendField(percent(33.33), 'RECOIL').appendField('% of it as recoil'),
    },
    { key: 'high_crit', menu: 'better critical-hit odds', build: (input) => input.appendField('with better critical-hit odds') },
    { key: 'always_crit', menu: 'always a critical hit', build: (input) => input.appendField('always landing a critical hit') },
  ],
  mlab_heal: [{ key: 'quiet', menu: 'no message', build: (input) => input.appendField('without a message') }],
  mlab_status: [
    { key: 'replace', menu: 'replace another status', build: (input) => input.appendField('even over another status') },
  ],
  mlab_two_turn: [
    { key: 'hidden', menu: 'hide while charging', build: (input) => input.appendField('hidden from attacks while charging') },
    { key: 'skip_sun', menu: 'no charge in sunlight', build: (input) => input.appendField('skipping the charge in harsh sunlight') },
  ],
  mlab_consecutive: [
    { key: 'confuse', menu: 'confuse the user after', build: (input) => input.appendField('then the user becomes confused') },
    { key: 'double', menu: 'double the power each turn', build: (input) => input.appendField('doubling the power each turn') },
  ],
  mlab_stats: [
    {
      key: 'stat',
      menu: 'another stat',
      repeat: 6,
      build: (input, suffix) =>
        input
          .appendField('and')
          .appendField(new Blockly.FieldDropdown(STAT_MENU), `STAT${suffix}`)
          .appendField('by')
          .appendField(new Blockly.FieldDropdown(STAGE_MENU), `STAGES${suffix}`),
    },
    {
      key: 'chance',
      menu: 'only some of the time',
      build: (input) => input.appendField('only').appendField(percent(10), 'CHANCE').appendField('% of the time'),
    },
  ],
};

// ---------------------------------------------------------------------------

const statement = { previousStatement: null, nextStatement: null, inputsInline: true };
const lastStatement = { previousStatement: null, inputsInline: true };
const dropdown = (name: string, options: Menu) => ({ type: 'field_dropdown', name, options });
const number = (name: string, value: number, min: number, max: number, precision = 1) => ({
  type: 'field_number', name, value, min, max, precision,
});
const pct = (name: string, value: number) => number(name, value, 0, 100, 0.01);
const endRow = { type: 'input_end_row' };

const WITH_OPTIONS: Record<string, object> = {
  mlab_damage: {
    message0: 'deal damage with power %1',
    args0: [{ type: 'input_value', name: 'POWER', check: 'Number' }],
    ...statement,
    style: 'damage_blocks',
    tooltip: 'A normal attack. The damage depends on power, stats and types. “+ option” adds extras such as draining HP.',
  },
  mlab_heal: {
    message0: 'heal the user by %1 % of its max HP',
    args0: [pct('PERCENT', 50)],
    ...statement,
    style: 'heal_blocks',
    tooltip: 'Restores part of the user’s HP. Fails if the user is already at full HP.',
  },
  mlab_status: {
    message0: 'give the target %1 %2 % of the time',
    args0: [dropdown('STATUS', GIVE_STATUS_MENU), pct('CHANCE', 100)],
    ...statement,
    style: 'status_blocks',
    tooltip: 'Inflicts a lasting condition on whoever the move is used on.',
  },
  mlab_two_turn: {
    message0: 'charge for a turn saying %1 %2 then hit with power %3',
    args0: [{ type: 'field_input', name: 'MESSAGE', text: '{user} is getting ready!' }, endRow, number('POWER', 80, 1, 999)],
    ...statement,
    style: 'turn_blocks',
    tooltip: 'The engine’s built-in two-turn attack. Type {user} for the user’s name.',
  },
  mlab_consecutive: {
    message0: 'keep attacking for %1 to %2 turns %3 with power %4',
    args0: [number('MIN', 2, 1, 8), number('MAX', 3, 1, 8), endRow, number('POWER', 120, 1, 999)],
    ...statement,
    style: 'turn_blocks',
    tooltip: 'The engine’s built-in rampage: the user repeats the move for a few turns.',
  },
  mlab_stats: {
    message0: 'change %1 %2 by %3',
    args0: [dropdown('WHO', WHOSE_MENU), dropdown('STAT', STAT_MENU), dropdown('STAGES', STAGE_MENU)],
    ...statement,
    style: 'stats_blocks',
    tooltip: 'Raises or lowers battle stats by stages. Each stage is a step, not a number of points.',
  },
};

const PLAIN: object[] = [
  // Starts
  {
    type: 'mlab_on_use',
    message0: 'when this move is used on %1',
    args0: [dropdown('TARGET', TARGET_MENU)],
    nextStatement: null,
    style: 'hat_blocks',
    tooltip: 'Everything under this block happens when the move is used.',
  },
  {
    type: 'mlab_on_hit',
    message0: 'when the user is hit while watching',
    nextStatement: null,
    style: 'hat_blocks',
    tooltip: 'Runs each time an attack hits the user, after “watch for hits”. Only messages and changes to the user’s stats work here.',
  },
  {
    type: 'mlab_on_interrupt',
    message0: 'when a multi-turn move is cut short',
    nextStatement: null,
    style: 'hat_blocks',
    tooltip: 'Runs if sleep, paralysis, a flinch or confusion stops one of the move’s later turns.',
  },

  // Damage
  { type: 'mlab_fixed_damage', message0: 'deal exactly %1 HP of damage', args0: [number('AMOUNT', 40, 1, 9999)], ...statement, style: 'damage_blocks', tooltip: 'Always the same amount, whatever the stats and types.' },
  { type: 'mlab_level_damage', message0: 'deal damage equal to the user’s level', ...statement, style: 'damage_blocks' },
  { type: 'mlab_ohko', message0: 'knock out in one hit', ...statement, style: 'damage_blocks', tooltip: 'Pair it with the “one-hit KO” accuracy rule in the move details.' },
  {
    type: 'mlab_multi_hit',
    message0: 'hit %1 to %2 times with power %3',
    args0: [number('MIN', 2, 1, 10), number('MAX', 5, 1, 10), number('POWER', 25, 1, 999)],
    ...statement,
    style: 'damage_blocks',
  },
  { type: 'mlab_recoil_max_hp', message0: 'the user takes %1 % of its max HP as recoil', args0: [pct('PERCENT', 25)], ...statement, style: 'damage_blocks' },
  { type: 'mlab_faint_user', message0: 'the user faints', ...statement, style: 'damage_blocks' },

  // Status
  { type: 'mlab_confuse_target', message0: 'confuse the target', ...statement, style: 'status_blocks' },
  { type: 'mlab_confuse_self', message0: 'confuse the user', ...statement, style: 'status_blocks' },
  { type: 'mlab_flinch', message0: 'make the target flinch %1 % of the time', args0: [pct('CHANCE', 30)], ...statement, style: 'status_blocks' },
  { type: 'mlab_bind', message0: 'trap the target for a few turns', ...statement, style: 'status_blocks' },
  { type: 'mlab_protect', message0: 'protect the user this turn', ...statement, style: 'status_blocks' },

  // Stats
  {
    type: 'mlab_boost_next',
    message0: 'boost the user’s next %1 move × %2',
    args0: [dropdown('TYPE', TYPE_MENU), number('MULT', 2, 0.1, 8, 0.1)],
    ...statement,
    style: 'stats_blocks',
    tooltip: 'The boost is used up by the user’s next move, whatever its type.',
  },

  // Field
  { type: 'mlab_weather', message0: 'start %1 for %2 turns', args0: [dropdown('WEATHER', WEATHER_MENU), number('TURNS', 5, 1, 99)], ...statement, style: 'field_blocks' },

  // Turns
  {
    type: 'mlab_force_move',
    message0: 'make this move take %1 turns, %2 aimed at %3',
    args0: [
      { type: 'input_value', name: 'TURNS', check: 'Number' },
      endRow,
      dropdown('POLICY', [['the same target', 'SameTarget'], ['a random foe', 'RandomOpponent']]),
    ],
    ...statement,
    style: 'turn_blocks',
    tooltip: 'From 2 to 8 turns in total, counting this one. The user cannot pick another move until it is over.',
  },
  { type: 'mlab_break_sequence', message0: 'end the multi-turn move now', ...statement, style: 'turn_blocks' },
  { type: 'mlab_recharge', message0: 'the user must recharge next turn', ...statement, style: 'turn_blocks' },
  { type: 'mlab_watch_hits', message0: 'watch for hits until the user’s next action', ...statement, style: 'turn_blocks', tooltip: 'Needs a “when the user is hit while watching” stack to say what happens.' },
  { type: 'mlab_fail', message0: 'the move fails', ...lastStatement, style: 'turn_blocks', tooltip: 'Shows “But it failed!” and stops.' },
  { type: 'mlab_stop', message0: 'stop here', ...lastStatement, style: 'turn_blocks', tooltip: 'Nothing below this block runs this turn.' },

  // Logic
  {
    type: 'mlab_if',
    message0: 'if %1 then',
    args0: [{ type: 'input_value', name: 'COND', check: 'Boolean' }],
    message1: '%1',
    args1: [{ type: 'input_statement', name: 'DO' }],
    ...statement,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_if_else',
    message0: 'if %1 then',
    args0: [{ type: 'input_value', name: 'COND', check: 'Boolean' }],
    message1: '%1',
    args1: [{ type: 'input_statement', name: 'DO' }],
    message2: 'else',
    message3: '%1',
    args3: [{ type: 'input_statement', name: 'ELSE' }],
    ...statement,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_repeat',
    message0: 'repeat %1 times',
    args0: [{ type: 'input_value', name: 'TIMES', check: 'Number' }],
    message1: '%1',
    args1: [{ type: 'input_statement', name: 'DO' }],
    ...statement,
    style: 'logic_blocks',
  },
  { type: 'mlab_repeat_count', message0: 'repeat number', output: 'Number', style: 'logic_blocks', tooltip: '1 the first time round, 2 the second, and so on. Use it inside “repeat”.' },
  {
    type: 'mlab_compare',
    message0: '%1 %2 %3',
    args0: [
      { type: 'input_value', name: 'A', check: 'Number' },
      dropdown('OP', [['=', 'EQ'], ['≠', 'NEQ'], ['<', 'LT'], ['≤', 'LTE'], ['>', 'GT'], ['≥', 'GTE']]),
      { type: 'input_value', name: 'B', check: 'Number' },
    ],
    output: 'Boolean',
    inputsInline: true,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_and_or',
    message0: '%1 %2 %3',
    args0: [
      { type: 'input_value', name: 'A', check: 'Boolean' },
      dropdown('OP', [['and', 'AND'], ['or', 'OR']]),
      { type: 'input_value', name: 'B', check: 'Boolean' },
    ],
    output: 'Boolean',
    inputsInline: true,
    style: 'logic_blocks',
  },
  { type: 'mlab_not', message0: 'not %1', args0: [{ type: 'input_value', name: 'A', check: 'Boolean' }], output: 'Boolean', inputsInline: true, style: 'logic_blocks' },
  {
    type: 'mlab_arith',
    message0: '%1 %2 %3',
    args0: [
      { type: 'input_value', name: 'A', check: 'Number' },
      dropdown('OP', [['+', 'ADD'], ['−', 'MINUS'], ['×', 'MULTIPLY'], ['÷', 'DIVIDE']]),
      { type: 'input_value', name: 'B', check: 'Number' },
    ],
    output: 'Number',
    inputsInline: true,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_random',
    message0: 'random number from %1 to %2',
    args0: [
      { type: 'input_value', name: 'FROM', check: 'Number' },
      { type: 'input_value', name: 'TO', check: 'Number' },
    ],
    output: 'Number',
    inputsInline: true,
    style: 'logic_blocks',
    tooltip: 'Whole numbers from 0 to 255, both ends included.',
  },
  { type: 'mlab_number', message0: '%1', args0: [{ type: 'field_number', name: 'NUM', value: 0 }], output: 'Number', style: 'logic_blocks' },

  // Battle info
  { type: 'mlab_hp', message0: '%1 HP', args0: [dropdown('WHO', WHOSE_MENU)], output: 'Number', style: 'info_blocks', classes: 'mlab-info' },
  { type: 'mlab_turn', message0: 'turn of this move', output: 'Number', style: 'info_blocks', classes: 'mlab-info', tooltip: '1 on the turn the move is chosen, 2 on the next, and so on.' },
  { type: 'mlab_last_damage', message0: 'damage of the last hit', output: 'Number', style: 'info_blocks', classes: 'mlab-info' },
  { type: 'mlab_status_is', message0: '%1 %2 %3', args0: [dropdown('WHO', WHO_MENU), dropdown('OP', IS_MENU), dropdown('STATUS', STATUS_IS_MENU)], output: 'Boolean', style: 'info_blocks', classes: 'mlab-info' },
  { type: 'mlab_weather_is', message0: 'weather %1 %2', args0: [dropdown('OP', IS_MENU), dropdown('WEATHER', WEATHER_IS_MENU)], output: 'Boolean', style: 'info_blocks', classes: 'mlab-info' },
  { type: 'mlab_first_turn', message0: 'first turn of this move', output: 'Boolean', style: 'info_blocks', classes: 'mlab-info' },
  { type: 'mlab_last_turn', message0: 'last turn of this move', output: 'Boolean', style: 'info_blocks', classes: 'mlab-info', tooltip: 'True on the final turn of a multi-turn move.' },
  { type: 'mlab_in_sequence', message0: 'already a multi-turn move', output: 'Boolean', style: 'info_blocks', classes: 'mlab-info', tooltip: 'True once “make this move take … turns” has run for this use of the move.' },
  { type: 'mlab_hit_landed', message0: 'the last hit landed', output: 'Boolean', style: 'info_blocks', classes: 'mlab-info' },

  // Text
  { type: 'mlab_message', message0: 'show message %1', args0: [{ type: 'field_input', name: 'TEXT', text: '{user} is ready!' }], ...statement, style: 'text_blocks', tooltip: 'Type {user} for the user’s name.' },
  { type: 'mlab_announce', message0: 'announce the move', ...statement, style: 'text_blocks', tooltip: 'Shows “… used …!” here instead of at the start of the move.' },
  { type: 'mlab_splash', message0: 'nothing happens', ...statement, style: 'text_blocks', tooltip: 'The engine’s Splash effect.' },
];

let defined = false;

export function defineBlocks(): void {
  if (defined) return;
  defined = true;
  Blockly.defineBlocksWithJsonArray(PLAIN);
  for (const [type, json] of Object.entries(WITH_OPTIONS)) {
    Blockly.Blocks[type] = {
      init(this: OptionBlock) {
        this.jsonInit(json);
        installOptions(this, OPTIONS[type], type !== 'mlab_damage');
      },
    };
  }
}

export const HAT_TYPES = ['mlab_on_use', 'mlab_on_hit', 'mlab_on_interrupt'];
