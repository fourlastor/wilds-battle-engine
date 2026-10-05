// The "Set up the test battle" dialog. It edits a copy; nothing changes until "Start battle".
import { SPECIES, findSpecies, monFor, statsFor } from '../data/species.ts';
import type { MoveInfo } from '../engine/engine.ts';
import { STATUSES, STATUS_LABELS, TYPES, TYPE_COLORS, clamp } from '../model.ts';
import type { BattleSetup, FoePolicy, MonSetup, OnEdit, Stats } from '../model.ts';
import { clear, h, icon } from './dom.ts';

export interface SetupOptions {
  setup: BattleSetup;
  catalog: MoveInfo[];
  editedId: string;
  editedName: string;
  onApply(setup: BattleSetup): void;
}

const MAX_PER_SIDE = 3;
const STAT_FIELDS: [keyof Stats, string][] = [
  ['max_hp', 'Max HP'], ['attack', 'Attack'], ['defense', 'Defense'], ['sp_attack', 'Sp. Atk'], ['sp_defense', 'Sp. Def'], ['speed', 'Speed'],
];

export function openSetup(options: SetupOptions): void {
  const draft: BattleSetup = structuredClone(options.setup);
  const moves = options.catalog.filter((move) => move.id !== 'struggle').sort((a, b) => a.name.localeCompare(b.name));
  const dialog = h('dialog', { class: 'setup', 'aria-labelledby': 'setup-title' });
  const body = h('div', { class: 'setup-body' });

  const close = () => {
    dialog.close();
    dialog.remove();
  };
  const apply = () => {
    options.onApply(draft);
    close();
  };

  const numberInput = (value: number, min: number, max: number, set: (value: number) => void, extra: Record<string, unknown> = {}) =>
    h('input', {
      type: 'number', class: 'in', value: String(value), min: String(min), max: String(max), ...extra,
      onchange: (event: Event) => {
        const input = event.target as HTMLInputElement;
        const next = clamp(Math.round(Number(input.value) || min), min, max);
        input.value = String(next);
        set(next);
      },
    });

  const select = (items: [string, string][], value: string, set: (value: string) => void, label: string) =>
    h('select', { class: 'in', 'aria-label': label, value, onchange: (event: Event) => set((event.target as HTMLSelectElement).value) },
      items.map(([id, text]) => h('option', { value: id }, text)),
    );

  function monCard(side: 'allies' | 'foes', index: number): HTMLElement {
    const mon = draft[side][index];
    const pinned = side === 'allies' && index === 0;
    const slots = pinned ? 3 : 4;
    const moveOptions: [string, string][] = [['', '—'], ...moves.filter((move) => !(pinned && move.id === options.editedId)).map((move): [string, string] => [move.id, move.name])];

    const setSpecies = (name: string) => {
      const species = findSpecies(name);
      const renamed = mon.name === mon.species || !mon.name.trim();
      mon.species = species ? name : 'Custom';
      if (species) {
        if (renamed) mon.name = name;
        mon.types = [...species.types];
        mon.stats = statsFor(species, mon.level);
        mon.hp = mon.stats.max_hp;
      }
      render();
    };
    const setLevel = (level: number) => {
      mon.level = level;
      const species = findSpecies(mon.species);
      if (species) {
        mon.stats = statsFor(species, level);
        mon.hp = mon.stats.max_hp;
      }
      render();
    };

    return h('div', { class: 'setup-card' },
      h('div', { class: 'setup-row' },
        h('div', { class: 'sprite sprite-small', style: `--tint: ${TYPE_COLORS[mon.types[0]] ?? TYPE_COLORS.Normal}` }, h('span', null, mon.name.slice(0, 1) || '?')),
        h('label', { class: 'field grow' }, 'Pokémon',
          select([...SPECIES.map((species): [string, string] => [species.name, species.name]), ['Custom', 'Custom']], mon.species, setSpecies, 'Pokémon'),
        ),
        h('label', { class: 'field grow' }, 'Name',
          h('input', { type: 'text', class: 'in', value: mon.name, maxlength: '24', oninput: (event: Event) => { mon.name = (event.target as HTMLInputElement).value; } }),
        ),
        h('label', { class: 'field level' }, 'Level', numberInput(mon.level, 1, 100, setLevel)),
        draft[side].length > 1
          ? h('button', { type: 'button', class: 'icon-btn', 'aria-label': `Remove ${mon.name}`, onclick: () => { draft[side].splice(index, 1); render(); } }, icon('trash', 15))
          : null,
      ),
      h('div', { class: 'setup-row' },
        h('label', { class: 'field grow' }, 'Type',
          select(TYPES.map((type): [string, string] => [type, type]), mon.types[0] ?? 'Normal', (value) => { mon.types[0] = value; mon.species = 'Custom'; render(); }, 'First type'),
        ),
        h('label', { class: 'field grow' }, 'Second type',
          select([['', '—'], ...TYPES.map((type): [string, string] => [type, type])], mon.types[1] ?? '', (value) => { mon.types = value ? [mon.types[0], value] : [mon.types[0]]; mon.species = 'Custom'; render(); }, 'Second type'),
        ),
        h('label', { class: 'field grow' }, 'Starts the battle',
          select([['', 'Healthy'], ...STATUSES.map((status): [string, string] => [status, STATUS_LABELS[status]])], mon.status ?? '', (value) => { mon.status = value || null; }, 'Starting condition'),
        ),
      ),
      h('label', { class: 'field' }, 'Starting HP',
        h('span', { class: 'hp-row' },
          h('input', { type: 'range', min: '1', max: String(mon.stats.max_hp), value: String(mon.hp), oninput: (event: Event) => {
            mon.hp = Number((event.target as HTMLInputElement).value);
            const out = (event.target as HTMLElement).parentElement?.querySelector('output');
            if (out) out.textContent = `${mon.hp} / ${mon.stats.max_hp}`;
          } }),
          h('output', null, `${mon.hp} / ${mon.stats.max_hp}`),
        ),
      ),
      h('div', null,
        h('div', { class: 'field-title' }, h('span', null, 'Stats'), h('span', { class: 'muted' }, mon.species === 'Custom' ? 'your own numbers' : `filled in from ${mon.species} at level ${mon.level}`)),
        h('div', { class: 'stat-grid' },
          STAT_FIELDS.map(([key, label]) =>
            h('label', { class: 'field stat' }, label,
              numberInput(mon.stats[key], 1, 999, (value) => {
                mon.stats[key] = value;
                if (key === 'max_hp') { mon.hp = Math.min(mon.hp, value) || value; render(); }
              }),
            ),
          ),
        ),
      ),
      h('div', null,
        h('div', { class: 'field-title' }, h('span', null, 'Moves')),
        h('div', { class: 'move-grid' },
          pinned ? h('div', { class: 'move-pinned' }, h('span', null, options.editedName), h('span', { class: 'tag' }, 'editing')) : null,
          Array.from({ length: slots }, (_, slot) =>
            select(moveOptions, mon.moves[slot] ?? '', (value) => {
              const next = [...mon.moves];
              next[slot] = value;
              mon.moves = next.filter((id, position) => id && next.indexOf(id) === position);
              render();
            }, `Move ${slot + 1}`),
          ),
        ),
      ),
    );
  }

  function sideColumn(side: 'allies' | 'foes'): HTMLElement {
    const title = side === 'allies' ? 'Your side' : 'Other side';
    const addName = side === 'allies' ? 'Eevee' : 'Furret';
    return h('section', { class: 'setup-side', 'aria-label': title },
      h('div', { class: 'setup-side-head' },
        h('h3', null, title),
        side === 'allies'
          ? h('span', { class: 'muted' }, 'you pick its moves in the battle')
          : h('label', { class: 'inline-field' }, 'Picks',
              select(
                [['first', 'always its first move'], ['random', 'a random move each turn'], ['manual', 'whatever I choose']],
                draft.foePolicy,
                (value) => { draft.foePolicy = value as FoePolicy; },
                'How the other side picks moves',
              ),
            ),
      ),
      draft[side].map((_, index) => monCard(side, index)),
      draft[side].length < MAX_PER_SIDE
        ? h('button', { type: 'button', class: 'add-btn', onclick: () => { draft[side].push(monFor(addName, draft[side][0]?.level ?? 30, ['tackle'])); render(); } },
            icon('plus', 13), `Add a Pokémon to ${side === 'allies' ? 'your side' : 'the other side'}`)
        : null,
    );
  }

  const situations: [string, () => void][] = [
    ['Target asleep', () => { draft.foes[0].status = 'Asleep'; }],
    ['User burned', () => { draft.allies[0].status = 'Burned'; }],
    ['User asleep', () => { draft.allies[0].status = 'Asleep'; }],
    ['User at low HP', () => { draft.allies[0].hp = Math.max(1, Math.ceil(draft.allies[0].stats.max_hp / 5)); }],
    ['Everyone healthy', () => { for (const mon of [...draft.allies, ...draft.foes]) { mon.status = null; mon.hp = mon.stats.max_hp; } }],
    ['Two on two', () => {
      if (draft.allies.length < 2) draft.allies.push(monFor('Eevee', draft.allies[0].level, ['tackle']));
      if (draft.foes.length < 2) draft.foes.push(monFor('Furret', draft.foes[0].level, ['tackle']));
    }],
  ];

  function render(): void {
    clear(body);
    body.append(
      h('div', { class: 'setup-head' },
        h('div', null,
          h('h2', { id: 'setup-title' }, 'Set up the test battle'),
          h('p', { class: 'muted' }, 'Takes effect when you press Start battle.'),
        ),
        h('button', { type: 'button', class: 'icon-btn', 'aria-label': 'Close without changes', onclick: close }, icon('close', 16)),
      ),
      h('div', { class: 'setup-quick' },
        h('span', { class: 'field-title' }, 'Quick situations'),
        situations.map(([label, run]) => h('button', { type: 'button', class: 'pill', onclick: () => { run(); render(); } }, label)),
      ),
      h('div', { class: 'setup-sides' }, sideColumn('allies'), sideColumn('foes')),
      h('p', { class: 'muted setup-note' }, 'Everyone on a side fights at once. Add more to try moves that hit several targets, like Explosion or Thrash.'),
      h('div', { class: 'setup-foot' },
        h('label', { class: 'field' }, 'Dice seed',
          h('span', { class: 'seed-row' },
            numberInput(draft.seed, 0, 999999, (value) => { draft.seed = value; }),
            h('button', { type: 'button', class: 'icon-btn', 'aria-label': 'Pick a new seed', onclick: () => { draft.seed = Math.floor(Math.random() * 9999) + 1; render(); } }, icon('dice', 15)),
          ),
          h('span', { class: 'muted' }, 'Same seed, same rolls.'),
        ),
        h('fieldset', { class: 'radio-group' },
          h('legend', null, 'When I edit the move'),
          ([['replay', 'Restart and replay my choices'], ['restart', 'Restart from turn 1'], ['manual', 'Wait until I press Restart']] as [OnEdit, string][]).map(([value, label]) =>
            h('label', { class: 'check' },
              h('input', { type: 'radio', name: 'on-edit', checked: draft.onEdit === value, onchange: () => { draft.onEdit = value; } }),
              label,
            ),
          ),
        ),
        h('div', { class: 'setup-actions' },
          h('button', { type: 'button', class: 'btn btn-large', onclick: close }, 'Cancel'),
          h('button', { type: 'button', class: 'btn btn-primary btn-large', onclick: apply }, 'Start battle'),
        ),
      ),
    );
  }

  render();
  dialog.append(body);
  dialog.addEventListener('cancel', () => dialog.remove());
  document.body.append(dialog);
  dialog.showModal();
}

/** The setup used the first time the app opens. */
export function defaultSetup(): BattleSetup {
  return {
    seed: 7,
    allies: [monFor('Sunflora', 30, ['sunny_day', 'morning_sun', 'tackle'])],
    foes: [monFor('Wobbuffet', 30, ['tackle'])],
    foePolicy: 'first',
    onEdit: 'replay',
  };
}

export type { MonSetup };
