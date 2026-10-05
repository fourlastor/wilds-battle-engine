// Turns a block workspace into a move file.
//
// The engine accepts two shapes: a plain `effects` list, or a `script` function (plus optional
// `on_hit` / `on_interrupt`). A move made only of blocks the effects list can express becomes a
// plain move; anything with logic becomes a script. Blocks the chosen shape cannot express are
// reported as problems instead of being silently dropped.
import * as Blockly from 'blockly/core';
import { LuaGenerator, Order } from 'blockly/lua';
import type { MoveSheet } from '../model.ts';
import { RULE_TYPES, hasOption, optionIds, optionSuffix } from './definitions.ts';

export interface Problem {
  blockId: string | null;
  message: string;
}

export type EffectRole = 'damage' | 'heal' | 'status' | 'stats' | 'weather' | 'other';

export interface Generated {
  /** The file to download. */
  lua: string;
  /** Same file with ` --@n` tags naming the block behind each line. */
  annotated: string;
  /** Same behaviour, plus invisible marker messages so the test battle knows which block ran. */
  traced: string;
  mode: 'effects' | 'script' | 'empty';
  problems: Problem[];
  /** Tag / marker number → block id. */
  marks: string[];
  /** Plain moves cannot carry markers; the battle matches their events by kind instead. */
  effectBlocks: { id: string; role: EffectRole }[];
}

/** Marker messages start with this byte; see bench/trace.ts for the grammar. */
export const MARK = '\u0001';

type PassKind = 'annotated' | 'traced';
type Hook = 'script' | 'on_hit' | 'on_interrupt' | 'on_turn_start';

const HOOKS: Record<string, Hook> = {
  mlab_on_use: 'script',
  mlab_on_hit: 'on_hit',
  mlab_on_interrupt: 'on_interrupt',
  mlab_on_turn_start: 'on_turn_start',
};

interface Pass {
  kind: PassKind;
  marks: string[];
  problems: Map<string, Problem>;
  useResult: boolean;
  /** Why this move is a script, said to the blocks that only exist in plain moves. */
  cause: string;
  /** The stack being written. Only the main one holds effects back until the move connects. */
  hook: Hook;
  /** How many “ignoring protection, hiding and accuracy” blocks are around the block being written. */
  sure: number;
}

let pass: Pass;

const lua = new LuaGenerator('MoveLab');
lua.INDENT = '  ';

const PLAIN_ONLY = 'This block only works in plain moves for now.';

/** The engine's ready-made multi-turn effects, which have no script calls. */
const EFFECT_ONLY = new Set(['mlab_two_turn', 'mlab_consecutive']);

/** A move aimed at several Pokémon reaches them all only as a script. */
const MANY_TARGETS: Record<string, string> = {
  AllOthers: 'everyone else',
  AllOpponents: 'all foes',
  AllAllies: 'all allies',
  UserAndAllies: 'the user and its allies',
  All: 'everyone',
};

/** What the engine lets a stack do while someone else is acting, or before the turn starts. */
const REACTION_STATEMENTS = new Set([
  'mlab_message', 'mlab_stats', 'mlab_reset_stats', 'mlab_status', 'mlab_cure', 'mlab_confuse_target', 'mlab_hurt', 'mlab_restore',
  'mlab_mark', 'mlab_unmark', 'mlab_effect', 'mlab_end_effect', 'mlab_end_group', 'mlab_steal_item', 'mlab_take_item', 'mlab_give_item',
  'mlab_payout', 'mlab_if', 'mlab_if_else', 'mlab_repeat', 'mlab_for_each', 'mlab_sure', 'mlab_stop',
]);
const TURN_START_STATEMENTS = new Set(['mlab_watch_hits', 'mlab_protect']);
const REACTION_BANNED_VALUES = new Set(['mlab_if_reaches']);

// ---------------------------------------------------------------------------
// Small formatting helpers

function num(value: number): string {
  if (!Number.isFinite(value)) return '0';
  return String(Number(value.toFixed(4)));
}

/** A percentage field as the 0–1 fraction the engine expects. Thirds stay exact. */
function fraction(percent: number): string {
  if (Math.abs(percent - 33.33) < 0.011) return '1/3';
  if (Math.abs(percent - 66.67) < 0.011) return '2/3';
  return num(percent / 100);
}

function quote(text: string): string {
  return `"${text.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\r?\n/g, '\\n')}"`;
}

/** Text typed on a block as a Lua expression: {user}, {target} and {pokemon} become names. */
function messageExpr(text: string, block?: Blockly.Block): string {
  const names: Record<string, string> = { '{user}': 'ctx.user.name', '{target}': 'ctx.target.name' };
  if (block && loopOf(block)) names['{pokemon}'] = `${LOOP_VARIABLE}.name`;
  const pieces: string[] = [];
  for (const part of text.split(/(\{user\}|\{target\}|\{pokemon\})/)) {
    if (!part) continue;
    pieces.push(names[part] ?? quote(part));
  }
  return pieces.length ? pieces.join(' .. ') : '""';
}

function field(block: Blockly.Block, name: string): string {
  return String(block.getFieldValue(name) ?? '');
}
function fieldNumber(block: Blockly.Block, name: string): number {
  return Number(block.getFieldValue(name) ?? 0);
}

/** The number in a value socket if it is a plain number, otherwise null. */
function literal(block: Blockly.Block, input: string): number | null {
  const target = block.getInputTargetBlock(input);
  return target && target.type === 'mlab_number' ? Number(target.getFieldValue('NUM')) : null;
}

function value(block: Blockly.Block, input: string, order: number, fallback: string): string {
  return lua.valueToCode(block, input, order) || fallback;
}

function markOf(block: Blockly.Block): number {
  let index = pass.marks.indexOf(block.id);
  if (index < 0) index = pass.marks.push(block.id) - 1;
  return index;
}

function marker(body: string): string {
  return `ctx:message("\\1${body}")\n`;
}

/** Wraps one statement: a marker before it when tracing, a line tag otherwise. */
function statement(block: Blockly.Block, code: string, traceSuffix = ''): string {
  const mark = markOf(block);
  if (pass.kind === 'traced') return marker(`@${mark}${traceSuffix}`) + code;
  const end = code.indexOf('\n');
  return `${code.slice(0, end)} --@${mark}${code.slice(end)}`;
}

function problem(block: Blockly.Block | null, message: string): void {
  const key = `${block?.id ?? ''}|${message}`;
  if (!pass.problems.has(key)) pass.problems.set(key, { blockId: block?.id ?? null, message });
}

/** Reports a block that a script cannot hold, and says what made the move a script. */
function unsupported(block: Blockly.Block, reason: string): string {
  problem(block, `${reason} ${pass.cause}`);
  return '';
}

function indent(code: string): string {
  return lua.prefixLines(code, lua.INDENT);
}

// ---------------------------------------------------------------------------
// Who a block is about

/** The name "that Pokémon" has inside a "for each" block. */
const LOOP_VARIABLE = 'pokemon';

function loopOf(block: Blockly.Block): Blockly.Block | null {
  for (let parent = block.getSurroundParent(); parent; parent = parent.getSurroundParent()) {
    if (parent.type === 'mlab_for_each') return parent;
  }
  return null;
}

