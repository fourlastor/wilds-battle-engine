// Runs a test battle in the engine and turns its events into what the battle panel shows:
// the combatants, a turn-by-turn log, and which blocks of the edited move produced what.
import type { EffectRole } from '../blocks/generate.ts';
import type { Advance, Battle, Engine, EngineChoice, EngineEvent, EngineMon, EngineSetup, EngineState, Ref, Side } from '../engine/engine.ts';
import type { FoePolicy } from '../model.ts';

/** How to recognise the edited move's blocks in the event stream. */
export interface TraceSource {
  moveName: string;
  mode: 'effects' | 'script' | 'empty';
  /** Marker number → block id (scripts). */
  marks: string[];
  /** Blocks of a plain move, matched by the kind of event they cause. */
  effectBlocks: { id: string; role: EffectRole }[];
}

/** A choice made in the battle, remembered so the battle can be replayed after an edit. */
export interface Pick {
  actor: Ref;
  moveId: string;
  target: Ref;
}

export interface LogEntry {
  turn: number;
  kind: 'message' | 'damage' | 'heal' | 'stat';
  text: string;
  amount?: string;
  tone?: 'down' | 'up';
  tail?: string;
  blockId: string | null;
  /** After an edit: what this line said before. */
  was?: string;
}

export interface MonView extends EngineMon {
  side: Side;
  index: number;
}

export interface PromptView {
  id: number;
  actor: Ref;
  actorName: string;
  moves: { moveId: string; targets: EngineChoice[] }[];
}

export interface Summary {
  hp: Record<string, { name: string; hp: number }>;
  marked: { turn: number; amount: string }[];
}

export interface BenchView {
  phase: 'idle' | 'awaiting' | 'ended' | 'crashed';
  turn: number;
  weather: EngineState['weather'];
  allies: MonView[];
  foes: MonView[];
  /** Marks and effects on everyone and on each side, and the money each side has earned. */
  field: EngineState['field'];
  sides: EngineState['sides'];
  payout: EngineState['payout'];
  prompt: PromptView | null;
  winner: Side | null;
  log: LogEntry[];
  /** Blocks of the edited move that ran in the latest turn it did anything, and which turn that was. */
  ran: string[];
  ranTurn: number | null;
  /** For each "if" block that ran: whether its condition held. */
  branches: Record<string, boolean>;
  error: string | null;
  picks: Pick[];
  /** After a replay: how many remembered choices were repeated, and what changed. */
  replayed: number;
  replayCut: boolean;
  changes: string[];
}

export interface StartOptions {
  setup: EngineSetup;
  policy: FoePolicy;
  trace: TraceSource | null;
  /** Choices to repeat, in order. */
  replay?: Pick[];
  /** The previous run, to report what an edit changed. */
  compareWith?: Summary | null;
}

const MARK = '\u0001';
const END_OF_TURN = /is hurt by|is buffeted by|^The sunlight|^The sandstorm|^The harsh sunlight|^Rain continues|^Hail continues|^The rain|^The hail|was freed from/;
const STAT_NAMES: Record<string, string> = {
  Attack: 'Attack', Defense: 'Defense', SpAttack: 'Sp. Atk', SpDefense: 'Sp. Def', Speed: 'Speed', Accuracy: 'Accuracy', Evasion: 'Evasion',
};

function emptyView(): BenchView {
  return {
    phase: 'idle', turn: 0, weather: null, allies: [], foes: [], field: [], sides: { allies: [], foes: [] }, payout: { allies: 0, foes: 0 },
    prompt: null, winner: null, log: [],
    ran: [], ranTurn: null, branches: {}, error: null, picks: [], replayed: 0, replayCut: false, changes: [],
  };
}

function mulberry(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let value = state;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}

const sameRef = (a: Ref, b: Ref) => a.side === b.side && a.index === b.index;

export class Bench {
  view: BenchView = emptyView();
  private readonly engine: Engine;
  private battle: Battle | null = null;
  private trace: TraceSource | null = null;
  private policy: FoePolicy = 'first';
  private random = mulberry(1);
  private queue: Pick[] = [];
  private state: EngineState | null = null;

  // Trace decoding (see generate.ts for what the markers mean).
  private frames: (string | null)[] = [];
  private current: string | null = null;
  private oneShot = false;
  private window = false;
  private windowBlock: string | null = null;

  constructor(engine: Engine) {
    this.engine = engine;
  }

