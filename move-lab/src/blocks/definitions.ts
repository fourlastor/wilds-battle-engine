// Block definitions. Every block maps to one piece of the engine's move API:
// either a `ctx:` call in a script, or an entry in a move's `effects` list.
import * as Blockly from 'blockly/core';
import { TYPES } from '../model.ts';

type Menu = [string, string][];

export const TARGET_MENU: Menu = [
  ['a chosen foe', 'Selected'],
  ['the user', 'User'],
  ['an ally', 'Ally'],
  ['the user or an ally', 'UserOrAlly'],
  ['a chosen foe or ally', 'AnyOther'],
  ['a random foe', 'RandomOpponent'],
  ['all foes', 'AllOpponents'],
  ['everyone else', 'AllOthers'],
  ['all allies', 'AllAllies'],
  ['the user and its allies', 'UserAndAllies'],
  ['everyone', 'All'],
  ['the field', 'Field'],
];
const BATTLE_STAT_MENU: Menu = [
  ['Attack', 'Attack'], ['Defense', 'Defense'], ['Sp. Atk', 'SpAttack'], ['Sp. Def', 'SpDefense'], ['Speed', 'Speed'],
];
const STAT_MENU: Menu = [...BATTLE_STAT_MENU, ['Accuracy', 'Accuracy'], ['Evasion', 'Evasion']];
const STAGE_MENU: Menu = [
  ['+1', '1'], ['+2', '2'], ['+3', '3'], ['+4', '4'], ['+5', '5'], ['+6', '6'],
  ['−1', '-1'], ['−2', '-2'], ['−3', '-3'], ['−4', '-4'], ['−5', '-5'], ['−6', '-6'],
];
const TYPE_MENU: Menu = TYPES.map((type) => [type, type]);
const GIVE_STATUS_MENU: Menu = [
  ['poison', 'Poisoned'], ['bad poison', 'BadlyPoisoned'], ['a burn', 'Burned'], ['paralysis', 'Paralyzed'],
  ['sleep', 'Asleep'], ['deep sleep (Rest)', 'RestSleep'], ['freeze', 'Frozen'],
];
const CURE_STATUS_MENU: Menu = [
  ['any status', 'Any'], ['poison', 'Poisoned'], ['bad poison', 'BadlyPoisoned'], ['its burn', 'Burned'],
  ['paralysis', 'Paralyzed'], ['sleep', 'Asleep'], ['being frozen', 'Frozen'],
];
const STATUS_IS_MENU: Menu = [
  ['asleep', 'Asleep'], ['poisoned', 'Poisoned'], ['badly poisoned', 'BadlyPoisoned'], ['burned', 'Burned'],
  ['paralyzed', 'Paralyzed'], ['frozen', 'Frozen'], ['healthy', 'None'], ['suffering from any status', 'Any'],
];
const WEATHER_MENU: Menu = [['harsh sunlight', 'Sun'], ['a sandstorm', 'Sandstorm'], ['rain', 'Rain'], ['hail', 'Hail']];
const WEATHER_IS_MENU: Menu = [
  ['sunny', 'Sun'], ['a sandstorm', 'Sandstorm'], ['rainy', 'Rain'], ['hail', 'Hail'], ['clear', 'None'],
];
const WHO_MENU: Menu = [['the user', 'user'], ['the target', 'target'], ['that Pokémon', 'each']];
const WHOSE_MENU: Menu = [['the user’s', 'user'], ['the target’s', 'target'], ['that Pokémon’s', 'each']];
const SCOPE_MENU: Menu = [
  ...WHO_MENU, ['the user’s side', 'user_side'], ['the target’s side', 'target_side'], ['the field', 'field'],
];
const SCOPE_OF_MENU: Menu = [
  ...WHOSE_MENU, ['the user’s side’s', 'user_side'], ['the target’s side’s', 'target_side'], ['the field’s', 'field'],
];
const IS_MENU: Menu = [['is', 'IS'], ['is not', 'NOT']];
const HIDDEN_MENU: Menu = [
  ['in the air', 'Air'], ['underground', 'Underground'], ['underwater', 'Underwater'], ['out of sight', 'Vanished'],
];
const REACH_MENU: Menu = [...HIDDEN_MENU, ['wherever they hide', 'All']];
// "out of sight" comes first: it is what moves saved before the menu existed mean.
const CHARGE_PLACE_MENU: Menu = [HIDDEN_MENU[3], ...HIDDEN_MENU.slice(0, 3)];
const MOVE_KIND_MENU: Menu = [['any', 'Any'], ['physical', 'Physical'], ['special', 'Special']];
const UNIT_MENU: Menu = [['% of its max HP', 'PERCENT'], ['HP', 'HP']];
const GROUP_MENU: Menu = [
  ['foe', 'foes'], ['ally of the user', 'allies'], ['other Pokémon', 'others'], ['Pokémon in the battle', 'everyone'],
  ['Pokémon the move is aimed at', 'targets'], ['other Pokémon on the target’s side', 'beside_target'],
];
const COUNT_MENU: Menu = [
  ['foes', 'foes'], ['allies of the user', 'allies'], ['other Pokémon', 'others'], ['Pokémon in the battle', 'everyone'],
];

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
const factor = (value: number) => new Blockly.FieldNumber(value, 0, 16, 0.01);
const menu = (options: Menu) => new Blockly.FieldDropdown(options);
const text = (value: string) => new Blockly.FieldTextInput(value);