/** A Pokémon picked in a menu, as Lua. */
function who(block: Blockly.Block, name = 'WHO'): string {
  const choice = field(block, name);
  if (choice === 'each') {
    if (!loopOf(block)) problem(block, '“that Pokémon” only means something inside a “for each” block.');
    return LOOP_VARIABLE;
  }
  return choice === 'user' ? 'ctx.user' : 'ctx.target';
}

/** A Pokémon, a side or the field picked in a menu, as Lua. */
function scope(block: Blockly.Block, name = 'SCOPE'): string {
  switch (field(block, name)) {
    case 'user_side':
      return 'ctx.user.team';
    case 'target_side':
      return 'ctx.target.team';
    case 'field':
      return 'ctx.field';
    default:
      return who(block, name);
  }
}

/** The Pokémon a place menu names, if it is one other than the user: what a block on it waits for. */
function scopeGate(block: Blockly.Block, name = 'SCOPE'): string {
  const choice = field(block, name);
  return choice === 'target' || choice === 'each' ? scope(block, name) : 'ctx.user';
}

/**
 * Holds an effect on another Pokémon back until the move has got through to it: the hit already
 * made on it decides, or else protection, hiding and accuracy are checked now, once.
 * Stacks that react to something act at once, and so does anything a Pokémon does to itself.
 */
function gate(target: string, code: string): string {
  if (pass.hook !== 'script' || pass.sure > 0 || target === 'ctx.user') return code;
  return `if ctx:reached(${target === 'ctx.target' ? '' : target}) then\n${indent(code)}end\n`;
}

/** The type set by the nearest "as a … move" block around this one, if any. */
function typeOverride(block: Blockly.Block): string | null {
  for (let parent = block.getSurroundParent(); parent; parent = parent.getSurroundParent()) {
    if (parent.type === 'mlab_as_type') return 'move_type';
  }
  return null;
}

// ---------------------------------------------------------------------------
// The two shapes a block can take

function statRows(block: Blockly.Block): [string, number][] {
  const rows: [string, number][] = [[field(block, 'STAT'), fieldNumber(block, 'STAGES')]];
  for (const id of optionIds(block, 'stat')) {
    const suffix = optionSuffix(id);
    rows.push([field(block, `STAT${suffix}`), fieldNumber(block, `STAGES${suffix}`)]);
  }
  return rows;
}

/** Why a block cannot be part of a script, or null if it can. */
function scriptReason(block: Blockly.Block): string | null {
  return EFFECT_ONLY.has(block.type) ? PLAIN_ONLY : null;
}

/** The options of "deal damage" that the effects list also has. */
const PLAIN_DAMAGE_OPTIONS = new Set(['recoil', 'drain', 'high_crit', 'always_crit']);

/**
 * The block as an entry of the `effects` list, or null if it has no such form.
 * `target` is who the move is used on: the engine applies a plain move's effects to that Pokémon.
 */
function effectForm(block: Blockly.Block, target: string): string | null {
  const entry = (kind: string, ...fields: (string | false)[]) =>
    `{${[`kind = Effect.${kind}`, ...fields.filter((item): item is string => item !== false)].join(', ')}}`;
  switch (block.type) {
    case 'mlab_damage': {
      const power = literal(block, 'POWER');
      if (power === null) return null;
      if (optionIds(block).some((id) => !PLAIN_DAMAGE_OPTIONS.has(id.split('#')[0]))) return null;
      return entry(
        'Damage',
        `power = ${num(power)}`,
        hasOption(block, 'recoil') && `recoil = ${fraction(fieldNumber(block, 'RECOIL'))}`,
        hasOption(block, 'drain') && `drain = ${fraction(fieldNumber(block, 'DRAIN'))}`,
        hasOption(block, 'high_crit') && 'high_crit = true',
        hasOption(block, 'always_crit') && 'always_crit = true',
      );
    }
    case 'mlab_fixed_damage':
      return entry('FixedDamage', `amount = ${num(fieldNumber(block, 'AMOUNT'))}`);
    case 'mlab_exact_damage': {
      const amount = literal(block, 'AMOUNT');
      if (amount === null || optionIds(block).length) return null;
      return entry('FixedDamage', `amount = ${num(amount)}`);
    }
    case 'mlab_level_damage':
      return entry('LevelDamage');
    case 'mlab_ohko':
      return entry('Ohko');
    case 'mlab_multi_hit':
      return entry('MultiHit', `power = ${num(fieldNumber(block, 'POWER'))}`, `min_hits = ${num(fieldNumber(block, 'MIN'))}`, `max_hits = ${num(fieldNumber(block, 'MAX'))}`);
    case 'mlab_heal':
      // The Heal effect heals whoever the move is used on; only a script can heal the user of a move aimed elsewhere.
      if (target !== 'User') return null;
      return entry('Heal', `fraction = ${fraction(fieldNumber(block, 'PERCENT'))}`, hasOption(block, 'quiet') && 'hide_message = true');
    case 'mlab_status': {
      // The Status effect acts on whoever the move is used on, and has its own way of reporting failure.
      if (field(block, 'WHO') !== 'target' || hasOption(block, 'announce')) return null;
      const chance = fieldNumber(block, 'CHANCE');
      return entry('Status', `status = Status.${field(block, 'STATUS')}`, chance < 100 && `chance = ${fraction(chance)}`, hasOption(block, 'replace') && 'replace = true');
    }
    case 'mlab_confuse_target':
      return entry('Confuse');
    case 'mlab_flinch':
      return entry('Flinch', `chance = ${fraction(fieldNumber(block, 'CHANCE'))}`);
    case 'mlab_bind':
      return entry('Bind');
    case 'mlab_protect':
      return entry('Protect');
    case 'mlab_stats': {
      if (field(block, 'WHO') === 'each') return null;
      const stages = statRows(block).map(([stat, delta]) => `[Stat.${stat}] = ${delta}`).join(', ');
      return entry(
        'Stats',
        hasOption(block, 'chance') && `chance = ${fraction(fieldNumber(block, 'CHANCE'))}`,
        field(block, 'WHO') === 'user' && 'self = true',
        `stages = {${stages}}`,
      );
    }
    case 'mlab_weather':
      return entry('Weather', `weather = Weather.${field(block, 'WEATHER')}`, `turns = ${num(fieldNumber(block, 'TURNS'))}`);
    case 'mlab_two_turn':
      return entry(
        'TwoTurn',
        hasOption(block, 'hidden') && 'semi_invulnerable = true',
        hasOption(block, 'hidden') && field(block, 'PLACE') !== 'Vanished' && `hidden = Hidden.${field(block, 'PLACE')}`,
        hasOption(block, 'skip_sun') && 'skip_in_sun = true',
        // The engine substitutes {0} with the user's name.
        `charge_message = ${quote(field(block, 'MESSAGE').split('{user}').join('{0}'))}`,
        `power = ${num(fieldNumber(block, 'POWER'))}`,
      );
    case 'mlab_consecutive':
      return entry(
        'Consecutive',
        `power = ${num(fieldNumber(block, 'POWER'))}`,
        `min_turns = ${num(fieldNumber(block, 'MIN'))}`,
        `max_turns = ${num(fieldNumber(block, 'MAX'))}`,
        hasOption(block, 'confuse') && 'confuse_after = true',
        hasOption(block, 'double') && 'double_power = true',
      );
    case 'mlab_splash':
      return entry('Splash');
    default:
      return null;
  }
}

