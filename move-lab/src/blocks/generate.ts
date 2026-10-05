// Turns a block workspace into a move file.
//
// The engine accepts two shapes: a plain `effects` list, or a `script` function (plus optional
// `on_hit` / `on_interrupt`). A move made only of blocks the effects list can express becomes a
// plain move; anything with logic becomes a script. Blocks the chosen shape cannot express are
// reported as problems instead of being silently dropped.
import * as Blockly from 'blockly/core';
import { LuaGenerator, Order } from 'blockly/lua';
import type { MoveSheet } from '../model.ts';
import { hasOption, optionIds, optionSuffix } from './definitions.ts';

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
type Hook = 'script' | 'on_hit' | 'on_interrupt';

interface Pass {
  kind: PassKind;
  marks: string[];
  problems: Map<string, Problem>;
  useResult: boolean;
}

let pass: Pass;

const lua = new LuaGenerator('MoveLab');
lua.INDENT = '  ';

const PLAIN_ONLY = 'This block only works in plain moves for now. It cannot share a move with Logic, Turns or Battle info blocks.';

const EFFECT_ONLY = new Set([
  'mlab_fixed_damage', 'mlab_level_damage', 'mlab_ohko', 'mlab_multi_hit', 'mlab_status', 'mlab_confuse_target',
  'mlab_bind', 'mlab_protect', 'mlab_weather', 'mlab_two_turn', 'mlab_consecutive', 'mlab_splash',
]);

const HIT_HOOK_STATEMENTS = new Set(['mlab_message', 'mlab_stats', 'mlab_if', 'mlab_if_else', 'mlab_repeat', 'mlab_stop']);

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