  /** Starts a battle with the move files currently loaded in the engine. Throws if the setup is rejected. */
  start(options: StartOptions): void {
    const battle = this.engine.createBattle(options.setup);
    this.battle?.dispose();
    this.battle = battle;
    this.trace = options.trace;
    this.policy = options.policy;
    this.random = mulberry(options.setup.seed);
    this.queue = [...(options.replay ?? [])];
    this.view = emptyView();
    this.view.replayCut = false;
    this.pump();
    this.view.replayed = (options.replay?.length ?? 0) - this.queue.length;
    this.view.replayCut = this.queue.length > 0;
    this.queue = [];
    if (options.compareWith) this.compare(options.compareWith);
  }

  choose(choice: EngineChoice): void {
    const prompt = this.view.prompt;
    if (!this.battle || !prompt || this.view.phase !== 'awaiting') return;
    this.view.picks.push({ actor: prompt.actor, moveId: choice.move_id, target: choice.target });
    this.view.changes = [];
    try {
      this.battle.respond(prompt.id, choice.id);
    } catch (error) {
      this.crash(error);
      return;
    }
    this.pump();
  }

  summary(): Summary {
    const hp: Summary['hp'] = {};
    for (const mon of [...this.view.allies, ...this.view.foes]) hp[`${mon.side}:${mon.index}`] = { name: mon.name, hp: mon.hp };
    const marked = this.view.log.filter((entry) => entry.blockId && entry.amount).map((entry) => ({ turn: entry.turn, amount: entry.amount! }));
    return { hp, marked };
  }

  dispose(): void {
    this.battle?.dispose();
    this.battle = null;
  }

  private pump(): void {
    const battle = this.battle;
    if (!battle) return;
    try {
      for (let guard = 0; guard < 400; guard += 1) {
        const result = battle.advance();
        const state = battle.state();
        this.consume(result, state);
        if (result.status.kind === 'end') {
          this.view.phase = 'ended';
          this.view.winner = result.status.winner;
          this.view.prompt = null;
          return;
        }
        const prompt = result.status;
        const automatic = this.automatic(prompt.actor, prompt.choices);
        if (automatic) {
          battle.respond(prompt.prompt_id, automatic.id);
          continue;
        }
        this.view.phase = 'awaiting';
        this.view.prompt = this.promptView(prompt.prompt_id, prompt.actor, prompt.choices, state);
        return;
      }
      throw new Error('The battle did not reach a choice after 400 steps.');
    } catch (error) {
      this.crash(error);
    }
  }

  private crash(error: unknown): void {
    this.view.phase = 'crashed';
    this.view.prompt = null;
    this.view.error = error instanceof Error ? error.message : String(error);
  }

  /** The choice to make without asking: the other side's policy, or the next remembered pick. */
  private automatic(actor: Ref, choices: EngineChoice[]): EngineChoice | null {
    if (actor.side === 'foes' && this.policy !== 'manual') {
      return this.policy === 'random' ? choices[Math.floor(this.random() * choices.length)] : choices[0];
    }
    const next = this.queue[0];
    if (!next) return null;
    const match =
      choices.find((choice) => choice.move_id === next.moveId && sameRef(choice.target, next.target)) ??
      choices.find((choice) => choice.move_id === next.moveId);
    if (!match || !sameRef(next.actor, actor)) return null;
    this.queue.shift();
    this.view.picks.push({ actor, moveId: match.move_id, target: match.target });
    return match;
  }

  private promptView(id: number, actor: Ref, choices: EngineChoice[], state: EngineState): PromptView {
    const moves: PromptView['moves'] = [];
    for (const choice of choices) {
      const group = moves.find((move) => move.moveId === choice.move_id);
      if (group) group.targets.push(choice);
      else moves.push({ moveId: choice.move_id, targets: [choice] });
    }
    return { id, actor, actorName: state[actor.side][actor.index]?.name ?? '?', moves };
  }

  private consume(result: Advance, state: EngineState): void {
    this.state = state;
    const view = this.view;
    view.turn = state.turn;
    view.weather = state.weather;
    view.allies = state.allies.map((mon, index) => ({ ...mon, side: 'allies' as const, index }));
    view.foes = state.foes.map((mon, index) => ({ ...mon, side: 'foes' as const, index }));
    view.field = state.field;
    view.sides = state.sides;
    view.payout = state.payout;
    // Each batch belongs to the turn that just resolved; nothing carries over between batches.
    this.frames = [];
    this.current = null;
    this.oneShot = false;
    this.window = false;
    this.windowBlock = null;
    for (const event of result.events) this.event(event, state.turn);
  }

  private event(event: EngineEvent, turn: number): void {
    if (event.kind === 'message' && event.text.startsWith(MARK)) {
      for (const token of event.text.slice(1).split(';')) this.token(token, turn);
      return;
    }
    const entry = this.entry(event, turn);
    if (!entry) return;
    entry.blockId = this.trace?.mode === 'effects' ? this.guess(event) : this.current;
    this.view.log.push(entry);
    if (this.oneShot) {
      this.oneShot = false;
      this.current = this.frames.pop() ?? null;
    }
  }