const MESSAGE_OPTION: OptionSpec = {
  key: 'message',
  menu: 'show a message each turn',
  build: (input) => input.appendField('showing').appendField(text('{name} is hurt!'), 'MESSAGE'),
};
const MOVE_TYPE_OPTION: OptionSpec = {
  key: 'type',
  menu: 'only for one type of move',
  build: (input) => input.appendField('of the').appendField(menu(TYPE_MENU), 'TYPE').appendField('type'),
};

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
    { key: 'never_miss', menu: 'never miss', build: (input) => input.appendField('never missing') },
    { key: 'through', menu: 'go through protection', build: (input) => input.appendField('going through protection') },
    {
      key: 'reach',
      menu: 'reach hidden targets',
      build: (input) => input.appendField('reaching targets').appendField(menu(REACH_MENU), 'REACH'),
    },
    {
      key: 'category',
      menu: 'count as physical or special',
      build: (input) =>
        input.appendField('as a').appendField(menu([['physical', 'Physical'], ['special', 'Special']]), 'CATEGORY').appendField('hit'),
    },
    {
      key: 'attack',
      menu: 'choose the attacking stat',
      build: (input) =>
        input.appendField('attacking with').appendField(menu(WHOSE_MENU.slice(0, 2)), 'ATK_WHO').appendField(menu(BATTLE_STAT_MENU), 'ATK_STAT'),
    },
    {
      key: 'defense',
      menu: 'choose the defending stat',
      build: (input) => input.appendField('against the target’s').appendField(menu(BATTLE_STAT_MENU.slice(1)), 'DEF_STAT'),
    },
    {
      key: 'ignore_stages',
      menu: 'ignore the target’s stat changes',
      build: (input) => input.appendField('ignoring the target’s stat changes'),
    },
    {
      key: 'super',
      menu: 'be super effective against a type',
      repeat: 3,
      build: (input, suffix) =>
        input.appendField('super effective against').appendField(menu(TYPE_MENU), `SUPER${suffix}`).appendField('types'),
    },
    {
      key: 'also_type',
      menu: 'count as a second type too',
      build: (input) => input.appendField('also counting as a').appendField(menu(TYPE_MENU), 'ALSO_TYPE').appendField('move'),
    },
    {
      key: 'only',
      menu: 'hit one Pokémon only',
      build: (input) => input.appendField('only on').appendField(menu(WHO_MENU), 'ONLY'),
    },
  ],
  mlab_exact_damage: [
    { key: 'typeless', menu: 'ignore immunities', build: (input) => input.appendField('even on a target immune to the move’s type') },
    {
      key: 'min_hp',
      menu: 'leave the target some HP',
      build: (input) =>
        input.appendField('but leave the target at least').appendField(new Blockly.FieldNumber(1, 0, 999, 1), 'MIN_HP').appendField('HP'),
    },
    { key: 'only', menu: 'aim at another Pokémon', build: (input) => input.appendField('on').appendField(menu(WHO_MENU), 'ONLY') },
  ],
  mlab_heal: [{ key: 'quiet', menu: 'no message', build: (input) => input.appendField('without a message') }],
  mlab_status: [
    { key: 'replace', menu: 'replace another status', build: (input) => input.appendField('even over another status') },
    { key: 'announce', menu: 'say when it does not work', build: (input) => input.appendField('saying so when it does not work') },
  ],
  mlab_two_turn: [
    {
      key: 'hidden',
      menu: 'hide while charging',
      build: (input) => input.appendField('hidden').appendField(menu(CHARGE_PLACE_MENU), 'PLACE').appendField('while charging'),
    },
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
  mlab_effect: [
    { key: 'forever', menu: 'last until it is removed', build: (input) => input.appendField('lasting until it is removed') },
    {
      key: 'group',
      menu: 'replace others of a group',
      build: (input) => input.appendField('replacing other effects of the group').appendField(text('terrain'), 'GROUP'),
    },
    {
      key: 'end_message',
      menu: 'show a message when it ends',
      build: (input) => input.appendField('showing').appendField(text('The effect wore off!'), 'END_MESSAGE').appendField('when it ends'),
    },
    { key: 'restart', menu: 'start over if already there', build: (input) => input.appendField('starting over if it is already there') },
  ],
  mlab_mark: [
    {
      key: 'turns',
      menu: 'forget it after some turns',
      build: (input) => input.appendField('for').appendField(new Blockly.FieldNumber(2, 1, 99, 1), 'TURNS').appendField('turns, counting this one'),
    },
  ],
  mlab_rule_damage: [
    {
      key: 'except',
      menu: 'spare a type',
      repeat: 3,
      build: (input, suffix) => input.appendField('except').appendField(menu(TYPE_MENU), `EXCEPT${suffix}`).appendField('types'),
    },
    MESSAGE_OPTION,
  ],
  mlab_rule_heal: [MESSAGE_OPTION],
  mlab_rule_drain: [MESSAGE_OPTION],
  mlab_rule_damage_taken: [
    MOVE_TYPE_OPTION,
    { key: 'not_on_crit', menu: 'not on critical hits', build: (input) => input.appendField('except from critical hits') },
  ],
  mlab_rule_damage_dealt: [MOVE_TYPE_OPTION],
  mlab_rule_block_status: [{ key: 'confusion', menu: 'block confusion too', build: (input) => input.appendField('nor be confused') }],
};

// ---------------------------------------------------------------------------
// What each block does. Shown when hovering a block and in the block reference, so every sentence
// here has to match what the engine does (crates/wilds-battle-engine/src).

