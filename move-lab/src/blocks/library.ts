// Builds block workspaces without the editor: built-in moves opened as blocks, and starting points.
import type { EffectInfo, MoveInfo } from '../engine/engine.ts';
import type { Category, MoveSheet } from '../model.ts';

export interface BlockState {
  type: string;
  x?: number;
  y?: number;
  deletable?: boolean;
  fields?: Record<string, unknown>;
  inputs?: Record<string, { block?: BlockState; shadow?: BlockState }>;
  next?: { block: BlockState };
  extraState?: unknown;
}

export interface WorkspaceState {
  blocks: { languageVersion: number; blocks: BlockState[] };
}

type Input = { block?: BlockState; shadow?: BlockState };

const shadow = (value: number): BlockState => ({ type: 'mlab_number', fields: { NUM: value } });
const num = (value: number): Input => ({ shadow: shadow(value) });
const plug = (block: BlockState, fallback = 0): Input => ({ shadow: shadow(fallback), block });
const socket = (block: BlockState): Input => ({ block });

function make(type: string, fields?: Record<string, unknown>, inputs?: BlockState['inputs'], options?: string[]): BlockState {
  const state: BlockState = { type };
  if (fields && Object.keys(fields).length) state.fields = fields;
  if (inputs) state.inputs = inputs;
  if (options?.length) state.extraState = { options };
  return state;
}

function chain(blocks: BlockState[]): BlockState | undefined {
  for (let index = blocks.length - 2; index >= 0; index -= 1) blocks[index].next = { block: blocks[index + 1] };
  return blocks[0];
}

const body = (blocks: BlockState[]): Input => ({ block: chain(blocks) });

/** Rough on-screen height of a stack, to place a second stack below the first. */
function height(block: BlockState | undefined): number {
  let total = 0;
  for (let current = block; current; current = current.next?.block) {
    total += 56;
    const options = (current.extraState as { options?: string[] } | undefined)?.options?.length ?? 0;
    total += options * 36;
    for (const name of ['DO', 'ELSE']) {
      if (current.inputs?.[name]) total += height(current.inputs[name].block) + 34;
    }
  }
  return total;
}

export function workspaceState(target: string, main: BlockState[], hooks: { hit?: BlockState[]; interrupt?: BlockState[] } = {}): WorkspaceState {
  const tops: BlockState[] = [];
  let y = 28;
  const place = (type: string, stack: BlockState[], extra: Partial<BlockState>) => {
    const first = chain(stack);
    const hat: BlockState = { type, x: 28, y, ...extra };
    if (first) hat.next = { block: first };
    tops.push(hat);
    y += 64 + height(first) + 90;
  };
  place('mlab_on_use', main, { deletable: false, fields: { TARGET: target } });
  if (hooks.hit) place('mlab_on_hit', hooks.hit, {});
  if (hooks.interrupt) place('mlab_on_interrupt', hooks.interrupt, {});
  return { blocks: { languageVersion: 0, blocks: tops } };
}

// ---------------------------------------------------------------------------
// Block shorthands

type DamageOptions = { drain?: number; min_hp?: number; accuracy?: number; typeless?: boolean; recoil?: number; high_crit?: boolean; always_crit?: boolean };

function damage(power: number | BlockState, options: DamageOptions = {}): BlockState {
  const fields: Record<string, unknown> = {};
  const rows: string[] = [];
  if (options.drain !== undefined) (rows.push('drain'), (fields.DRAIN = options.drain));
  if (options.min_hp !== undefined) (rows.push('min_hp'), (fields.MIN_HP = options.min_hp));
  if (options.accuracy !== undefined) (rows.push('accuracy'), (fields.ACC = options.accuracy));
  if (options.typeless) rows.push('typeless');
  if (options.recoil !== undefined) (rows.push('recoil'), (fields.RECOIL = options.recoil));
  if (options.high_crit) rows.push('high_crit');
  if (options.always_crit) rows.push('always_crit');
  return make('mlab_damage', fields, { POWER: typeof power === 'number' ? num(power) : plug(power, 40) }, rows);
}

function stats(who: 'user' | 'target', rows: [string, number][], chance?: number): BlockState {
  const fields: Record<string, unknown> = { WHO: who, STAT: rows[0][0], STAGES: String(rows[0][1]) };
  const options: string[] = [];
  rows.slice(1).forEach(([stat, delta], index) => {
    options.push(`stat#${index + 1}`);
    fields[`STAT_${index + 1}`] = stat;
    fields[`STAGES_${index + 1}`] = String(delta);
  });
  if (chance !== undefined) {
    options.push('chance');
    fields.CHANCE = chance;
  }
  return make('mlab_stats', fields, undefined, options);
}

