// The test battle panel. It only draws what the bench reports; all rules live in the engine.
import type { BenchView, LogEntry, MonView } from '../bench/bench.ts';
import type { EngineChoice, MoveInfo, Ref } from '../engine/engine.ts';
import { STAT_SHORT, STATUS_SHORT, TYPE_COLORS } from '../model.ts';
import { clear, h, icon } from './dom.ts';

export interface Banner {
  tone: 'good' | 'warn' | 'bad';
  title: string;
  lines: string[];
  action?: { label: string; run: () => void };
}

export interface BattleModel {
  view: BenchView;
  catalog: Map<string, MoveInfo>;
  /** Id of the move open in the editor. */
  editedId: string;
  banners: Banner[];
  seed: number;
  autoRestart: boolean;
  blockColor: (id: string) => string | null;
}

export interface BattleHandlers {
  restart(): void;
  openSetup(): void;
  choose(choice: EngineChoice): void;
  newRolls(): void;
  setAutoRestart(on: boolean): void;
  focusBlock(id: string): void;
}

/** Panel-only state: the move waiting for a target. */
export interface BattleUi {
  pendingMove: string | null;
}

const WEATHER_TEXT: Record<string, string> = { Sun: 'Harsh sunlight', Sandstorm: 'Sandstorm' };

export function renderBattle(root: HTMLElement, model: BattleModel, on: BattleHandlers, ui: BattleUi, redraw: () => void): void {
  const { view } = model;
  clear(root);

  root.append(
    h('div', { class: 'bench-head' },
      h('h2', null, 'Test battle'),
      h('button', { type: 'button', class: 'btn', onclick: on.restart }, icon('restart', 14), 'Restart'),
      h('button', { type: 'button', class: 'btn', onclick: on.openSetup }, icon('sliders', 14), 'Set up'),
    ),
    ...model.banners.map(banner),
  );

  if (view.phase === 'idle') {
    root.append(h('p', { class: 'bench-empty' }, 'The battle starts as soon as the move can run.'));
    root.append(footer(model, on));
    return;
  }

  root.append(screen(model, ui), moves(model, on, ui, redraw), log(model, on), footer(model, on));
}

function banner(item: Banner): HTMLElement {
  return h('div', { class: `banner banner-${item.tone}`, role: 'status' },
    h('div', { class: 'banner-title' }, icon(item.tone === 'good' ? 'restart' : 'warning', 14), item.title),
    item.lines.map((line) => h('div', { class: 'banner-line' }, line)),
    item.action ? h('button', { type: 'button', class: 'btn btn-small', onclick: item.action.run }, item.action.label) : null,
  );
}

function screen(model: BattleModel, ui: BattleUi): HTMLElement {
  const { view } = model;
  const weather = view.weather;
  const weatherText = weather
    ? `${WEATHER_TEXT[weather.kind] ?? weather.kind} · ${weather.turns_left + 1} ${weather.turns_left === 0 ? 'turn' : 'turns'} left`
    : 'Clear skies';
  const turnText = view.phase === 'ended' ? 'Battle over' : view.phase === 'crashed' ? 'Stopped' : `Turn ${view.turn + 1}`;
  const sky = weather?.kind === 'Sun' ? 'sky-sun' : weather?.kind === 'Sandstorm' ? 'sky-sand' : 'sky-clear';

  return h('div', { class: 'screen' },
    h('div', { class: 'screen-strip' }, h('span', null, turnText), h('span', null, weatherText)),
    h('div', { class: `stage ${sky}` },
      h('div', { class: 'stage-hills' }),
      h('div', { class: 'stage-ground' }),
      h('div', { class: 'stage-cards stage-foe-cards' }, view.foes.map(card)),
      h('div', { class: 'stage-sprites stage-foe-sprites' }, view.foes.map(sprite)),
      h('div', { class: 'stage-sprites stage-ally-sprites' }, view.allies.map(sprite)),
      h('div', { class: 'stage-cards stage-ally-cards' }, view.allies.map(card)),
    ),
    h('div', { class: 'message-box' }, message(model, ui)),
  );
}