  /** `[n` opens a stack, `]` closes it, `@n` is a block, with `+`/`-` for an "if" and `!` for a block that ends the script. */
  private token(token: string, turn: number): void {
    const view = this.view;
    const marks = this.trace?.marks ?? [];
    if (token.startsWith('[')) {
      const hat = marks[Number(token.slice(1))];
      this.startTrace(turn);
      this.frames.push(this.current);
      this.current = null;
      if (hat && !view.ran.includes(hat)) view.ran.push(hat);
    } else if (token === ']') {
      this.current = this.frames.pop() ?? null;
    } else if (token.startsWith('@')) {
      const suffix = /[+\-!]$/.test(token) ? token.slice(-1) : '';
      const id = marks[Number(suffix ? token.slice(1, -1) : token.slice(1))];
      if (!id) return;
      this.current = id;
      if (!view.ran.includes(id)) view.ran.push(id);
      if (suffix === '+') view.branches[id] = true;
      if (suffix === '-') view.branches[id] = false;
      if (suffix === '!') this.oneShot = true;
    }
  }

  /** The glow shows one turn at a time: the latest turn in which the move did anything. */
  private startTrace(turn: number): void {
    if (this.view.ranTurn === turn) return;
    this.view.ran = [];
    this.view.branches = {};
    this.view.ranTurn = turn;
  }

  /** Plain moves carry no markers: match what happened after "… used <move>!" by the kind of event. */
  private guess(event: EngineEvent): string | null {
    const trace = this.trace;
    if (!trace) return null;
    if (event.kind === 'message') {
      const used = /^.* used (.+)!$/.exec(event.text);
      if (used) {
        this.window = used[1] === trace.moveName;
        this.windowBlock = null;
        if (this.window) this.startTrace(this.state?.turn ?? 0);
        return null;
      }
      if (END_OF_TURN.test(event.text)) this.window = false;
      return this.window ? this.windowBlock : null;
    }
    if (!this.window) return null;
    const role: EffectRole | null =
      event.kind === 'damage' ? 'damage' : event.kind === 'heal' ? 'heal' : event.kind === 'status' ? 'status'
        : event.kind === 'stat_change' ? 'stats' : event.kind === 'weather' ? 'weather' : null;
    const block = role ? trace.effectBlocks.find((candidate) => candidate.role === role) : undefined;
    if (!block) return this.windowBlock;
    this.windowBlock = block.id;
    if (!this.view.ran.includes(block.id)) this.view.ran.push(block.id);
    return block.id;
  }

  private entry(event: EngineEvent, turn: number): LogEntry | null {
    const mon = (ref: Ref) => this.state?.[ref.side][ref.index];
    switch (event.kind) {
      case 'message':
        return { turn, kind: 'message', text: event.text, blockId: null };
      case 'damage': {
        const target = mon(event.target);
        return { turn, kind: 'damage', text: target?.name ?? '?', amount: `−${event.amount}`, tone: 'down', tail: `${event.hp}/${target?.max_hp ?? '?'} HP`, blockId: null };
      }
      case 'heal': {
        const target = mon(event.target);
        return { turn, kind: 'heal', text: target?.name ?? '?', amount: `+${event.amount}`, tone: 'up', tail: `${event.hp}/${target?.max_hp ?? '?'} HP`, blockId: null };
      }
      case 'stat_change': {
        const target = mon(event.target);
        const up = event.stages > 0;
        return { turn, kind: 'stat', text: target?.name ?? '?', amount: `${STAT_NAMES[event.stat] ?? event.stat} ${up ? '+' : '−'}${Math.abs(event.stages)}`, tone: up ? 'up' : 'down', blockId: null };
      }
      default:
        // Status, weather, faint and end events are already told by the engine's messages.
        if (this.trace?.mode === 'effects') this.guess(event);
        return null;
    }
  }

  private compare(before: Summary): void {
    const after = this.summary();
    const changes: string[] = [];
    for (const [key, now] of Object.entries(after.hp)) {
      const was = before.hp[key];
      if (was && was.hp !== now.hp) changes.push(`${now.name}: ${was.hp} → ${now.hp} HP`);
    }
    const marked = this.view.log.filter((entry) => entry.blockId && entry.amount);
    marked.forEach((entry, index) => {
      const old = before.marked[index];
      if (old && old.turn === entry.turn && old.amount !== entry.amount) entry.was = old.amount;
    });
    this.view.changes = changes;
  }
}