const heal = (percent: number, quiet = false) => make('mlab_heal', { PERCENT: percent }, undefined, quiet ? ['quiet'] : []);
const statusIs = (who: 'user' | 'target', op: 'IS' | 'NOT', status: string) => make('mlab_status_is', { WHO: who, OP: op, STATUS: status });
const weatherIs = (op: 'IS' | 'NOT', weather: string) => make('mlab_weather_is', { OP: op, WEATHER: weather });
const not = (inner: BlockState) => make('mlab_not', undefined, { A: socket(inner) });
const both = (a: BlockState, b: BlockState) => make('mlab_and_or', { OP: 'AND' }, { A: socket(a), B: socket(b) });
const either = (a: BlockState, b: BlockState) => make('mlab_and_or', { OP: 'OR' }, { A: socket(a), B: socket(b) });
const when = (condition: BlockState, then: BlockState[]) => make('mlab_if', undefined, { COND: socket(condition), DO: body(then) });
const whenElse = (condition: BlockState, then: BlockState[], otherwise: BlockState[]) =>
  make('mlab_if_else', undefined, { COND: socket(condition), DO: body(then), ELSE: body(otherwise) });
const forceMove = (turns: number | BlockState, policy: string) =>
  make('mlab_force_move', { POLICY: policy }, { TURNS: typeof turns === 'number' ? num(turns) : plug(turns, 2) });

/** Engine fractions arrive as 32-bit floats; show them as tidy percentages. */
const percent = (fraction: unknown) => Math.round(Number(fraction) * 10000) / 100;

// ---------------------------------------------------------------------------
// Built-in moves

function effectBlock(effect: EffectInfo): BlockState {
  switch (effect.kind) {
    case 'Damage':
      return damage(Number(effect.power), {
        recoil: effect.recoil === null ? undefined : percent(effect.recoil),
        drain: effect.drain === null ? undefined : percent(effect.drain),
        high_crit: effect.high_crit === true,
        always_crit: effect.always_crit === true,
      });
    case 'MultiHit':
      return make('mlab_multi_hit', { MIN: effect.min_hits, MAX: effect.max_hits, POWER: effect.power });
    case 'FixedDamage':
      return make('mlab_fixed_damage', { AMOUNT: effect.amount });
    case 'LevelDamage':
      return make('mlab_level_damage');
    case 'Ohko':
      return make('mlab_ohko');
    case 'Stats': {
      const chance = Number(effect.chance);
      return stats(effect.self ? 'user' : 'target', effect.stages as [string, number][], chance < 1 ? percent(chance) : undefined);
    }
    case 'Status':
      return make('mlab_status', { STATUS: effect.status, CHANCE: percent(effect.chance) }, undefined, effect.replace ? ['replace'] : []);
    case 'Confuse':
      return make('mlab_confuse_target');
    case 'Flinch':
      return make('mlab_flinch', { CHANCE: percent(effect.chance) });
    case 'Bind':
      return make('mlab_bind');
    case 'Protect':
      return make('mlab_protect');
    case 'Heal':
      return heal(percent(effect.fraction), effect.hide_message === true);
    case 'Weather':
      return make('mlab_weather', { WEATHER: effect.weather, TURNS: effect.turns });
    case 'TwoTurn':
      return make(
        'mlab_two_turn',
        { MESSAGE: String(effect.charge_message).split('{0}').join('{user}'), POWER: effect.power },
        undefined,
        [effect.semi_invulnerable ? 'hidden' : '', effect.skip_in_sun ? 'skip_sun' : ''].filter(Boolean),
      );
    case 'Consecutive':
      return make(
        'mlab_consecutive',
        { MIN: effect.min_turns, MAX: effect.max_turns, POWER: effect.power },
        undefined,
        [effect.confuse_after ? 'confuse' : '', effect.double_power ? 'double' : ''].filter(Boolean),
      );
    default:
      return make('mlab_splash');
  }
}

interface Authored {
  main: BlockState[];
  hit?: BlockState[];
  interrupt?: BlockState[];
  sheet?: Partial<MoveSheet>;
}