function messageExpr(text: string): string {
  const pieces: string[] = [];
  text.split('{user}').forEach((part, index, all) => {
    if (part) pieces.push(quote(part));
    if (index < all.length - 1) pieces.push('ctx.user.name');
  });
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

function unsupported(block: Blockly.Block, message: string): string {
  problem(block, message);
  return '';
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
  if (EFFECT_ONLY.has(block.type)) return PLAIN_ONLY;
  switch (block.type) {
    case 'mlab_damage':
      if (hasOption(block, 'recoil') || hasOption(block, 'high_crit') || hasOption(block, 'always_crit')) {
        return 'Recoil from the damage and critical-hit options only work in plain moves for now. Remove them, or remove the Logic, Turns and Battle info blocks.';
      }
      return null;
    case 'mlab_heal':
      return hasOption(block, 'quiet') ? 'Healing without a message only works in plain moves for now.' : null;
    case 'mlab_stats':
      if (field(block, 'WHO') !== 'user') return 'Changing the target’s stats only works in plain moves for now. Scripts can change the user’s stats.';
      if (hasOption(block, 'chance')) return 'A chance on stat changes only works in plain moves for now.';
      return null;
    default:
      return null;
  }
}

/** The block as an entry of the `effects` list, or null if it has no such form. */
function effectForm(block: Blockly.Block): string | null {
  const entry = (kind: string, ...fields: (string | false)[]) =>
    `{${[`kind = Effect.${kind}`, ...fields.filter((item): item is string => item !== false)].join(', ')}}`;
  switch (block.type) {
    case 'mlab_damage': {
      const power = literal(block, 'POWER');
      if (power === null) return null;
      if (hasOption(block, 'accuracy') || hasOption(block, 'min_hp') || hasOption(block, 'typeless')) return null;
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
    case 'mlab_level_damage':
      return entry('LevelDamage');
    case 'mlab_ohko':
      return entry('Ohko');
    case 'mlab_multi_hit':
      return entry('MultiHit', `power = ${num(fieldNumber(block, 'POWER'))}`, `min_hits = ${num(fieldNumber(block, 'MIN'))}`, `max_hits = ${num(fieldNumber(block, 'MAX'))}`);
    case 'mlab_heal':
      return entry('Heal', `fraction = ${fraction(fieldNumber(block, 'PERCENT'))}`, hasOption(block, 'quiet') && 'hide_message = true');
    case 'mlab_status': {
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
  if (['mlab_damage', 'mlab_fixed_damage', 'mlab_level_damage', 'mlab_ohko', 'mlab_multi_hit', 'mlab_two_turn', 'mlab_consecutive'].includes(type)) return 'damage';
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

for (const type of EFFECT_ONLY) emit[type] = (block) => unsupported(block, PLAIN_ONLY);

emit['mlab_damage'] = (block) => {
  const reason = scriptReason(block);
  if (reason) return unsupported(block, reason);
  const options: string[] = [];
  if (hasOption(block, 'accuracy')) options.push(`accuracy = ${fraction(fieldNumber(block, 'ACC'))}`);
  if (hasOption(block, 'drain')) options.push(`drain = ${fraction(fieldNumber(block, 'DRAIN'))}`);
  if (hasOption(block, 'min_hp')) options.push(`min_target_hp = ${num(fieldNumber(block, 'MIN_HP'))}`);
  if (hasOption(block, 'typeless')) options.push('typeless = true');
  const call = `ctx:damage(${value(block, 'POWER', Order.NONE, '0')}${options.length ? `, {${options.join(', ')}}` : ''})`;
  return statement(block, `${pass.useResult ? 'result = ' : ''}${call}\n`);
};

emit['mlab_recoil_max_hp'] = (block) => statement(block, `ctx:recoil_max_hp(${fraction(fieldNumber(block, 'PERCENT'))})\n`);

emit['mlab_heal'] = (block) => {
  const reason = scriptReason(block);
  if (reason) return unsupported(block, reason);
  return statement(block, `ctx:heal_self(${fraction(fieldNumber(block, 'PERCENT'))})\n`);
};

emit['mlab_flinch'] = (block) => statement(block, `ctx:flinch_target(${fraction(fieldNumber(block, 'CHANCE'))})\n`);

emit['mlab_stats'] = (block) => {
  const reason = scriptReason(block);
  if (reason) return unsupported(block, reason);
  const lines = statRows(block).map(([stat, delta]) => `ctx:change_self_stat(ctx.Stat.${stat}, ${delta})\n`);
  return statement(block, lines.join(''));
};

emit['mlab_boost_next'] = (block) =>
  statement(block, `ctx:boost_next_move(ctx.Type.${field(block, 'TYPE')}, ${num(fieldNumber(block, 'MULT'))})\n`);

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

emit['mlab_message'] = (block) => statement(block, `ctx:message(${messageExpr(field(block, 'TEXT'))})\n`);

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

emit['mlab_number'] = (block) => {
  const amount = fieldNumber(block, 'NUM');
  return [num(amount), amount < 0 ? Order.UNARY : Order.ATOMIC];
};

const ARITHMETIC: Record<string, [string, number]> = {
  ADD: ['+', Order.ADDITIVE], MINUS: ['-', Order.ADDITIVE], MULTIPLY: ['*', Order.MULTIPLICATIVE], DIVIDE: ['/', Order.MULTIPLICATIVE],
};
emit['mlab_arith'] = (block) => {
  const [operator, order] = ARITHMETIC[field(block, 'OP')] ?? ARITHMETIC.ADD;
  return [`${value(block, 'A', order, '0')} ${operator} ${value(block, 'B', order, '0')}`, order];
};

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

emit['mlab_hp'] = (block) => [`ctx.${field(block, 'WHO')}.hp`, Order.HIGH];
emit['mlab_turn'] = () => ['ctx.turn', Order.HIGH];
emit['mlab_last_damage'] = () => ['(result and result.damage or 0)', Order.ATOMIC];
emit['mlab_hit_landed'] = () => ['(result ~= nil and result.hit)', Order.ATOMIC];
emit['mlab_first_turn'] = () => ['ctx.turn == 1', Order.RELATIONAL];
emit['mlab_last_turn'] = () => ['(ctx.total_turns ~= nil and ctx.turn == ctx.total_turns)', Order.ATOMIC];
emit['mlab_in_sequence'] = () => ['ctx.total_turns ~= nil', Order.RELATIONAL];
emit['mlab_status_is'] = (block) => {
  const status = field(block, 'STATUS');
  const operator = field(block, 'OP') === 'NOT' ? '~=' : '==';
  return [`ctx.${field(block, 'WHO')}.status ${operator} ${status === 'None' ? 'nil' : `ctx.Status.${status}`}`, Order.RELATIONAL];
};
emit['mlab_weather_is'] = (block) => {
  const weather = field(block, 'WEATHER');
  const operator = field(block, 'OP') === 'NOT' ? '~=' : '==';
  return [`ctx.weather ${operator} ${weather === 'None' ? 'nil' : `Weather.${weather}`}`, Order.RELATIONAL];
};

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
  if (manualAnnounce) lines.push('manual_announce = true,');
  return lines;
}

function hookFunction(name: Hook, hat: Blockly.Block): string[] {
  const inside = below(hat);
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
      if (reason) problem(block, reason);
    }
  }
  const hit = hats.on_hit;
  if (hit) {
    for (const block of below(hit)) {
      const allowed = block.outputConnection ? block.type !== 'mlab_random' : HIT_HOOK_STATEMENTS.has(block.type);
      if (!allowed) problem(block, 'This block can’t run here yet. A “when the user is hit” stack can show messages and change the user’s stats.');
    }
  }
  if (hats.script && !hit) {
    for (const block of below(hats.script)) {
      if (block.type === 'mlab_watch_hits') problem(block, 'Add a “when the user is hit while watching” stack to say what happens on a hit.');
    }
  }
}

function build(kind: PassKind, workspace: Blockly.Workspace, sheet: MoveSheet, problems: Map<string, Problem>, marks: string[]) {
  pass = { kind, marks, problems, useResult: false };
  const tops = workspace.getTopBlocks(true).filter((block) => block.isEnabled());
  const hats: Partial<Record<Hook, Blockly.Block>> = {
    script: tops.find((block) => block.type === 'mlab_on_use'),
    on_hit: tops.find((block) => block.type === 'mlab_on_hit'),
    on_interrupt: tops.find((block) => block.type === 'mlab_on_interrupt'),
  };
  const main = hats.script;
  const target = main ? field(main, 'TARGET') || 'Selected' : 'Selected';
  const stack = main ? stackOf(main) : [];
  const effectBlocks: Generated['effectBlocks'] = [];

  if (!sheet.name.trim()) problem(null, 'Give the move a name.');

  let mode: Generated['mode'];
  const body: string[] = [];
  const forms = stack.map(effectForm);
  if (!main || stack.length === 0) {
    mode = 'empty';
    problem(main ?? null, 'Add at least one block under “when this move is used”.');
    body.push('-- nothing yet: add a block under "when this move is used"');
  } else if (!hats.on_hit && !hats.on_interrupt && forms.every((form) => form !== null)) {
    mode = 'effects';
    body.push(`effects = {${kind === 'annotated' ? ` --@${markOf(main)}` : ''}`);
    stack.forEach((block, index) => {
      body.push(`${lua.INDENT}${forms[index]},${kind === 'annotated' ? ` --@${markOf(block)}` : ''}`);
      effectBlocks.push({ id: block.id, role: effectRole(block.type) });
      if ((block.type === 'mlab_multi_hit' || block.type === 'mlab_consecutive') && fieldNumber(block, 'MIN') > fieldNumber(block, 'MAX')) {
        problem(block, 'The first number must not be larger than the second.');
      }
    });
    body.push('},');
  } else {
    mode = 'script';
    validateScript(hats);
    lua.init(workspace);
    if (hats.on_interrupt) body.push(...hookFunction('on_interrupt', hats.on_interrupt));
    if (hats.on_hit) body.push(...hookFunction('on_hit', hats.on_hit));
    body.push(...hookFunction('script', main));
  }

  const announces = mode === 'script' && !!main && below(main).some((block) => block.type === 'mlab_announce');
  const lines = [...header(sheet, target, announces), ...body];
  const text = `return {\n${lines.map((line) => lua.INDENT + line).join('\n')}\n}\n`;
  return { text, mode, effectBlocks };
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