function effectRole(type: string): EffectRole {
  if (['mlab_damage', 'mlab_fixed_damage', 'mlab_exact_damage', 'mlab_level_damage', 'mlab_ohko', 'mlab_multi_hit', 'mlab_two_turn', 'mlab_consecutive'].includes(type)) return 'damage';
  if (type === 'mlab_heal') return 'heal';
  if (type === 'mlab_status' || type === 'mlab_confuse_target') return 'status';
  if (type === 'mlab_stats') return 'stats';
  if (type === 'mlab_weather') return 'weather';
  return 'other';
}

// ---------------------------------------------------------------------------
// Script generators

type Emit = (block: Blockly.Block) => string | [string, number] | null;
const emit = lua.forBlock as Record<string, Emit>;

function simple(type: string, call: string): void {
  emit[type] = (block) => statement(block, `${call}\n`);
}

simple('mlab_faint_user', 'ctx:faint_user()');
simple('mlab_confuse_self', 'ctx:confuse_self()');
simple('mlab_break_sequence', 'ctx:break_sequence()');
simple('mlab_recharge', 'ctx:recharge()');
simple('mlab_watch_hits', 'ctx:watch_hits_until_next_action()');
simple('mlab_announce', 'ctx:announce()');
simple('mlab_protect', 'ctx:protect()');
simple('mlab_free', 'ctx:free(ctx.user)');
simple('mlab_break_protect', 'ctx:break_protect()');
simple('mlab_clear_weather', 'ctx:clear_weather()');
simple('mlab_hurry', 'ctx:hurry()');

for (const type of EFFECT_ONLY) emit[type] = (block) => unsupported(block, PLAIN_ONLY);

/** Only the first words of a block, to point at it in a message. */
function shortName(block: Blockly.Block): string {
  for (const input of block.inputList) {
    for (const item of input.fieldRow) {
      if (item instanceof Blockly.FieldLabel && item.getText().trim()) return item.getText().trim();
    }
  }
  return block.type;
}

/** What a hit says so that nothing stops it, inside “ignoring protection, hiding and accuracy”. */
const SURE_HIT = ['never_miss = true', 'ignore_protect = true', 'hits_hidden = true'];

/** `result = ` in front of a hit when a block below asks about the last hit. */
function keep(): string {
  return pass.useResult ? 'result = ' : '';
}

emit['mlab_damage'] = (block) => {
  const options: string[] = [];
  if (hasOption(block, 'accuracy')) options.push(`accuracy = ${fraction(fieldNumber(block, 'ACC'))}`);
  if (hasOption(block, 'drain')) options.push(`drain = ${fraction(fieldNumber(block, 'DRAIN'))}`);
  if (hasOption(block, 'min_hp')) options.push(`min_target_hp = ${num(fieldNumber(block, 'MIN_HP'))}`);
  if (hasOption(block, 'typeless')) options.push('typeless = true');
  if (hasOption(block, 'recoil')) options.push(`recoil = ${fraction(fieldNumber(block, 'RECOIL'))}`);
  if (hasOption(block, 'high_crit')) options.push('high_crit = true');
  if (hasOption(block, 'always_crit')) options.push('always_crit = true');
  if (pass.sure > 0) {
    options.push(...SURE_HIT);
  } else {
    if (hasOption(block, 'never_miss')) options.push('never_miss = true');
    if (hasOption(block, 'through')) options.push('ignore_protect = true');
    if (hasOption(block, 'reach')) {
      const place = field(block, 'REACH');
      options.push(place === 'All' ? 'hits_hidden = true' : `hits_hidden = {ctx.Hidden.${place}}`);
    }
  }
  const type = typeOverride(block);
  if (type) options.push(`type = ${type}`);
  if (hasOption(block, 'also_type')) options.push(`also_type = ctx.Type.${field(block, 'ALSO_TYPE')}`);
  const beaten = optionIds(block, 'super').map((id) => `[ctx.Type.${field(block, `SUPER${optionSuffix(id)}`)}] = 2`);
  if (beaten.length) options.push(`effective = {${beaten.join(', ')}}`);
  if (hasOption(block, 'category')) options.push(`category = ctx.Category.${field(block, 'CATEGORY')}`);
  if (hasOption(block, 'attack')) {
    if (field(block, 'ATK_WHO') === 'target') options.push('attack_from = ctx.target');
    options.push(`attack_stat = ctx.Stat.${field(block, 'ATK_STAT')}`);
  }
  if (hasOption(block, 'defense')) options.push(`defense_stat = ctx.Stat.${field(block, 'DEF_STAT')}`);
  if (hasOption(block, 'ignore_stages')) options.push('ignore_stages = true');
  if (hasOption(block, 'only')) options.push(`target = ${who(block, 'ONLY')}`);
  const call = `ctx:damage(${value(block, 'POWER', Order.NONE, '0')}${options.length ? `, {${options.join(', ')}}` : ''})`;
  return statement(block, `${keep()}${call}\n`);
};

/** An exact amount of damage: the move has to reach its target first, like any attack. */
function directDamage(block: Blockly.Block, target: string, amount: string, options: string[], after = ''): string {
  const call = `ctx:direct_damage(${target}, ${amount}${options.length ? `, {${options.join(', ')}}` : ''})`;
  const hit = after ? `local hit = ${call}\n${pass.useResult ? 'result = hit\n' : ''}${after}` : `${keep()}${call}\n`;
  const reset = pass.useResult && pass.hook === 'script' ? 'result = nil\n' : '';
  return statement(block, reset + gate(target, hit));
}

emit['mlab_fixed_damage'] = (block) => directDamage(block, 'ctx.target', num(fieldNumber(block, 'AMOUNT')), []);
emit['mlab_exact_damage'] = (block) => {
  const options: string[] = [];
  if (hasOption(block, 'typeless')) options.push('typeless = true');
  if (hasOption(block, 'min_hp')) options.push(`min_target_hp = ${num(fieldNumber(block, 'MIN_HP'))}`);
  const target = hasOption(block, 'only') ? who(block, 'ONLY') : 'ctx.target';
  return directDamage(block, target, value(block, 'AMOUNT', Order.NONE, '0'), options);
};
emit['mlab_level_damage'] = (block) => directDamage(block, 'ctx.target', 'ctx.user.level', []);
emit['mlab_ohko'] = (block) =>
  directDamage(block, 'ctx.target', 'ctx.target.max_hp', [], 'if hit.hit then ctx:message("It\'s a one-hit KO!") end\n');

emit['mlab_multi_hit'] = (block) => {
  const options = pass.sure > 0 ? `, {${SURE_HIT.join(', ')}}` : '';
  return statement(block, `${keep()}ctx:multi_hit(${num(fieldNumber(block, 'POWER'))}, ${num(fieldNumber(block, 'MIN'))}, ${num(fieldNumber(block, 'MAX'))}${options})\n`);
};

emit['mlab_recoil_max_hp'] = (block) => statement(block, `ctx:recoil_max_hp(${fraction(fieldNumber(block, 'PERCENT'))})\n`);