const TIPS: Record<string, string> = {
  // Starts
  mlab_on_use:
    'Everything under this block happens when the move is used. The menu picks who it is used on. Blocks that say “the target” mean that Pokémon.',
  mlab_on_hit:
    'Runs each time a move damages the user without knocking it out, from “watch for hits” until the user’s next action. In this stack “the target” is the attacker. Blocks that attack or take several turns do not work here.',
  mlab_on_interrupt:
    'Runs if sleep, paralysis, freezing, a flinch or confusion stops a later turn of a multi-turn move. The multi-turn move ends there.',
  mlab_on_turn_start:
    'Runs before anyone acts, on each turn this move was chosen. Use it to get ready: show a message, watch for hits. Blocks that attack or take several turns do not work here.',

  // Damage
  mlab_damage:
    'A normal attack on the target. The damage comes from this power, the stats of both Pokémon and the type matchup. “+ option” adds extras such as draining HP.',
  mlab_fixed_damage: 'Takes away exactly this much HP, whatever the stats. It does nothing to a target that is immune to the move’s type.',
  mlab_exact_damage: 'Takes away exactly this much HP, whatever the stats. It does nothing to a target that is immune to the move’s type.',
  mlab_level_damage: 'Takes away as many HP as the user’s level. It does nothing to a target that is immune to the move’s type.',
  mlab_ohko:
    'Takes away the target’s whole max HP. It is meant to go with the “One-hit KO rule” for accuracy, under More in the move details.',
  mlab_multi_hit: 'Hits a random number of times between the two numbers, each time with this power, then shows “Hit … times!”.',
  mlab_recoil_max_hp: 'The user loses this share of its own max HP, whether or not the move did any damage.',
  mlab_faint_user: 'The user’s HP drops to 0. The blocks below it still run.',
  mlab_hurt: 'Takes HP away directly: it is not an attack, so nothing can block it and nobody reacts to it. No message is shown.',
  mlab_as_type:
    'Every “deal damage” block inside counts as this type instead of the move’s own: for the type matchup, the user’s own-type bonus and the weather.',

  // Healing
  mlab_heal: 'Gives the user back this share of its max HP. It has no effect when the user is already at full HP.',
  mlab_restore: 'Gives HP back to a Pokémon, up to its max HP. No message is shown.',

  // Status
  mlab_status: 'Gives a Pokémon a lasting condition. It does not replace one that Pokémon already has, unless you add that option.',
  mlab_confuse_target:
    'Confuses the target for 1 to 4 turns. A confused Pokémon hurts itself instead of moving about a third of the time.',
  mlab_confuse_self: 'Confuses the user for 1 to 4 turns. It does nothing if the user is already confused.',
  mlab_flinch: 'The target loses its turn if it has not moved yet this turn, so this only matters when the user moves first.',
  mlab_bind: 'Traps the target for 4 or 5 turns. It loses 1/8 of its max HP at the end of each of them.',
  mlab_protect:
    'Moves used on the user fail for the rest of this turn. Give the move a high priority, under More, so that it goes first. Used turn after turn, it works 1 time in 3, then 1 in 9, and so on.',
  mlab_cure: 'Removes a lasting condition from a Pokémon. Nothing happens if it does not have that condition.',
  mlab_free: 'Ends what a trapping move such as Bind is doing to the user.',
  mlab_break_protect: 'Ends the target’s protection for the rest of the turn, so the blocks below can reach it.',

  // Stats
  mlab_stats: 'Raises or lowers a battle stat by stages, between −6 and +6 in total. Each stage is a step, not a number of points.',
  mlab_boost_next:
    'Multiplies the power of the user’s next move if that is a damaging move of this type. The boost is used up by the next move either way.',
  mlab_reset_stats: 'Puts every stat stage of that Pokémon back to 0.',
  mlab_steal_boosts: 'Each stat the target has raised goes back down, and the same stat of the user goes up by as much.',
  mlab_stage: 'A number from −6 to +6. 0 means the stat has not been changed.',
  mlab_raised_stages: 'Adds up the stages by which Attack, Defense, Sp. Atk, Sp. Def and Speed have been raised. Lowered stats count as 0.',

  // Field
  mlab_weather: 'Changes the weather for this many turns after this one. It fails if that weather is already there.',
  mlab_clear_weather: 'Ends the weather at once. Nothing happens if the sky is already clear.',

  // Effects
  mlab_effect:
    'Starts an effect that lasts for this many turns, counting this one. The blocks inside say what it does. Nothing happens if an effect with the same name is already there.',
  mlab_end_effect: 'Ends an effect before its time, the way Brick Break ends a screen. Nothing happens if it is not there.',
  mlab_end_group: 'Ends every effect that was started as part of this group, whatever each one is called.',
  mlab_effect_active: 'True while an effect with this name is running there.',
  mlab_mark:
    'Leaves a number under a name for other moves, or later turns, to read. A mark on a Pokémon is gone when it faints. A mark has no effect of its own.',
  mlab_unmark: 'Removes a mark. Nothing happens if it is not there.',
  mlab_mark_value: 'The number left under this name, or 0 if there is no such mark.',
  mlab_has_mark: 'True if a mark with this name is there.',
  mlab_rule_damage: 'At the end of each turn, each Pokémon under the effect loses this share of its max HP.',
  mlab_rule_heal: 'At the end of each turn, each Pokémon under the effect gets back this share of its max HP.',
  mlab_rule_drain: 'At the end of each turn, each Pokémon under the effect loses this share of its max HP, and the other Pokémon gets it.',
  mlab_rule_damage_taken: 'Multiplies the damage that moves do to the Pokémon under the effect. 0.5 halves it, as Reflect does for physical moves.',
  mlab_rule_damage_dealt: 'Multiplies the damage that the moves of the Pokémon under the effect do.',
  mlab_rule_stat: 'Multiplies one stat of the Pokémon under the effect, on top of its stat stages.',
  mlab_rule_block_status: 'Other Pokémon cannot give this condition to those under the effect. What a Pokémon does to itself still works.',
  mlab_rule_block_drops: 'Other Pokémon cannot lower the stats of those under the effect.',
  mlab_rule_effect_chance: 'Multiplies every “% of the time” and “% chance” in the moves of the Pokémon under the effect.',
  mlab_rule_move_type: 'Moves of the first type are used as moves of the second type by the Pokémon under the effect.',
  mlab_rule_grounded: 'Ground moves hit the Pokémon under the effect even if it is a Flying type.',
  mlab_rule_trapped: 'The Pokémon under the effect counts as trapped. The engine has no switching, so this is only something to read.',
  mlab_rule_endure: 'A move that would knock out the Pokémon under the effect leaves it at 1 HP instead.',
  mlab_rule_always_hit: 'The moves of the Pokémon under the effect cannot miss, and reach targets that are hiding.',
  mlab_rule_without_type: 'The Pokémon under the effect counts as not having that type.',

  // Turns
  mlab_force_move:
    'Makes this a multi-turn move: the user repeats it on the following turns without choosing, until this many turns have passed, counting this one. From 2 to 8 turns. Run it on the first turn only.',
  mlab_break_sequence: 'Ends the multi-turn move with this turn. Next turn the user chooses a move again.',
  mlab_recharge: 'The user cannot act on the next turn. “… must recharge!” is shown instead.',
  mlab_watch_hits:
    'From now until the user’s next action, each move that damages the user runs the “when the user is hit while watching” stack. The move needs that stack.',
  mlab_fail: 'Shows “But it failed!” and stops. Nothing below this block runs.',
  mlab_stop: 'Ends the stack for this turn. Nothing below this block runs.',
  mlab_two_turn:
    'The engine’s built-in two-turn attack: the first turn only shows the message, the second turn hits. Type {user} for the user’s name. A hiding user can only be hit by moves that reach that place.',
  mlab_consecutive:
    'The engine’s built-in rampage: the user repeats the move for a random number of turns between the two numbers and cannot choose another.',
  mlab_hide: 'Moves used on the user miss until it next acts, except moves that say they reach that place.',
  mlab_unhide: 'Brings a hidden Pokémon back at once and calls off the move it was in the middle of.',
  mlab_hurry: 'Everyone else who chose this same move this turn acts right after the user, in their usual order.',
  mlab_sure:
    'The blocks inside do not check whether the move gets through: no protection, hiding place or miss stops them. Use it for effects that always work, like Haze or Heal Bell.',
  mlab_if_reaches:
    'True if the move gets through to that Pokémon. After a “deal damage” block it tells whether that hit landed. Otherwise it makes the usual checks now, once: protection, hiding, then accuracy.',

  // Logic
  mlab_if: 'Runs the blocks inside only when the condition is true.',
  mlab_if_else: 'Runs the first group of blocks when the condition is true, and the second group when it is not.',
  mlab_repeat: 'Runs the blocks inside this many times. A stack can do at most 256 things in one turn, or the engine stops the battle.',
  mlab_repeat_count: '1 the first time round, 2 the second, and so on. Use it inside “repeat”.',
  mlab_for_each:
    'Runs the blocks inside once for each of those Pokémon that is still standing. Inside, “that Pokémon” is the one whose turn it is.',
  mlab_compare: 'True when the comparison between the two numbers holds.',
  mlab_and_or: '“and” is true when both sides are true. “or” is true when at least one side is.',
  mlab_not: 'True when the condition is false, and false when it is true.',
  mlab_arith:
    'Adds, subtracts, multiplies or divides two numbers, or raises the first to the power of the second. Where the game needs a whole number, such as a power or an amount of HP, what comes after the decimal point is dropped.',
  mlab_min_max: 'The smaller or the larger of two numbers. Use it to cap a power.',
  mlab_round: 'Drops what comes after the decimal point: 12.8 becomes 12.',
  mlab_random: 'A whole number picked at random, both ends included. The numbers can go from 0 to 255.',
  mlab_chance: 'True that often, picked at random each time.',
  mlab_number: 'A number. Drop another block on it to work the number out instead.',

  // Battle info
  mlab_hp: 'A number about that Pokémon as it is right now. Attack and the other stats include stat changes. “this turn” means since the turn began.',
  mlab_turn: '1 on the turn the move is chosen, 2 on the next, and so on.',
  mlab_last_damage: 'How much HP the last “deal damage” block above took away. 0 if it missed, or if none has run yet.',
  mlab_status_is: 'Checks the lasting condition that Pokémon has right now. “healthy” means none.',
  mlab_weather_is: 'Checks the weather right now. “clear” means none.',
  mlab_first_turn: 'True on the turn the move is chosen, false on the later turns of a multi-turn move.',
  mlab_last_turn: 'True on the final turn of a multi-turn move.',
  mlab_in_sequence: 'True on the later turns of a multi-turn move, false on its first turn.',
  mlab_hit_landed: 'About the last “deal damage” block above. False if none has run yet.',
  mlab_type_is: 'Checks the types that Pokémon counts as having right now.',
  mlab_state_is: 'Checks one thing about that Pokémon right now. “this turn” means since the turn began.',
  mlab_pair_is: 'Compares the user and the target. A genderless Pokémon is never of the opposite gender.',
  mlab_hit_by_target: 'True if the last move to damage the user this turn was used by that Pokémon.',
  mlab_text_is: 'Compares the name the game gave for that item, ability or species, such as oran_berry. Empty if there is none.',
  mlab_used_all: 'True once the user has used each of its other moves at least once in this battle. False if this is its only move.',
  mlab_last_failed: 'True if the move the user used before this one failed, missed, was blocked or had no effect.',
  mlab_ally_fainted: 'True if a Pokémon on the user’s side fainted during the previous turn.',
  mlab_first_turn_out: 'True during the user’s first turn in the battle.',
  mlab_env_is: 'Compares the name the game gave for the place of the battle, such as cave. Set it in the battle setup to test.',
  mlab_hit_was:
    'About the last hit the user took this turn. In a “when the user is hit while watching” stack, that is the hit that set it off. False if nothing has hit the user this turn.',
  mlab_count: 'How many of those Pokémon are still standing.',
  mlab_streak: 'How many turns in a row the user has used this move without it failing, not counting this turn.',
  mlab_repeat_works:
    'True the first time. After that, true 1 time in 3 for each turn in a row the move has already worked: 1 in 3, then 1 in 9, and so on. “protect the user” follows this rule by itself.',
  mlab_iv: 'One of that Pokémon’s individual values, from 0 to 31. 31 if the game did not give any.',
  mlab_type: 'A type.',
  mlab_type_by_number: 'The type with that number, counting from 0: Fighting, Flying, Poison, Ground, Rock, Bug, Ghost, Steel, Fire, Water, Grass, Electric, Psychic, Ice, Dragon, Dark.',
  mlab_first_type: 'The first of that Pokémon’s types.',

  // Traits
  mlab_steal_item: 'The user takes the item the target holds. Nothing happens if the target holds none, or the user already holds one.',
  mlab_take_item: 'That Pokémon’s item is gone for the rest of the battle.',
  mlab_give_item: 'Gives a Pokémon an item to hold. Nothing happens if it already holds one.',
  mlab_suppress_ability: 'That Pokémon counts as having no ability. The engine only keeps track of this; it does not know what abilities do.',
  mlab_set_type: 'That Pokémon has only this type from now on.',
  mlab_lose_type: 'That Pokémon no longer has this type, for as long as it stays in the battle.',
  mlab_set_weight: 'Changes that Pokémon’s weight in kilograms, for moves that read it.',
  mlab_payout: 'Adds to the money the user’s side gets when the battle ends. The game decides what to do with it.',

  // Text
  mlab_message: 'Shows a line of text in the battle. Type {user} for the user’s name and {target} for the target’s.',
  mlab_announce: 'Shows “… used …!” at this point instead of at the start of the move.',
  mlab_splash: 'Shows “But nothing happened!”. One time in a hundred it hits the target for 1/16 of its max HP instead.',
};