/** The engine cannot describe a script, so the scripted built-ins are rebuilt by hand, block for line. */
const SCRIPTED: Record<string, () => Authored> = {
  charge: () => ({ main: [stats('user', [['SpDefense', 1]]), make('mlab_boost_next', { TYPE: 'Electric', MULT: 2 })] }),
  dream_eater: () => ({ main: [when(statusIs('target', 'NOT', 'Asleep'), [make('mlab_fail')]), damage(100, { drain: 50 })] }),
  explosion: () => ({ main: [make('mlab_faint_user'), damage(250)] }),
  facade: () => ({
    main: [
      whenElse(
        either(statusIs('user', 'IS', 'Burned'), either(statusIs('user', 'IS', 'Paralyzed'), either(statusIs('user', 'IS', 'Poisoned'), statusIs('user', 'IS', 'BadlyPoisoned')))),
        [damage(140)],
        [damage(70)],
      ),
    ],
  }),
  false_swipe: () => ({ main: [damage(40, { min_hp: 1 })] }),
  hyper_beam: () => ({ main: [damage(150), when(make('mlab_hit_landed'), [make('mlab_recharge')])] }),
  morning_sun: () => ({
    main: [whenElse(weatherIs('IS', 'Sun'), [heal(66.67)], [whenElse(weatherIs('IS', 'Sandstorm'), [heal(25)], [heal(50)])])],
  }),
  rage: () => ({ main: [make('mlab_watch_hits'), damage(20)], hit: [stats('user', [['Attack', 1]])] }),
  snore: () => ({
    main: [
      when(statusIs('user', 'NOT', 'Asleep'), [make('mlab_fail')]),
      damage(50),
      when(make('mlab_hit_landed'), [make('mlab_flinch', { CHANCE: 30 })]),
    ],
    sheet: { usableWhileAsleep: true },
  }),
  solar_beam: () => ({
    main: [
      when(both(make('mlab_first_turn'), weatherIs('NOT', 'Sun')), [
        make('mlab_message', { TEXT: '{user} took in sunlight!' }),
        forceMove(2, 'SameTarget'),
        make('mlab_stop'),
      ]),
      make('mlab_announce'),
      damage(120),
    ],
  }),
  thrash: () => ({
    main: [
      when(not(make('mlab_in_sequence')), [
        forceMove(make('mlab_random', undefined, { FROM: num(2), TO: num(3) }), 'RandomOpponent'),
      ]),
      damage(120),
      when(not(make('mlab_hit_landed')), [
        when(make('mlab_last_turn'), [make('mlab_confuse_self')]),
        make('mlab_break_sequence'),
        make('mlab_stop'),
      ]),
      when(make('mlab_last_turn'), [make('mlab_confuse_self')]),
    ],
    interrupt: [when(make('mlab_last_turn'), [make('mlab_confuse_self')]), make('mlab_break_sequence')],
  }),
  triple_kick: () => ({
    main: [
      make('mlab_repeat', undefined, {
        TIMES: num(3),
        DO: body([
          damage(make('mlab_arith', { OP: 'MULTIPLY' }, { A: num(10), B: plug(make('mlab_repeat_count'), 1) }), { accuracy: 90 }),
          when(not(make('mlab_hit_landed')), [make('mlab_stop')]),
        ]),
      }),
    ],
  }),
};

export function sheetFromInfo(info: MoveInfo): MoveSheet {
  return {
    id: info.id,
    name: info.name,
    type: info.type,
    category: info.category as Category,
    pp: info.pp,
    accuracy: info.accuracy.kind === 'chance' ? { kind: 'chance', percent: percent(info.accuracy.value) } : { kind: info.accuracy.kind },
    priority: info.priority,
    failOnFullHp: info.fail_on_full_hp,
    usableWhileAsleep: false,
    ...SCRIPTED[info.id]?.().sheet,
  };
}

/** Whether a move from the catalog can be opened as blocks. */
export function canOpen(info: MoveInfo): boolean {
  return !info.scripted || info.id in SCRIPTED;
}

export function blocksFromInfo(info: MoveInfo): WorkspaceState {
  if (!info.scripted) return workspaceState(info.target, info.effects.map(effectBlock));
  const authored = SCRIPTED[info.id]?.();
  if (!authored) return workspaceState(info.target, []);
  return workspaceState(info.target, authored.main, { hit: authored.hit, interrupt: authored.interrupt });
}

export function newMoveSheet(name: string, id: string): MoveSheet {
  return {
    id,
    name,
    type: 'Normal',
    category: 'Physical',
    pp: 20,
    accuracy: { kind: 'chance', percent: 100 },
    priority: 0,
    failOnFullHp: false,
    usableWhileAsleep: false,
  };
}

export function newMoveBlocks(): WorkspaceState {
  return workspaceState('Selected', [damage(40)]);
}