/** An amount of HP typed as a share of max HP or as points. */
function hpAmount(block: Blockly.Block): string {
  const amount = literal(block, 'AMOUNT');
  if (field(block, 'UNIT') === 'HP') return value(block, 'AMOUNT', Order.NONE, '0');
  if (amount !== null) return `{fraction = ${fraction(amount)}}`;
  return `{fraction = ${value(block, 'AMOUNT', Order.MULTIPLICATIVE, '0')} / 100}`;
}

emit['mlab_hurt'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:hurt(${target}, ${hpAmount(block)})\n`));
};
emit['mlab_restore'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:heal(${target}, ${hpAmount(block)})\n`));
};

emit['mlab_as_type'] = (block) =>
  statement(block, `do\n${indent(`local move_type = ${value(block, 'TYPE', Order.NONE, 'ctx.move.type')}\n`)}${lua.statementToCode(block, 'DO')}end\n`);

emit['mlab_heal'] = (block) =>
  statement(
    block,
    hasOption(block, 'quiet')
      ? `ctx:heal(ctx.user, {fraction = ${fraction(fieldNumber(block, 'PERCENT'))}})\n`
      : `ctx:heal_self(${fraction(fieldNumber(block, 'PERCENT'))})\n`,
  );

emit['mlab_status'] = (block) => {
  const target = who(block);
  const options: string[] = [];
  const chance = fieldNumber(block, 'CHANCE');
  if (chance < 100) options.push(`chance = ${fraction(chance)}`);
  if (hasOption(block, 'replace')) options.push('replace = true');
  if (hasOption(block, 'announce')) options.push('announce_failure = true');
  const call = `ctx:apply_status(${target}, ctx.Status.${field(block, 'STATUS')}${options.length ? `, {${options.join(', ')}}` : ''})\n`;
  return statement(block, gate(target, call));
};

emit['mlab_cure'] = (block) => {
  const target = who(block);
  const status = field(block, 'STATUS');
  return statement(block, gate(target, `ctx:cure_status(${target}${status === 'Any' ? '' : `, ctx.Status.${status}`})\n`));
};

emit['mlab_confuse_target'] = (block) => statement(block, gate('ctx.target', 'ctx:confuse(ctx.target)\n'));
emit['mlab_bind'] = (block) => statement(block, gate('ctx.target', 'ctx:bind(ctx.target)\n'));
emit['mlab_flinch'] = (block) => statement(block, gate('ctx.target', `ctx:flinch_target(${fraction(fieldNumber(block, 'CHANCE'))})\n`));
emit['mlab_steal_boosts'] = (block) => statement(block, gate('ctx.target', 'ctx:steal_boosts(ctx.target, ctx.user)\n'));

emit['mlab_stats'] = (block) => {
  const target = who(block);
  const chance = hasOption(block, 'chance') ? fraction(fieldNumber(block, 'CHANCE')) : null;
  // The user's own stats without a chance keep the call the first script moves were written with.
  const lines =
    target === 'ctx.user' && chance === null
      ? statRows(block).map(([stat, delta]) => `ctx:change_self_stat(ctx.Stat.${stat}, ${delta})\n`)
      : statRows(block).map(([stat, delta]) => `ctx:change_stat(${target}, ctx.Stat.${stat}, ${delta})\n`);
  const changes = chance === null ? lines.join('') : `if ctx:effect_chance(${chance}) then\n${indent(lines.join(''))}end\n`;
  return statement(block, gate(target, changes));
};