function message(model: BattleModel, ui: BattleUi): string {
  const { view } = model;
  if (view.phase === 'crashed') return 'The battle stopped.';
  if (view.phase === 'ended') {
    if (view.winner === 'allies') return 'Your side wins!';
    if (view.winner === 'foes') return 'The other side wins.';
    return 'It’s a draw.';
  }
  if (!view.prompt) return '…';
  if (ui.pendingMove) return `Use ${model.catalog.get(ui.pendingMove)?.name ?? ui.pendingMove} on…`;
  return `What will ${view.prompt.actorName} do?`;
}

function card(mon: MonView): HTMLElement {
  const part = mon.max_hp ? mon.hp / mon.max_hp : 0;
  const color = part > 0.5 ? '#2E9E5B' : part > 0.2 ? '#D99A00' : '#C5372D';
  const chips: HTMLElement[] = [];
  const chip = (text: string, tone: string) => chips.push(h('span', { class: `chip chip-${tone}` }, text));
  if (mon.status) chip(STATUS_SHORT[mon.status] ?? mon.status, 'status');
  for (const [stat, stage] of mon.stages) chip(`${STAT_SHORT[stat] ?? stat} ${stage > 0 ? '+' : '−'}${Math.abs(stage)}`, stage > 0 ? 'up' : 'down');
  if (mon.confused) chip('CONF', 'flag');
  if (mon.bound) chip('TRAP', 'flag');
  if (mon.protected) chip('PROT', 'flag');
  if (mon.hidden) chip('AWAY', 'flag');
  if (mon.recharging) chip('RCHG', 'flag');
  if (mon.locked) chip('LOCK', 'flag');
  if (mon.boost) chip(`${mon.boost.type} ×${mon.boost.multiplier}`, 'up');
  return h('div', { class: `mon-card${mon.hp === 0 ? ' fainted' : ''}` },
    h('div', { class: 'mon-name' }, h('span', null, mon.name), h('span', { class: 'mon-level' }, `Lv${mon.level}`)),
    h('div', { class: 'hp-track' }, h('div', { class: 'hp-fill', style: `width: ${Math.round(part * 100)}%; background: ${color}` })),
    h('div', { class: 'mon-foot' }, h('span', { class: 'mon-chips' }, chips.length ? chips : 'HP'), h('span', null, `${mon.hp}/${mon.max_hp}`)),
  );
}

function sprite(mon: MonView): HTMLElement {
  const color = TYPE_COLORS[mon.types[0]] ?? TYPE_COLORS.Normal;
  // Sprites come from the game; here a tinted placeholder stands in for each Pokémon.
  return h('div', { class: `sprite${mon.hp === 0 ? ' fainted' : ''}`, style: `--tint: ${color}`, title: `${mon.name} (${mon.types.join(' / ')})` },
    h('span', null, mon.name.slice(0, 1)),
  );
}

function moves(model: BattleModel, on: BattleHandlers, ui: BattleUi, redraw: () => void): HTMLElement {
  const { view } = model;
  const prompt = view.prompt;
  if (!prompt) {
    return h('div', { class: 'moves moves-empty' },
      view.phase === 'ended' || view.phase === 'crashed'
        ? h('button', { type: 'button', class: 'btn btn-primary', onclick: on.restart }, icon('restart', 14), 'Restart the battle')
        : null,
    );
  }
  const actor = (prompt.actor.side === 'allies' ? view.allies : view.foes)[prompt.actor.index];
  const pending = ui.pendingMove ? prompt.moves.find((move) => move.moveId === ui.pendingMove) : undefined;
  if (pending) {
    return h('div', { class: 'moves' },
      pending.targets.map((choice) =>
        h('button', { type: 'button', class: 'move', onclick: () => { ui.pendingMove = null; on.choose(choice); } },
          h('span', { class: 'move-name' }, name(view, choice.target)),
          h('span', { class: 'move-meta' }, choice.target.side === 'allies' ? 'Your side' : 'Other side'),
        ),
      ),
      h('button', { type: 'button', class: 'move move-cancel', onclick: () => { ui.pendingMove = null; redraw(); } }, h('span', { class: 'move-name' }, 'Back')),
    );
  }
  return h('div', { class: 'moves' },
    prompt.moves.map((move) => {
      const info = model.catalog.get(move.moveId);
      const editing = move.moveId === model.editedId;
      const left = actor?.move_pp[move.moveId];
      return h('button', {
        type: 'button',
        class: `move${editing ? ' move-editing' : ''}`,
        onclick: () => {
          if (move.targets.length > 1) { ui.pendingMove = move.moveId; redraw(); } else on.choose(move.targets[0]);
        },
      },
        h('span', { class: 'move-name' }, info?.name ?? move.moveId, editing ? h('span', { class: 'tag' }, 'editing') : null),
        h('span', { class: 'move-meta' },
          h('span', { class: 'dot', style: `background: ${TYPE_COLORS[info?.type ?? 'Normal'] ?? TYPE_COLORS.Normal}` }),
          `${info?.type ?? '?'} · PP ${left ?? '–'}/${info?.pp ?? '–'}`,
        ),
      );
    }),
  );
}