// ---------------------------------------------------------------------------

// Rules only fit inside an effect, and nothing else fits there.
const statement = { previousStatement: 'MlabStatement', nextStatement: 'MlabStatement', inputsInline: true };
const lastStatement = { previousStatement: 'MlabStatement', inputsInline: true };
const rule = { previousStatement: 'MlabRule', nextStatement: 'MlabRule', inputsInline: true, style: 'effect_blocks' };
const body = (name: string) => ({ type: 'input_statement', name, check: 'MlabStatement' });
const dropdown = (name: string, options: Menu) => ({ type: 'field_dropdown', name, options });
const number = (name: string, value: number, min: number, max: number, precision = 1) => ({
  type: 'field_number', name, value, min, max, precision,
});
const pct = (name: string, value: number) => number(name, value, 0, 100, 0.01);
const times = (name: string, value: number) => number(name, value, 0, 16, 0.01);
const input = (name: string, check: string) => ({ type: 'input_value', name, check });
const word = (name: string, value: string) => ({ type: 'field_input', name, text: value });
const endRow = { type: 'input_end_row' };
const info = { style: 'info_blocks', classes: 'mlab-info' };

const WITH_OPTIONS: Record<string, object> = {
  mlab_damage: {
    message0: 'deal damage with power %1',
    args0: [input('POWER', 'Number')],
    ...statement,
    style: 'damage_blocks',
  },
  mlab_exact_damage: {
    message0: 'deal exactly %1 HP of damage',
    args0: [input('AMOUNT', 'Number')],
    ...statement,
    style: 'damage_blocks',
  },
  mlab_heal: {
    message0: 'heal the user by %1 % of its max HP',
    args0: [pct('PERCENT', 50)],
    ...statement,
    style: 'heal_blocks',
  },
  mlab_status: {
    message0: 'give %1 %2 %3 % of the time',
    args0: [dropdown('WHO', [WHO_MENU[1], WHO_MENU[0], WHO_MENU[2]]), dropdown('STATUS', GIVE_STATUS_MENU), pct('CHANCE', 100)],
    ...statement,
    style: 'status_blocks',
  },
  mlab_two_turn: {
    message0: 'charge for a turn saying %1 %2 then hit with power %3',
    args0: [word('MESSAGE', '{user} is getting ready!'), endRow, number('POWER', 80, 1, 999)],
    ...statement,
    style: 'turn_blocks',
  },
  mlab_consecutive: {
    message0: 'keep attacking for %1 to %2 turns %3 with power %4',
    args0: [number('MIN', 2, 1, 8), number('MAX', 3, 1, 8), endRow, number('POWER', 120, 1, 999)],
    ...statement,
    style: 'turn_blocks',
  },
  mlab_stats: {
    message0: 'change %1 %2 by %3',
    args0: [dropdown('WHO', WHOSE_MENU), dropdown('STAT', STAT_MENU), dropdown('STAGES', STAGE_MENU)],
    ...statement,
    style: 'stats_blocks',
  },
  mlab_effect: {
    message0: 'for %1 turns %2 gets %3 the effect %4',
    args0: [number('TURNS', 5, 1, 99), dropdown('SCOPE', SCOPE_MENU), endRow, word('NAME', 'my effect')],
    ...statement,
    style: 'effect_blocks',
  },
  mlab_mark: {
    message0: 'set %1 mark %2 to %3',
    args0: [dropdown('SCOPE', SCOPE_OF_MENU), word('NAME', 'my mark'), input('VALUE', 'Number')],
    ...statement,
    style: 'effect_blocks',
  },
  mlab_rule_damage: { message0: 'loses %1 % of max HP each turn', args0: [pct('PERCENT', 12.5)], ...rule },
  mlab_rule_heal: { message0: 'regains %1 % of max HP each turn', args0: [pct('PERCENT', 6.25)], ...rule },
  mlab_rule_drain: {
    message0: 'has %1 % of max HP drained %2 by %3 each turn',
    args0: [pct('PERCENT', 12.5), endRow, dropdown('TO', WHO_MENU)],
    ...rule,
  },
  mlab_rule_damage_taken: {
    message0: 'takes × %1 damage %2 from %3 moves',
    args0: [times('FACTOR', 0.5), endRow, dropdown('CATEGORY', MOVE_KIND_MENU)],
    ...rule,
  },
  mlab_rule_damage_dealt: {
    message0: 'deals × %1 damage %2 with %3 moves',
    args0: [times('FACTOR', 1.5), endRow, dropdown('CATEGORY', MOVE_KIND_MENU)],
    ...rule,
  },
  mlab_rule_block_status: { message0: 'cannot be given %1', args0: [dropdown('STATUS', CURE_STATUS_MENU)], ...rule },
};