emit['mlab_reset_stats'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:reset_stats(${target})\n`));
};

emit['mlab_boost_next'] = (block) =>
  statement(block, `ctx:boost_next_move(ctx.Type.${field(block, 'TYPE')}, ${num(fieldNumber(block, 'MULT'))})\n`);

emit['mlab_weather'] = (block) =>
  statement(block, `ctx:set_weather(ctx.Weather.${field(block, 'WEATHER')}, ${num(fieldNumber(block, 'TURNS'))})\n`);

emit['mlab_splash'] = (block) =>
  statement(
    block,
    'if ctx:chance(0.01) then\n' +
      indent('ctx:direct_damage(ctx.target, math.max(1, ctx.target.max_hp // 16), {typeless = true})\nctx:message("Whoa! Its splash hit with force!")\n') +
      'else\n' +
      indent('ctx:message("But nothing happened!")\n') +
      'end\n',
  );

// --- Effects and marks

function ruleTable(block: Blockly.Block): string {
  const parts: string[] = [];
  const category = () => {
    if (field(block, 'CATEGORY') !== 'Any') parts.push(`category = ctx.Category.${field(block, 'CATEGORY')}`);
    if (hasOption(block, 'type')) parts.push(`type = ctx.Type.${field(block, 'TYPE')}`);
  };
  const message = () => {
    if (hasOption(block, 'message')) parts.push(`message = ${quote(field(block, 'MESSAGE'))}`);
  };
  let kind = '';
  switch (block.type) {
    case 'mlab_rule_damage': {
      kind = 'DamageEachTurn';
      parts.push(`fraction = ${fraction(fieldNumber(block, 'PERCENT'))}`);
      const spared = optionIds(block, 'except').map((id) => `ctx.Type.${field(block, `EXCEPT${optionSuffix(id)}`)}`);
      if (spared.length) parts.push(`except_types = {${spared.join(', ')}}`);
      message();
      break;
    }
    case 'mlab_rule_heal':
      kind = 'HealEachTurn';
      parts.push(`fraction = ${fraction(fieldNumber(block, 'PERCENT'))}`);
      message();
      break;
    case 'mlab_rule_drain':
      kind = 'DrainEachTurn';
      parts.push(`fraction = ${fraction(fieldNumber(block, 'PERCENT'))}`, `to = ${who(block, 'TO')}`);
      message();
      break;
    case 'mlab_rule_damage_taken':
      kind = 'DamageTaken';
      parts.push(`factor = ${num(fieldNumber(block, 'FACTOR'))}`);
      category();
      if (hasOption(block, 'not_on_crit')) parts.push('not_on_crit = true');
      break;
    case 'mlab_rule_damage_dealt':
      kind = 'DamageDealt';
      parts.push(`factor = ${num(fieldNumber(block, 'FACTOR'))}`);
      category();
      break;
    case 'mlab_rule_stat':
      kind = 'StatMultiplier';
      parts.push(`stat = ctx.Stat.${field(block, 'STAT')}`, `factor = ${num(fieldNumber(block, 'FACTOR'))}`);
      break;
    case 'mlab_rule_block_status':
      kind = 'BlockStatus';
      if (field(block, 'STATUS') !== 'Any') parts.push(`statuses = {ctx.Status.${field(block, 'STATUS')}}`);
      if (hasOption(block, 'confusion')) parts.push('confusion = true');
      break;
    case 'mlab_rule_block_drops':
      kind = 'BlockStatDrops';
      break;
    case 'mlab_rule_effect_chance':
      kind = 'EffectChance';
      parts.push(`factor = ${num(fieldNumber(block, 'FACTOR'))}`);
      break;
    case 'mlab_rule_move_type':
      kind = 'MoveType';
      parts.push(`from = ctx.Type.${field(block, 'FROM')}`, `to = ctx.Type.${field(block, 'TO')}`);
      break;
    case 'mlab_rule_grounded':
      kind = 'Grounded';
      break;
    case 'mlab_rule_trapped':
      kind = 'Trapped';
      break;
    case 'mlab_rule_endure':
      kind = 'Endure';
      break;
    case 'mlab_rule_always_hit':
      kind = 'AlwaysHit';
      if (field(block, 'AGAINST') !== 'any') parts.push(`against = ${who(block, 'AGAINST')}`);
      break;
    case 'mlab_rule_without_type':
      kind = 'WithoutType';
      parts.push(`type = ctx.Type.${field(block, 'TYPE')}`);
      break;
  }
  return `{${[`kind = ctx.Rule.${kind}`, ...parts].join(', ')}}`;
}

/** A name typed on a block, checked the way the engine will. */
function label(block: Blockly.Block, name: string): string {
  const typed = field(block, name).trim();
  if (!typed || typed.length > 40) problem(block, 'Give it a name of 1 to 40 characters.');
  return quote(typed || 'unnamed');
}

emit['mlab_effect'] = (block) => {
  const place = scope(block);
  const parts: string[] = [];
  if (!hasOption(block, 'forever')) parts.push(`turns = ${num(fieldNumber(block, 'TURNS'))}`);
  if (hasOption(block, 'group')) parts.push(`group = ${label(block, 'GROUP')}`);
  if (hasOption(block, 'end_message')) parts.push(`end_message = ${quote(field(block, 'END_MESSAGE'))}`);
  if (hasOption(block, 'restart')) parts.push('restart = true');
  const rules: string[] = [];
  for (let rule = block.getInputTargetBlock('RULES'); rule; rule = rule.getNextBlock()) {
    if (rule.isEnabled()) rules.push(ruleTable(rule));
  }
  if (rules.length) parts.push(`rules = {\n${indent(rules.map((rule) => `${rule},\n`).join(''))}}`);
  const call = `ctx:start_effect(${place}, ${label(block, 'NAME')}${parts.length ? `, {${parts.join(', ')}}` : ''})\n`;
  return statement(block, gate(scopeGate(block), call));
};
emit['mlab_end_effect'] = (block) => statement(block, gate(scopeGate(block), `ctx:end_effect(${scope(block)}, ${label(block, 'NAME')})\n`));
emit['mlab_end_group'] = (block) => statement(block, gate(scopeGate(block), `ctx:end_group(${scope(block)}, ${label(block, 'GROUP')})\n`));
emit['mlab_mark'] = (block) => {
  const turns = hasOption(block, 'turns') ? `, ${num(fieldNumber(block, 'TURNS'))}` : '';
  return statement(block, gate(scopeGate(block), `ctx:mark(${scope(block)}, ${label(block, 'NAME')}, ${value(block, 'VALUE', Order.NONE, '1')}${turns})\n`));
};
emit['mlab_unmark'] = (block) => statement(block, gate(scopeGate(block), `ctx:unmark(${scope(block)}, ${label(block, 'NAME')})\n`));
emit['mlab_mark_value'] = (block) => [`(${scope(block)}.marks[${label(block, 'NAME')}] or 0)`, Order.ATOMIC];
emit['mlab_has_mark'] = (block) => [`${scope(block)}.marks[${label(block, 'NAME')}] ~= nil`, Order.RELATIONAL];
emit['mlab_effect_active'] = (block) => [`${scope(block)}.effects[${label(block, 'NAME')}] ~= nil`, Order.RELATIONAL];

// --- Turns

emit['mlab_force_move'] = (block) => {
  const turns = literal(block, 'TURNS');
  if (turns !== null && (turns < 2 || turns > 8 || !Number.isInteger(turns))) problem(block, 'A multi-turn move takes from 2 to 8 whole turns.');
  return statement(block, `ctx:force_move(${value(block, 'TURNS', Order.NONE, '2')}, ctx.TargetPolicy.${field(block, 'POLICY')})\n`);
};

emit['mlab_fail'] = (block) => statement(block, 'return ctx:fail()\n', '!');

emit['mlab_stop'] = (block) => {
  if (pass.kind === 'traced') return marker(`@${markOf(block)};]`) + 'return\n';
  return statement(block, 'return\n');
};

emit['mlab_hide'] = (block) => statement(block, `ctx:hide(ctx.Hidden.${field(block, 'HIDDEN')})\n`);
emit['mlab_unhide'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:unhide(${target})\n`));
};
emit['mlab_sure'] = (block) => {
  pass.sure += 1;
  const body = lua.statementToCode(block, 'DO');
  pass.sure -= 1;
  return statement(block, `do\n${body}end\n`);
};
emit['mlab_if_reaches'] = (block) => {
  const target = who(block);
  return [`ctx:reached(${target === 'ctx.target' ? '' : target})`, Order.HIGH];
};

// --- Text

emit['mlab_message'] = (block) => statement(block, `ctx:message(${messageExpr(field(block, 'TEXT'), block)})\n`);

// --- Logic

function branch(block: Blockly.Block, input: string, taken: string): string {
  const body = lua.statementToCode(block, input);
  return pass.kind === 'traced' ? lua.INDENT + marker(`@${markOf(block)}${taken}`) + body : body;
}

emit['mlab_if'] = (block) => {
  const condition = value(block, 'COND', Order.NONE, 'false');
  if (pass.kind === 'traced') {
    return `if ${condition} then\n${branch(block, 'DO', '+')}else\n${lua.INDENT}${marker(`@${markOf(block)}-`)}end\n`;
  }
  return statement(block, `if ${condition} then\n${branch(block, 'DO', '+')}end\n`);
};

emit['mlab_if_else'] = (block) => {
  const condition = value(block, 'COND', Order.NONE, 'false');
  const code = `if ${condition} then\n${branch(block, 'DO', '+')}else\n${branch(block, 'ELSE', '-')}end\n`;
  return pass.kind === 'traced' ? code : statement(block, code);
};

function repeatVariable(block: Blockly.Block): string {
  let depth = 1;
  for (let parent = block.getSurroundParent(); parent; parent = parent.getSurroundParent()) {
    if (parent.type === 'mlab_repeat') depth += 1;
  }
  return depth === 1 ? 'hit' : `hit${depth}`;
}

emit['mlab_repeat'] = (block) => {
  const times = value(block, 'TIMES', Order.NONE, '1');
  return statement(block, `for ${repeatVariable(block)} = 1, ${times} do\n${lua.statementToCode(block, 'DO')}end\n`);
};

emit['mlab_repeat_count'] = (block) => {
  for (let parent = block.getSurroundParent(); parent; parent = parent.getSurroundParent()) {
    if (parent.type === 'mlab_repeat') return [repeatVariable(parent), Order.ATOMIC];
  }
  problem(block, '“repeat number” only has a value inside a “repeat” block.');
  return ['1', Order.ATOMIC];
};

const GROUPS: Record<string, string> = {
  foes: 'ctx.foes',
  allies: 'ctx.allies',
  others: 'ctx.others',
  everyone: 'ctx.everyone',
  targets: 'ctx.targets',
};