function name(view: BenchView, ref: Ref): string {
  return (ref.side === 'allies' ? view.allies : view.foes)[ref.index]?.name ?? '?';
}

function log(model: BattleModel, on: BattleHandlers): HTMLElement {
  const { view } = model;
  const turns = [...new Set(view.log.map((entry) => entry.turn))].sort((a, b) => b - a);
  return h('section', { class: 'log', 'aria-label': 'Battle log' },
    h('div', { class: 'log-head' },
      h('h3', null, 'What happened'),
      h('span', { class: 'muted' }, 'newest turn first'),
      h('span', { class: 'log-legend muted' }, h('span', { class: 'dot', style: 'background: #C5372D' }), 'from your blocks'),
    ),
    h('div', { class: 'log-list' },
      view.phase === 'crashed' ? h('div', { class: 'log-error' }, view.error ?? 'The engine stopped.') : null,
      turns.length === 0 && view.phase !== 'crashed' ? h('div', { class: 'muted log-none' }, 'Nothing yet. Pick a move above.') : null,
      turns.map((turn) => [
        h('div', { class: 'log-turn' }, `Turn ${turn}`),
        view.log.filter((entry) => entry.turn === turn).map((entry) => row(entry, model, on)),
      ]),
    ),
  );
}

function row(entry: LogEntry, model: BattleModel, on: BattleHandlers): HTMLElement {
  const color = entry.blockId ? model.blockColor(entry.blockId) : null;
  const blockId = entry.blockId;
  const content = [
    h('span', { class: 'dot', style: `background: ${color ?? 'transparent'}` }),
    entry.kind === 'message'
      ? h('span', null, entry.text)
      : [
          h('span', { class: 'log-who' }, entry.text),
          h('span', { class: `log-amount log-${entry.tone ?? 'down'}` }, entry.amount ?? ''),
          entry.tail ? h('span', { class: 'log-tail' }, entry.tail) : null,
        ],
    entry.was ? h('span', { class: 'log-was' }, `was ${entry.was}`) : null,
  ];
  if (blockId && color) {
    return h('button', { type: 'button', class: `log-row log-row-link${entry.was ? ' log-row-changed' : ''}`, title: 'Show the block that did this', onclick: () => on.focusBlock(blockId) }, content);
  }
  return h('div', { class: 'log-row' }, content);
}

function footer(model: BattleModel, on: BattleHandlers): HTMLElement {
  return h('div', { class: 'bench-foot' },
    h('span', { class: 'seed' }, `seed ${model.seed}`),
    h('button', { type: 'button', class: 'btn btn-small', onclick: on.newRolls, title: 'Restart with a different seed' }, icon('dice', 14), 'New rolls'),
    h('label', { class: 'check' },
      h('input', { type: 'checkbox', checked: model.autoRestart, onchange: (event: Event) => on.setAutoRestart((event.target as HTMLInputElement).checked) }),
      'Restart when I edit',
    ),
  );
}