/** Blocks whose "+ option" sits on the first row because that row is short. */
const OPTION_ON_FIRST_ROW = new Set(['mlab_damage', 'mlab_exact_damage', 'mlab_rule_block_status', 'mlab_rule_heal', 'mlab_rule_damage']);

const PLAIN: object[] = [
  // Starts
  { type: 'mlab_on_use', message0: 'when this move is used on %1', args0: [dropdown('TARGET', TARGET_MENU)], nextStatement: 'MlabStatement', style: 'hat_blocks' },
  { type: 'mlab_on_hit', message0: 'when the user is hit while watching', nextStatement: 'MlabStatement', style: 'hat_blocks' },
  { type: 'mlab_on_interrupt', message0: 'when a multi-turn move is cut short', nextStatement: 'MlabStatement', style: 'hat_blocks' },
  { type: 'mlab_on_turn_start', message0: 'at the start of a turn %1 where this move was chosen', args0: [endRow], nextStatement: 'MlabStatement', style: 'hat_blocks' },

  // Damage
  { type: 'mlab_fixed_damage', message0: 'deal exactly %1 HP of damage', args0: [number('AMOUNT', 40, 1, 9999)], ...statement, style: 'damage_blocks' },
  { type: 'mlab_level_damage', message0: 'deal damage equal to the user’s level', ...statement, style: 'damage_blocks' },
  { type: 'mlab_ohko', message0: 'knock out in one hit', ...statement, style: 'damage_blocks' },
  {
    type: 'mlab_multi_hit',
    message0: 'hit %1 to %2 times with power %3',
    args0: [number('MIN', 2, 1, 10), number('MAX', 5, 1, 10), number('POWER', 25, 1, 999)],
    ...statement,
    style: 'damage_blocks',
  },
  { type: 'mlab_recoil_max_hp', message0: 'the user takes %1 % of its max HP as recoil', args0: [pct('PERCENT', 25)], ...statement, style: 'damage_blocks' },
  { type: 'mlab_faint_user', message0: 'the user faints', ...statement, style: 'damage_blocks' },
  {
    type: 'mlab_hurt',
    message0: '%1 loses %2 %3',
    args0: [dropdown('WHO', WHO_MENU), input('AMOUNT', 'Number'), dropdown('UNIT', UNIT_MENU)],
    ...statement,
    style: 'damage_blocks',
  },
  {
    type: 'mlab_as_type',
    message0: 'as a %1 move',
    args0: [input('TYPE', 'Type')],
    message1: '%1',
    args1: [body('DO')],
    ...statement,
    style: 'damage_blocks',
  },

  // Healing
  {
    type: 'mlab_restore',
    message0: '%1 regains %2 %3',
    args0: [dropdown('WHO', WHO_MENU), input('AMOUNT', 'Number'), dropdown('UNIT', UNIT_MENU)],
    ...statement,
    style: 'heal_blocks',
  },

  // Status
  { type: 'mlab_confuse_target', message0: 'confuse the target', ...statement, style: 'status_blocks' },
  { type: 'mlab_confuse_self', message0: 'confuse the user', ...statement, style: 'status_blocks' },
  { type: 'mlab_flinch', message0: 'make the target flinch %1 % of the time', args0: [pct('CHANCE', 30)], ...statement, style: 'status_blocks' },
  { type: 'mlab_bind', message0: 'trap the target for a few turns', ...statement, style: 'status_blocks' },
  { type: 'mlab_protect', message0: 'protect the user this turn', ...statement, style: 'status_blocks' },
  { type: 'mlab_cure', message0: 'cure %1 of %2', args0: [dropdown('WHO', WHO_MENU), dropdown('STATUS', CURE_STATUS_MENU)], ...statement, style: 'status_blocks' },
  { type: 'mlab_free', message0: 'free the user from trapping moves', ...statement, style: 'status_blocks' },
  { type: 'mlab_break_protect', message0: 'lift the target’s protection', ...statement, style: 'status_blocks' },

  // Stats
  {
    type: 'mlab_boost_next',
    message0: 'boost the user’s next %1 move × %2',
    args0: [dropdown('TYPE', TYPE_MENU), number('MULT', 2, 0.1, 8, 0.1)],
    ...statement,
    style: 'stats_blocks',
  },
  { type: 'mlab_reset_stats', message0: 'reset %1 stat changes', args0: [dropdown('WHO', WHOSE_MENU)], ...statement, style: 'stats_blocks' },
  { type: 'mlab_steal_boosts', message0: 'the user steals the target’s stat raises', ...statement, style: 'stats_blocks' },
  { type: 'mlab_stage', message0: '%1 %2 stage', args0: [dropdown('WHO', WHOSE_MENU), dropdown('STAT', STAT_MENU)], output: 'Number', ...info },
  { type: 'mlab_raised_stages', message0: '%1 raised stat stages', args0: [dropdown('WHO', WHOSE_MENU)], output: 'Number', ...info },

  // Field
  { type: 'mlab_weather', message0: 'start %1 for %2 turns', args0: [dropdown('WEATHER', WEATHER_MENU), number('TURNS', 5, 1, 99)], ...statement, style: 'field_blocks' },
  { type: 'mlab_clear_weather', message0: 'end the weather', ...statement, style: 'field_blocks' },

  // Effects
  { type: 'mlab_end_effect', message0: 'end the effect %1 on %2', args0: [word('NAME', 'my effect'), dropdown('SCOPE', SCOPE_MENU)], ...statement, style: 'effect_blocks' },
  { type: 'mlab_end_group', message0: 'end the effects of the group %1 on %2', args0: [word('GROUP', 'terrain'), dropdown('SCOPE', SCOPE_MENU)], ...statement, style: 'effect_blocks' },
  { type: 'mlab_unmark', message0: 'remove %1 mark %2', args0: [dropdown('SCOPE', SCOPE_OF_MENU), word('NAME', 'my mark')], ...statement, style: 'effect_blocks' },
  { type: 'mlab_effect_active', message0: 'the effect %1 is active on %2', args0: [word('NAME', 'my effect'), dropdown('SCOPE', SCOPE_MENU)], output: 'Boolean', ...info },
  { type: 'mlab_mark_value', message0: '%1 mark %2', args0: [dropdown('SCOPE', SCOPE_OF_MENU), word('NAME', 'my mark')], output: 'Number', ...info },
  { type: 'mlab_has_mark', message0: '%1 has the mark %2', args0: [dropdown('SCOPE', SCOPE_MENU), word('NAME', 'my mark')], output: 'Boolean', ...info },
  { type: 'mlab_rule_stat', message0: 'has its %1 × %2', args0: [dropdown('STAT', BATTLE_STAT_MENU), times('FACTOR', 0.5)], ...rule },
  { type: 'mlab_rule_block_drops', message0: 'cannot have its stats lowered', ...rule },
  { type: 'mlab_rule_effect_chance', message0: 'has the chances of its moves’ %1 extra effects × %2', args0: [endRow, times('FACTOR', 2)], ...rule },
  { type: 'mlab_rule_move_type', message0: 'uses %1 moves as %2 moves', args0: [dropdown('FROM', TYPE_MENU), dropdown('TO', TYPE_MENU)], ...rule },
  { type: 'mlab_rule_grounded', message0: 'can be hit by Ground moves', ...rule },
  { type: 'mlab_rule_trapped', message0: 'cannot leave the battle', ...rule },
  { type: 'mlab_rule_endure', message0: 'survives any hit with 1 HP', ...rule },
  { type: 'mlab_rule_always_hit', message0: 'cannot miss %1', args0: [dropdown('AGAINST', [['anyone', 'any'], ['the target', 'target'], ['that Pokémon', 'each']])], ...rule },
  { type: 'mlab_rule_without_type', message0: 'loses its %1 type', args0: [dropdown('TYPE', TYPE_MENU)], ...rule },

  // Turns
  {
    type: 'mlab_force_move',
    message0: 'make this move take %1 turns, %2 aimed at %3',
    args0: [input('TURNS', 'Number'), endRow, dropdown('POLICY', [['the same target', 'SameTarget'], ['a random foe', 'RandomOpponent']])],
    ...statement,
    style: 'turn_blocks',
  },
  { type: 'mlab_break_sequence', message0: 'end the multi-turn move now', ...statement, style: 'turn_blocks' },
  { type: 'mlab_recharge', message0: 'the user must recharge next turn', ...statement, style: 'turn_blocks' },
  { type: 'mlab_watch_hits', message0: 'watch for hits until the user’s next action', ...statement, style: 'turn_blocks' },
  { type: 'mlab_fail', message0: 'the move fails', ...lastStatement, style: 'turn_blocks' },
  { type: 'mlab_stop', message0: 'stop here', ...lastStatement, style: 'turn_blocks' },
  { type: 'mlab_hide', message0: 'the user hides %1 %2 until it next acts', args0: [dropdown('HIDDEN', HIDDEN_MENU), endRow], ...statement, style: 'turn_blocks' },
  { type: 'mlab_unhide', message0: 'knock %1 out of hiding', args0: [dropdown('WHO', [WHO_MENU[1], WHO_MENU[2]])], ...statement, style: 'turn_blocks' },
  { type: 'mlab_hurry', message0: 'let the others who chose %1 this move act next', args0: [endRow], ...statement, style: 'turn_blocks' },
  {
    type: 'mlab_sure',
    message0: 'ignoring protection, hiding and accuracy',
    message1: '%1',
    args1: [body('DO')],
    ...statement,
    style: 'turn_blocks',
  },
  { type: 'mlab_if_reaches', message0: 'the move reaches %1', args0: [dropdown('WHO', [WHO_MENU[1], WHO_MENU[2]])], output: 'Boolean', style: 'turn_blocks' },

  // Logic
  { type: 'mlab_if', message0: 'if %1 then', args0: [input('COND', 'Boolean')], message1: '%1', args1: [body('DO')], ...statement, style: 'logic_blocks' },
  {
    type: 'mlab_if_else',
    message0: 'if %1 then',
    args0: [input('COND', 'Boolean')],
    message1: '%1',
    args1: [body('DO')],
    message2: 'else',
    message3: '%1',
    args3: [body('ELSE')],
    ...statement,
    style: 'logic_blocks',
  },
  { type: 'mlab_repeat', message0: 'repeat %1 times', args0: [input('TIMES', 'Number')], message1: '%1', args1: [body('DO')], ...statement, style: 'logic_blocks' },
  { type: 'mlab_repeat_count', message0: 'repeat number', output: 'Number', style: 'logic_blocks' },
  { type: 'mlab_for_each', message0: 'for each %1', args0: [dropdown('GROUP', GROUP_MENU)], message1: '%1', args1: [body('DO')], ...statement, style: 'logic_blocks' },
  {
    type: 'mlab_compare',
    message0: '%1 %2 %3',
    args0: [input('A', 'Number'), dropdown('OP', [['=', 'EQ'], ['≠', 'NEQ'], ['<', 'LT'], ['≤', 'LTE'], ['>', 'GT'], ['≥', 'GTE']]), input('B', 'Number')],
    output: 'Boolean',
    inputsInline: true,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_and_or',
    message0: '%1 %2 %3',
    args0: [input('A', 'Boolean'), dropdown('OP', [['and', 'AND'], ['or', 'OR']]), input('B', 'Boolean')],
    output: 'Boolean',
    inputsInline: true,
    style: 'logic_blocks',
  },
  { type: 'mlab_not', message0: 'not %1', args0: [input('A', 'Boolean')], output: 'Boolean', inputsInline: true, style: 'logic_blocks' },
  {
    type: 'mlab_arith',
    message0: '%1 %2 %3',
    args0: [input('A', 'Number'), dropdown('OP', [['+', 'ADD'], ['−', 'MINUS'], ['×', 'MULTIPLY'], ['÷', 'DIVIDE'], ['to the power of', 'POWER']]), input('B', 'Number')],
    output: 'Number',
    inputsInline: true,
    style: 'logic_blocks',
  },
  {
    type: 'mlab_min_max',
    message0: '%1 of %2 and %3',
    args0: [dropdown('OP', [['the smaller', 'MIN'], ['the larger', 'MAX']]), input('A', 'Number'), input('B', 'Number')],
    output: 'Number',
    inputsInline: true,
    style: 'logic_blocks',
  },
  { type: 'mlab_round', message0: '%1 rounded down', args0: [input('A', 'Number')], output: 'Number', inputsInline: true, style: 'logic_blocks' },
  {
    type: 'mlab_random',
    message0: 'random number from %1 to %2',
    args0: [input('FROM', 'Number'), input('TO', 'Number')],
    output: 'Number',
    inputsInline: true,
    style: 'logic_blocks',
  },
  { type: 'mlab_chance', message0: '%1 % chance', args0: [pct('PERCENT', 30)], output: 'Boolean', style: 'logic_blocks' },
  { type: 'mlab_number', message0: '%1', args0: [{ type: 'field_number', name: 'NUM', value: 0 }], output: 'Number', style: 'logic_blocks' },

  // Battle info
  {
    type: 'mlab_hp',
    message0: '%1 %2',
    args0: [
      dropdown('WHO', WHOSE_MENU),
      dropdown('FACT', [
        ['HP', 'hp'], ['max HP', 'max_hp'], ['level', 'level'], ['weight in kg', 'weight'],
        ['Attack', 'attack'], ['Defense', 'defense'], ['Sp. Atk', 'sp_attack'], ['Sp. Def', 'sp_defense'], ['Speed', 'speed'],
        ['damage taken this turn', 'damage_taken'], ['damage of the last hit taken this turn', 'last_hit_damage'],
        ['turns in the battle', 'turns_on_field'], ['happiness', 'happiness'],
      ]),
    ],
    output: 'Number',
    ...info,
  },
  { type: 'mlab_turn', message0: 'turn of this move', output: 'Number', ...info },
  { type: 'mlab_last_damage', message0: 'damage of the last hit', output: 'Number', ...info },
  { type: 'mlab_count', message0: 'number of %1', args0: [dropdown('GROUP', COUNT_MENU)], output: 'Number', ...info },
  { type: 'mlab_streak', message0: 'turns in a row this move was used', output: 'Number', ...info },
  { type: 'mlab_repeat_works', message0: 'repeating this move works', output: 'Boolean', ...info },
  { type: 'mlab_iv', message0: '%1 %2 IV', args0: [dropdown('WHO', WHOSE_MENU), dropdown('STAT', [['HP', 'hp'], ['Attack', 'attack'], ['Defense', 'defense'], ['Sp. Atk', 'sp_attack'], ['Sp. Def', 'sp_defense'], ['Speed', 'speed']])], output: 'Number', ...info },
  { type: 'mlab_status_is', message0: '%1 %2 %3', args0: [dropdown('WHO', WHO_MENU), dropdown('OP', IS_MENU), dropdown('STATUS', STATUS_IS_MENU)], output: 'Boolean', ...info },
  { type: 'mlab_type_is', message0: '%1 %2 a %3 type', args0: [dropdown('WHO', WHO_MENU), dropdown('OP', IS_MENU), dropdown('TYPE', TYPE_MENU)], output: 'Boolean', ...info },
  {
    type: 'mlab_state_is',
    message0: '%1 %2',
    args0: [
      dropdown('WHO', WHO_MENU),
      dropdown('STATE', [
        ['has acted this turn', 'acted'], ['chose a damaging move', 'damaging'], ['was hurt this turn', 'hurt'],
        ['is confused', 'confused'], ['is protected', 'protected'], ['is trapped', 'trapped'],
        ['is hiding', 'hidden'], ['is in the air', 'Air'], ['is underground', 'Underground'], ['is underwater', 'Underwater'],
        ['holds an item', 'item'], ['has an ability', 'ability'],
        ['is male', 'Male'], ['is female', 'Female'], ['has no gender', 'Genderless'],
        ['has fainted', 'fainted'], ['is on the user’s side', 'ally'],
      ]),
    ],
    output: 'Boolean',
    ...info,
  },
  { type: 'mlab_pair_is', message0: 'the user and the target %1', args0: [dropdown('PAIR', [['are of opposite genders', 'genders'], ['share a type', 'type']])], output: 'Boolean', ...info },
  {
    type: 'mlab_hit_by_target',
    message0: '%1 was the last to hit %2 the user this turn',
    args0: [dropdown('WHO', [WHO_MENU[1], WHO_MENU[2]]), endRow],
    output: 'Boolean',
    ...info,
  },
  {
    type: 'mlab_text_is',
    message0: '%1 %2 %3 %4',
    args0: [
      dropdown('WHO', WHOSE_MENU),
      dropdown('WHAT', [['item', 'item'], ['ability', 'ability'], ['species', 'species']]),
      dropdown('OP', [['is', 'IS'], ['contains', 'HAS']]),
      word('TEXT', 'berry'),
    ],
    output: 'Boolean',
    ...info,
  },
  { type: 'mlab_weather_is', message0: 'weather %1 %2', args0: [dropdown('OP', IS_MENU), dropdown('WEATHER', WEATHER_IS_MENU)], output: 'Boolean', ...info },
  { type: 'mlab_env_is', message0: 'the battle takes place in %1', args0: [word('TEXT', 'cave')], output: 'Boolean', ...info },
  {
    type: 'mlab_hit_landed',
    message0: 'the last hit %1',
    args0: [dropdown('RESULT', [['landed', 'hit'], ['was a critical hit', 'critical'], ['knocked out its target', 'fainted'], ['was blocked by protection', 'blocked'], ['had no effect', 'immune']])],
    output: 'Boolean',
    ...info,
  },
  { type: 'mlab_hit_was', message0: 'the hit taken %1', args0: [dropdown('KIND', [['made contact', 'contact'], ['was physical', 'Physical'], ['was special', 'Special']])], output: 'Boolean', ...info },
  { type: 'mlab_first_turn', message0: 'first turn of this move', output: 'Boolean', ...info },
  { type: 'mlab_last_turn', message0: 'last turn of this move', output: 'Boolean', ...info },
  { type: 'mlab_in_sequence', message0: 'already a multi-turn move', output: 'Boolean', ...info },
  { type: 'mlab_first_turn_out', message0: 'the user’s first turn in the battle', output: 'Boolean', ...info },
  { type: 'mlab_used_all', message0: 'the user has used all its other moves', output: 'Boolean', ...info },
  { type: 'mlab_last_failed', message0: 'the user’s last move failed', output: 'Boolean', ...info },
  { type: 'mlab_ally_fainted', message0: 'an ally of the user fainted last turn', output: 'Boolean', ...info },
  { type: 'mlab_type', message0: '%1', args0: [dropdown('TYPE', TYPE_MENU)], output: 'Type', ...info },
  { type: 'mlab_type_by_number', message0: 'type number %1', args0: [input('NUMBER', 'Number')], output: 'Type', inputsInline: true, ...info },
  { type: 'mlab_first_type', message0: '%1 first type', args0: [dropdown('WHO', WHOSE_MENU)], output: 'Type', ...info },

  // Traits
  { type: 'mlab_steal_item', message0: 'the user takes the target’s item', ...statement, style: 'trait_blocks' },
  { type: 'mlab_take_item', message0: 'remove %1 item', args0: [dropdown('WHO', WHOSE_MENU)], ...statement, style: 'trait_blocks' },
  { type: 'mlab_give_item', message0: 'give %1 the item %2', args0: [dropdown('WHO', WHO_MENU), word('ITEM', 'oran_berry')], ...statement, style: 'trait_blocks' },
  { type: 'mlab_suppress_ability', message0: 'suppress %1 ability', args0: [dropdown('WHO', WHOSE_MENU)], ...statement, style: 'trait_blocks' },
  { type: 'mlab_set_type', message0: '%1 becomes a %2 type', args0: [dropdown('WHO', WHO_MENU), input('TYPE', 'Type')], ...statement, style: 'trait_blocks' },
  { type: 'mlab_lose_type', message0: '%1 loses its %2 type', args0: [dropdown('WHO', WHO_MENU), dropdown('TYPE', TYPE_MENU)], ...statement, style: 'trait_blocks' },
  { type: 'mlab_set_weight', message0: 'set %1 weight to %2 kg', args0: [dropdown('WHO', WHOSE_MENU), input('WEIGHT', 'Number')], ...statement, style: 'trait_blocks' },
  { type: 'mlab_payout', message0: 'the user’s side earns %1 money', args0: [input('AMOUNT', 'Number')], ...statement, style: 'trait_blocks' },

  // Text
  { type: 'mlab_message', message0: 'show message %1', args0: [word('TEXT', '{user} is ready!')], ...statement, style: 'text_blocks' },
  { type: 'mlab_announce', message0: 'announce the move', ...statement, style: 'text_blocks' },
  { type: 'mlab_splash', message0: 'nothing happens', ...statement, style: 'text_blocks' },
];