emit['mlab_for_each'] = (block) => {
  if (loopOf(block)) problem(block, 'A “for each” block cannot go inside another one.');
  const group = field(block, 'GROUP');
  const body = lua.statementToCode(block, 'DO');
  if (group === 'beside_target') {
    // The list is read once, so that a Pokémon knocked out on the way does not shift it.
    const guarded = `if ${LOOP_VARIABLE} ~= ctx.target then\n${indent(body)}end\n`;
    return statement(block, `for _, ${LOOP_VARIABLE} in ipairs(ctx.target.team.pokemon) do\n${indent(guarded)}end\n`);
  }
  return statement(block, `for _, ${LOOP_VARIABLE} in ipairs(${GROUPS[group] ?? 'ctx.foes'}) do\n${body}end\n`);
};

emit['mlab_number'] = (block) => {
  const amount = fieldNumber(block, 'NUM');
  return [num(amount), amount < 0 ? Order.UNARY : Order.ATOMIC];
};

const ARITHMETIC: Record<string, [string, number]> = {
  ADD: ['+', Order.ADDITIVE], MINUS: ['-', Order.ADDITIVE], MULTIPLY: ['*', Order.MULTIPLICATIVE], DIVIDE: ['/', Order.MULTIPLICATIVE],
  POWER: ['^', Order.EXPONENTIATION],
};
emit['mlab_arith'] = (block) => {
  const [operator, order] = ARITHMETIC[field(block, 'OP')] ?? ARITHMETIC.ADD;
  // `^` groups from the right in Lua, so its sides are always parenthesised when they are not plain.
  const side = order === Order.EXPONENTIATION ? Order.HIGH : order;
  return [`${value(block, 'A', side, '0')} ${operator} ${value(block, 'B', side, '0')}`, order];
};

emit['mlab_min_max'] = (block) => [
  `math.${field(block, 'OP') === 'MAX' ? 'max' : 'min'}(${value(block, 'A', Order.NONE, '0')}, ${value(block, 'B', Order.NONE, '0')})`,
  Order.HIGH,
];
emit['mlab_round'] = (block) => [`math.floor(${value(block, 'A', Order.NONE, '0')})`, Order.HIGH];

const COMPARISON: Record<string, string> = { EQ: '==', NEQ: '~=', LT: '<', LTE: '<=', GT: '>', GTE: '>=' };
emit['mlab_compare'] = (block) => [
  `${value(block, 'A', Order.RELATIONAL, '0')} ${COMPARISON[field(block, 'OP')] ?? '=='} ${value(block, 'B', Order.RELATIONAL, '0')}`,
  Order.RELATIONAL,
];

emit['mlab_and_or'] = (block) => {
  const isAnd = field(block, 'OP') !== 'OR';
  const order = isAnd ? Order.AND : Order.OR;
  const fallback = isAnd ? 'true' : 'false';
  return [`${value(block, 'A', order, fallback)} ${isAnd ? 'and' : 'or'} ${value(block, 'B', order, fallback)}`, order];
};

emit['mlab_not'] = (block) => [`not ${value(block, 'A', Order.UNARY, 'true')}`, Order.UNARY];

emit['mlab_random'] = (block) => {
  const from = literal(block, 'FROM');
  const to = literal(block, 'TO');
  for (const bound of [from, to]) {
    if (bound !== null && (bound < 0 || bound > 255 || !Number.isInteger(bound))) problem(block, 'Random numbers are whole numbers from 0 to 255.');
  }
  if (from !== null && to !== null && from > to) problem(block, 'The first number must not be larger than the second.');
  return [`ctx:random_int(${value(block, 'FROM', Order.NONE, '1')}, ${value(block, 'TO', Order.NONE, '1')})`, Order.HIGH];
};

emit['mlab_chance'] = (block) => [`ctx:effect_chance(${fraction(fieldNumber(block, 'PERCENT'))})`, Order.HIGH];

// --- Battle info

emit['mlab_hp'] = (block) => {
  const fact = field(block, 'FACT') || 'hp';
  const subject = who(block);
  if (fact === 'happiness') return [`(${subject}.happiness or 0)`, Order.ATOMIC];
  if (fact === 'last_hit_damage') return [`(${subject}.last_hit and ${subject}.last_hit.damage or 0)`, Order.ATOMIC];
  return [`${subject}.${fact}`, Order.HIGH];
};
emit['mlab_turn'] = () => ['ctx.turn', Order.HIGH];
emit['mlab_last_damage'] = () => ['(result and result.damage or 0)', Order.ATOMIC];
emit['mlab_hit_landed'] = (block) => [`(result ~= nil and result.${field(block, 'RESULT') || 'hit'})`, Order.ATOMIC];
emit['mlab_first_turn'] = () => ['ctx.turn == 1', Order.RELATIONAL];
emit['mlab_last_turn'] = () => ['(ctx.total_turns ~= nil and ctx.turn == ctx.total_turns)', Order.ATOMIC];
emit['mlab_in_sequence'] = () => ['ctx.total_turns ~= nil', Order.RELATIONAL];
emit['mlab_status_is'] = (block) => {
  const status = field(block, 'STATUS');
  const negated = field(block, 'OP') === 'NOT';
  // "any status" turns the question round: having one is not being healthy.
  if (status === 'Any') return [`${who(block)}.status ${negated ? '==' : '~='} nil`, Order.RELATIONAL];
  return [`${who(block)}.status ${negated ? '~=' : '=='} ${status === 'None' ? 'nil' : `ctx.Status.${status}`}`, Order.RELATIONAL];
};
emit['mlab_weather_is'] = (block) => {
  const weather = field(block, 'WEATHER');
  const operator = field(block, 'OP') === 'NOT' ? '~=' : '==';
  return [`ctx.weather ${operator} ${weather === 'None' ? 'nil' : `Weather.${weather}`}`, Order.RELATIONAL];
};
emit['mlab_type_is'] = (block) => {
  const check = `${who(block)}:has_type(ctx.Type.${field(block, 'TYPE')})`;
  return field(block, 'OP') === 'NOT' ? [`not ${check}`, Order.UNARY] : [check, Order.HIGH];
};
emit['mlab_state_is'] = (block) => {
  const subject = who(block);
  const state = field(block, 'STATE');
  switch (state) {
    case 'acted':
    case 'confused':
    case 'protected':
    case 'trapped':
    case 'fainted':
      return [`${subject}.${state}`, Order.HIGH];
    case 'hurt':
      return [`${subject}.hurt_this_turn`, Order.HIGH];
    case 'damaging':
      return [`(${subject}.selected_move ~= nil and ${subject}.selected_move.damaging)`, Order.ATOMIC];
    case 'hidden':
    case 'item':
    case 'ability':
      return [`${subject}.${state} ~= nil`, Order.RELATIONAL];
    case 'Air':
    case 'Underground':
    case 'Underwater':
      return [`${subject}.hidden == ctx.Hidden.${state}`, Order.RELATIONAL];
    case 'Male':
    case 'Female':
    case 'Genderless':
      return [`${subject}.gender == ctx.Gender.${state}`, Order.RELATIONAL];
    default:
      return [`${subject}.side == ctx.user.side`, Order.RELATIONAL];
  }
};
emit['mlab_pair_is'] = (block) => [
  field(block, 'PAIR') === 'type' ? 'ctx:share_type(ctx.user, ctx.target)' : 'ctx:opposite_genders(ctx.user, ctx.target)',
  Order.HIGH,
];
emit['mlab_hit_by_target'] = (block) => [`ctx.user.last_attacker == ${who(block)}`, Order.RELATIONAL];
emit['mlab_text_is'] = (block) => {
  const subject = `${who(block)}.${field(block, 'WHAT')}`;
  const typed = quote(field(block, 'TEXT').trim());
  return field(block, 'OP') === 'HAS'
    ? [`(${subject} or ""):find(${typed}, 1, true) ~= nil`, Order.RELATIONAL]
    : [`${subject} == ${typed}`, Order.RELATIONAL];
};
emit['mlab_env_is'] = (block) => [`ctx.environment == ${quote(field(block, 'TEXT').trim())}`, Order.RELATIONAL];
emit['mlab_hit_was'] = (block) => {
  const kind = field(block, 'KIND');
  // A stack that reacts to a hit is handed that hit; elsewhere it is the last one of this turn.
  if (pass.hook === 'on_hit') {
    return kind === 'contact' ? ['ctx.hit.contact', Order.HIGH] : [`ctx.hit.category == ctx.Category.${kind}`, Order.RELATIONAL];
  }
  const hit = 'ctx.user.last_hit';
  return [`(${hit} ~= nil and ${kind === 'contact' ? `${hit}.contact` : `${hit}.category == ctx.Category.${kind}`})`, Order.ATOMIC];
};
emit['mlab_first_turn_out'] = () => ['ctx.user.turns_on_field <= 1', Order.RELATIONAL];
emit['mlab_used_all'] = () => ['ctx.user:used_all_other_moves(ctx.move.id)', Order.HIGH];
emit['mlab_last_failed'] = () => ['ctx.user.last_move_failed', Order.HIGH];
emit['mlab_ally_fainted'] = () => ['ctx.user.team.fainted_last_turn', Order.HIGH];
emit['mlab_count'] = (block) => [`#${GROUPS[field(block, 'GROUP')] ?? 'ctx.foes'}`, Order.UNARY];
emit['mlab_streak'] = () => ['ctx.user.streak', Order.HIGH];
emit['mlab_repeat_works'] = () => ['ctx:chance((1 / 3) ^ ctx.user.streak)', Order.HIGH];
emit['mlab_iv'] = (block) => {
  const subject = who(block);
  return [`(${subject}.ivs and ${subject}.ivs.${field(block, 'STAT')} or 31)`, Order.ATOMIC];
};
emit['mlab_stage'] = (block) => [`${who(block)}:stage(ctx.Stat.${field(block, 'STAT')})`, Order.HIGH];
emit['mlab_raised_stages'] = (block) => [`${who(block)}:raised_stages()`, Order.HIGH];
emit['mlab_type'] = (block) => [`ctx.Type.${field(block, 'TYPE')}`, Order.HIGH];
emit['mlab_type_choice'] = emit['mlab_type'];
emit['mlab_type_by_number'] = (block) => [`ctx:type_number(${value(block, 'NUMBER', Order.NONE, '0')})`, Order.HIGH];
emit['mlab_first_type'] = (block) => [`(${who(block)}.types[1] or ctx.Type.None)`, Order.ATOMIC];

// --- Items and traits

emit['mlab_steal_item'] = (block) => statement(block, gate('ctx.target', 'ctx:steal_item(ctx.target, ctx.user)\n'));
emit['mlab_take_item'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:take_item(${target})\n`));
};
emit['mlab_give_item'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:give_item(${target}, ${label(block, 'ITEM')})\n`));
};
emit['mlab_suppress_ability'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:suppress_ability(${target})\n`));
};
emit['mlab_set_type'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:set_types(${target}, {${value(block, 'TYPE', Order.NONE, 'ctx.Type.Normal')}})\n`));
};
emit['mlab_lose_type'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:lose_type(${target}, ctx.Type.${field(block, 'TYPE')})\n`));
};
emit['mlab_set_weight'] = (block) => {
  const target = who(block);
  return statement(block, gate(target, `ctx:set_weight(${target}, ${value(block, 'WEIGHT', Order.NONE, '1')})\n`));
};
emit['mlab_payout'] = (block) => statement(block, `ctx:add_payout(${value(block, 'AMOUNT', Order.NONE, '0')})\n`);

// ---------------------------------------------------------------------------

function stackOf(hat: Blockly.Block): Blockly.Block[] {
  const blocks: Blockly.Block[] = [];
  for (let block = hat.getNextBlock(); block; block = block.getNextBlock()) {
    if (block.isEnabled()) blocks.push(block);
  }
  return blocks;
}

/** Every enabled, real (non-shadow) block below a hat, nested ones included. */
function below(hat: Blockly.Block): Blockly.Block[] {
  return hat.getDescendants(true).filter((block) => block !== hat && block.isEnabled() && !block.isShadow());
}

const PLACES = ['Air', 'Underground', 'Underwater', 'Vanished'];

function header(sheet: MoveSheet, target: string, manualAnnounce: boolean): string[] {
  const lines = [
    `id = ${quote(sheet.id)},`,
    `name = ${quote(sheet.name)},`,
    `type = Type.${sheet.type},`,
    `category = Category.${sheet.category},`,
    `pp = ${sheet.pp},`,
  ];
  if (sheet.accuracy.kind === 'always') lines.push('accuracy = Accuracy.Always,');
  else if (sheet.accuracy.kind === 'ohko') lines.push('accuracy = Accuracy.Ohko,');
  else if (sheet.accuracy.percent !== 100) lines.push(`accuracy = ${num(sheet.accuracy.percent / 100)},`);
  if (target !== 'Selected') lines.push(`target = Target.${target},`);
  if (sheet.priority) lines.push(`priority = ${sheet.priority},`);
  if (sheet.failOnFullHp) lines.push('fail_on_full_hp = true,');
  if (sheet.usableWhileAsleep) lines.push('usable_while_asleep = true,');
  if (sheet.usableWhileFrozen) lines.push('usable_while_frozen = true,');
  const flags = (sheet.flags ?? []).filter((flag) => /^[a-z_]{1,20}$/.test(flag));
  if (flags.length) lines.push(`flags = {${flags.map(quote).join(', ')}},`);
  const places = (sheet.hitsHidden ?? []).filter((place) => PLACES.includes(place));
  if (places.length) lines.push(`hits_hidden = {${places.map((place) => `Hidden.${place}`).join(', ')}},`);
  if (manualAnnounce) lines.push('manual_announce = true,');
  return lines;
}

function hookFunction(name: Hook, hat: Blockly.Block): string[] {
  const inside = below(hat);
  pass.hook = name;
  pass.useResult = inside.some((block) => block.type === 'mlab_hit_landed' || block.type === 'mlab_last_damage');
  const first = hat.getNextBlock();
  let body = first ? (lua.blockToCode(first) as string) : '';
  if (pass.useResult) body = `local result\n${body}`;
  const mark = markOf(hat);
  if (pass.kind === 'traced') {
    // A stack that ends in "stop here" or "the move fails" has already closed its frame.
    const last = stackOf(hat).at(-1)?.type;
    const open = marker(`[${mark}`);
    body = last === 'mlab_stop' || last === 'mlab_fail' ? open + body : open + body + marker(']');
  }
  const head = `${name} = function(ctx)${pass.kind === 'annotated' ? ` --@${mark}` : ''}`;
  return [head, ...lua.prefixLines(body, lua.INDENT).split('\n').filter((line) => line.length), 'end,'];
}

function validateScript(hats: Partial<Record<Hook, Blockly.Block>>): void {
  for (const hat of Object.values(hats)) {
    if (!hat) continue;
    for (const block of below(hat)) {
      if (block.outputConnection) continue;
      const reason = scriptReason(block);
      if (reason) unsupported(block, reason);
    }
  }
  for (const name of ['on_hit', 'on_turn_start'] as const) {
    const hat = hats[name];
    if (!hat) continue;
    for (const block of below(hat)) {
      if (RULE_TYPES.includes(block.type)) continue;
      const allowed = block.outputConnection
        ? !REACTION_BANNED_VALUES.has(block.type)
        : REACTION_STATEMENTS.has(block.type) || (name === 'on_turn_start' && TURN_START_STATEMENTS.has(block.type));
      if (!allowed) {
        problem(
          block,
          name === 'on_hit'
            ? 'This block can’t run here. A “when the user is hit” stack reacts: it can show messages and change stats, status, HP, marks and effects, but not attack.'
            : 'This block can’t run here. An “at the start of a turn” stack gets ready: it can show messages, watch for hits and change stats, status, HP, marks and effects, but not attack.',
        );
      }
    }
  }
  const watches = Object.values(hats).flatMap((hat) => (hat ? below(hat) : [])).filter((block) => block.type === 'mlab_watch_hits');
  if (!hats.on_hit) {
    for (const block of watches) problem(block, 'Add a “when the user is hit while watching” stack to say what happens on a hit.');
  }
}

function build(kind: PassKind, workspace: Blockly.Workspace, sheet: MoveSheet, problems: Map<string, Problem>, marks: string[]) {
  pass = { kind, marks, problems, useResult: false, cause: '', hook: 'script', sure: 0 };
  const tops = workspace.getTopBlocks(true).filter((block) => block.isEnabled());
  const hats: Partial<Record<Hook, Blockly.Block>> = {
    script: tops.find((block) => block.type === 'mlab_on_use'),
    on_hit: tops.find((block) => block.type === 'mlab_on_hit'),
    on_interrupt: tops.find((block) => block.type === 'mlab_on_interrupt'),
    on_turn_start: tops.find((block) => block.type === 'mlab_on_turn_start'),
  };
  const main = hats.script;
  const target = main ? field(main, 'TARGET') || 'Selected' : 'Selected';
  const stack = main ? stackOf(main) : [];
  const effectBlocks: Generated['effectBlocks'] = [];

  if (!sheet.name.trim()) problem(null, 'Give the move a name.');

  let mode: Generated['mode'];
  const body: string[] = [];
  const forms = stack.map((block) => effectForm(block, target));
  const scripted = stack.find((_, index) => forms[index] === null);
  // A plain move's effects reach one Pokémon, so a move aimed at several takes a script too.
  const cause = hats.on_hit
    ? 'it has a “when the user is hit while watching” stack'
    : hats.on_interrupt
      ? 'it has a “when a multi-turn move is cut short” stack'
      : hats.on_turn_start
        ? 'it has an “at the start of a turn” stack'
        : MANY_TARGETS[target]
          ? `it is used on “${MANY_TARGETS[target]}”`
          : scripted
            ? `of “${shortName(scripted)}”`
            : '';
  if (!main || stack.length === 0) {
    mode = 'empty';
    problem(main ?? null, 'Add at least one block under “when this move is used”.');
    body.push('-- nothing yet: add a block under "when this move is used"');
  } else if (!cause) {
    mode = 'effects';
    body.push(`effects = {${kind === 'annotated' ? ` --@${markOf(main)}` : ''}`);
    stack.forEach((block, index) => {
      body.push(`${lua.INDENT}${forms[index]},${kind === 'annotated' ? ` --@${markOf(block)}` : ''}`);
      effectBlocks.push({ id: block.id, role: effectRole(block.type) });
      if ((block.type === 'mlab_multi_hit' || block.type === 'mlab_consecutive') && fieldNumber(block, 'MIN') > fieldNumber(block, 'MAX')) {
        problem(block, 'The first number must not be larger than the second.');
      }
      // The Protect effect protects whoever the move is used on.
      if (block.type === 'mlab_protect' && target !== 'User') {
        problem(block, 'This protects whoever the move is used on. Set “when this move is used on” to “the user”.');
      }
    });
    body.push('},');
  } else {
    mode = 'script';
    pass.cause = `This move is a script because ${cause}.`;
    validateScript(hats);
    lua.init(workspace);
    if (hats.on_interrupt) body.push(...hookFunction('on_interrupt', hats.on_interrupt));
    if (hats.on_hit) body.push(...hookFunction('on_hit', hats.on_hit));
    if (hats.on_turn_start) body.push(...hookFunction('on_turn_start', hats.on_turn_start));
    body.push(...hookFunction('script', main));
  }

  const announces = mode === 'script' && !!main && below(main).some((block) => block.type === 'mlab_announce');
  const lines = [...header(sheet, target, announces), ...body];
  const text = `return {\n${lines.map((line) => lua.INDENT + line).join('\n')}\n}\n`;
  return { text, mode, effectBlocks };
}

export interface BlockForms {
  /** What the block writes in a plain move, or null if it cannot be in one. */
  plain: string | null;
  /** What the block writes in a script, or null if it cannot be in one. */
  script: string | null;
}

/**
 * What one block writes in each shape of move file, for the block reference.
 * The plain form is the one for a move used on its own user, where every plain block is allowed.
 */
export function formsOf(block: Blockly.Block): BlockForms {
  const hook = HOOKS[block.type];
  if (hook) return { plain: hook === 'script' ? 'effects = { … }' : null, script: `${hook} = function(ctx) … end` };
  const inside = block.getDescendants(false);
  pass = {
    kind: 'annotated',
    marks: [],
    problems: new Map(),
    useResult: inside.some((other) => other.type === 'mlab_hit_landed' || other.type === 'mlab_last_damage'),
    cause: '',
    hook: 'script',
    sure: 0,
  };
  lua.init(block.workspace);
  if (RULE_TYPES.includes(block.type)) return { plain: null, script: ruleTable(block) };
  if (block.outputConnection) return { plain: null, script: (lua.blockToCode(block) as [string, number])[0] };
  const code = scriptReason(block) ? null : (lua.blockToCode(block, true) as string).replace(/ --@\d+$/gm, '').trimEnd();
  return { plain: effectForm(block, 'User'), script: code };
}

export function generate(workspace: Blockly.Workspace, sheet: MoveSheet): Generated {
  const problems = new Map<string, Problem>();
  const marks: string[] = [];
  const annotated = build('annotated', workspace, sheet, problems, marks);
  const traced = build('traced', workspace, sheet, problems, marks);
  return {
    lua: annotated.text.replace(/ --@\d+$/gm, ''),
    annotated: annotated.text,
    traced: traced.text,
    mode: annotated.mode,
    problems: [...problems.values()],
    marks,
    effectBlocks: annotated.effectBlocks,
  };
}