let defined = false;

export function defineBlocks(): void {
  if (defined) return;
  defined = true;
  Blockly.defineBlocksWithJsonArray(PLAIN.map((json) => ({ ...json, tooltip: TIPS[(json as { type: string }).type] })));
  for (const [type, json] of Object.entries(WITH_OPTIONS)) {
    Blockly.Blocks[type] = {
      init(this: OptionBlock) {
        this.jsonInit({ ...json, tooltip: TIPS[type] });
        installOptions(this, OPTIONS[type], !OPTION_ON_FIRST_ROW.has(type));
        // An effect holds its rules below its option rows.
        if (type === 'mlab_effect') this.appendStatementInput('RULES').setCheck('MlabRule');
      },
    };
  }
}

/** The rows that "+ option" can add to a block. */
export function optionSpecs(type: string): OptionSpec[] {
  return OPTIONS[type] ?? [];
}

export const HAT_TYPES = ['mlab_on_use', 'mlab_on_turn_start', 'mlab_on_hit', 'mlab_on_interrupt'];
/** Blocks that only exist as rows inside an effect. */
export const RULE_TYPES = [
  'mlab_rule_damage', 'mlab_rule_heal', 'mlab_rule_drain', 'mlab_rule_damage_taken', 'mlab_rule_damage_dealt', 'mlab_rule_stat',
  'mlab_rule_block_status', 'mlab_rule_block_drops', 'mlab_rule_effect_chance', 'mlab_rule_move_type', 'mlab_rule_grounded',
  'mlab_rule_trapped', 'mlab_rule_endure', 'mlab_rule_always_hit', 'mlab_rule_without_type',
];
