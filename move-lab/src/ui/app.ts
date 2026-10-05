// The editor: move details, the block workspace, the generated file, and the test battle beside it.
import * as Blockly from 'blockly/core';
import * as En from 'blockly/msg/en';
import { Bench } from '../bench/bench.ts';
import type { TraceSource } from '../bench/bench.ts';
import { defineBlocks } from '../blocks/definitions.ts';
import { generate } from '../blocks/generate.ts';
import type { Generated } from '../blocks/generate.ts';
import { blocksFromInfo, canOpen, newMoveBlocks, newMoveSheet, sheetFromInfo } from '../blocks/library.ts';
import { theme } from '../blocks/theme.ts';
import { CATEGORIES, STYLE_COLORS, toolboxDefinition } from '../blocks/toolbox.ts';
import { BUILTIN_FILES } from '../data/builtin.ts';
import { Engine } from '../engine/engine.ts';
import type { EngineChoice, EngineSetup, MoveInfo, SetupMon } from '../engine/engine.ts';
import { DEFAULT_WEIGHT } from '../data/species.ts';
import { HIDING_PLACES, CATEGORIES as MOVE_CATEGORIES, TYPES, TYPE_COLORS, clamp, slug, uid } from '../model.ts';
import type { Category, MonSetup, MoveDoc } from '../model.ts';
import { loadProject, saveProject } from '../store.ts';
import type { Project } from '../store.ts';
import { renderBattle } from './battle.ts';
import type { Banner, BattleUi } from './battle.ts';
import { brand, clear, h, icon } from './dom.ts';
import { openReference } from './reference.ts';
import { defaultSetup, openSetup } from './setup.ts';

/** One runnable version of the edited move: the files to load and how to read its trace. */
interface Build {
  files: Record<string, string>;
  text: string;
  trace: TraceSource;
  catalog: Map<string, MoveInfo>;
  /** Id of the edited move in this version; it changes when a new move is renamed. */
  editedId: string;
}

const DEFAULT_SCALE = 0.85;
/** The block reference has its own address, so the browser's Back button leaves it. */
const REFERENCE_HASH = '#blocks';

interface Blocked {
  title: string;
  lines: string[];
  blockId: string | null;
}

/**
 * Blockly sizes a block from the width of its text and remembers the widths, so the blocks' font
 * has to be there before the first block is drawn. Gives up after a moment rather than hang offline.
 */
function blockFont(): Promise<unknown> {
  const { family, weight, size } = theme.fontStyle;
  const patience = new Promise((resolve) => window.setTimeout(resolve, 3000));
  return Promise.race([document.fonts.load(`${weight} ${size}pt ${family}`), patience]).catch(() => undefined);
}

export async function startApp(root: HTMLElement): Promise<void> {
  root.append(h('div', { class: 'splash' }, 'Loading the battle engine…'));
  const font = blockFont();
  let engine: Engine;
  try {
    engine = await Engine.load(new URL('engine/', document.baseURI).href);
  } catch (error) {
    clear(root);
    root.append(
      h('div', { class: 'splash' },
        h('h1', null, 'The battle engine is missing'),
        h('p', null, 'Move Lab runs the real engine as WebAssembly. Build it once with:'),
        h('pre', null, 'npm run setup:emsdk\nnpm run build:engine'),
        h('p', { class: 'muted' }, String(error instanceof Error ? error.message : error)),
      ),
    );
    return;
  }
  await font;
  Blockly.setLocale(En as unknown as { [key: string]: string });
  defineBlocks();
  clear(root);
  new App(root, engine);
}

class App {
  private readonly engine: Engine;
  private readonly bench: Bench;
  private readonly builtin: Map<string, MoveInfo>;
  private readonly project: Project;
  private doc: MoveDoc;
  private workspace!: Blockly.WorkspaceSvg;

  private generated: Generated | null = null;
  /** The newest version that loads in the engine, and the one the battle is running. */
  private pending: Build | null = null;
  private running: Build | null = null;
  private blocked: Blocked | null = null;
  private stale = false;
  private notice: Banner | null = null;
  private setupError: string | null = null;
  private traceOff = false;
  private loading = false;
  private timer = 0;
  private saved = true;
  private readonly battleUi: BattleUi = { pendingMove: null };
  private reference: { close(): void } | null = null;
  /** Whether the reference was opened from the editor, so that leaving it can step back in history. */
  private referenceFromEditor = false;

  // Elements that are updated in place.
  private readonly moveButton = h('button', { type: 'button', class: 'move-switch', 'aria-haspopup': 'dialog' });
  private readonly moveMenu = h('div', { class: 'menu', hidden: true });
  private readonly pathLabel = h('span', { class: 'path' });
  private readonly savedLabel = h('span', { class: 'saved' });
  private readonly sheetEl = h('section', { class: 'sheet', 'aria-label': 'Move details' });
  private readonly railEl = h('nav', { class: 'rail', 'aria-label': 'Block categories' });
  private readonly blocklyEl = h('div', { class: 'blockly' });
  private readonly legendEl = h('div', { class: 'legend' });
  private readonly luaEl = h('section', { class: 'lua', 'aria-label': 'The move file' });
  private readonly benchEl = h('aside', { class: 'bench', 'aria-label': 'Test battle' });
  private menuSearch = '';

  constructor(root: HTMLElement, engine: Engine) {
    this.engine = engine;
    this.bench = new Bench(engine);
    engine.setMoves(BUILTIN_FILES);
    this.builtin = new Map(engine.catalog().map((move) => [move.id, move]));

    const stored = loadProject();
    if (stored) {
      this.project = stored;
    } else {
      const first = this.builtin.get('solar_beam') ?? [...this.builtin.values()].find(canOpen)!;
      const doc = this.docFromInfo(first);
      this.project = { version: 1, docs: [doc], current: doc.uid, setup: defaultSetup(), luaOpen: true };
    }
    this.doc = this.project.docs.find((doc) => doc.uid === this.project.current) ?? this.project.docs[0];

    this.buildDom(root);
    this.initWorkspace();
    this.openDoc(this.doc);
    window.addEventListener('hashchange', () => {
      this.referenceFromEditor = true;
      this.syncReference();
    });
    this.syncReference();
    if (import.meta.env.DEV) (window as unknown as { moveLab: unknown }).moveLab = this;
  }

  // -------------------------------------------------------------------------
  // The block reference

  /** Shows or hides the reference to match the address. */
  private syncReference(): void {
    const wanted = window.location.hash === REFERENCE_HASH;
    if (wanted && !this.reference) {
      this.toggleMenu(false);
      this.reference = openReference({ moveName: this.doc.sheet.name || 'your move', onClose: () => this.leaveReference() });
    } else if (!wanted && this.reference) {
      this.reference.close();
      this.reference = null;
    }
  }

  private leaveReference(): void {
    if (this.referenceFromEditor) {
      window.history.back();
      return;
    }
    // The page was opened straight on the reference: there is no editor entry to go back to.
    window.history.replaceState(null, '', window.location.pathname + window.location.search);
    this.syncReference();
  }

  // -------------------------------------------------------------------------
  // Layout

  private buildDom(root: HTMLElement): void {
    this.moveButton.addEventListener('click', () => this.toggleMenu());
    document.addEventListener('pointerdown', (event) => {
      if (this.moveMenu.hidden) return;
      const target = event.target as Node;
      if (!this.moveMenu.contains(target) && !this.moveButton.contains(target)) this.toggleMenu(false);
    });
    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape' && !this.moveMenu.hidden) this.toggleMenu(false);
    });

    const header = h('header', { class: 'topbar' },
      brand(),
      h('div', { class: 'move-switch-wrap' }, this.moveButton, this.moveMenu),
      this.pathLabel,
      h('div', { class: 'top-actions' },
        this.savedLabel,
        h('button', { type: 'button', class: 'top-icon', 'aria-label': 'Undo', title: 'Undo', onclick: () => this.workspace.undo(false) }, icon('undo')),
        h('button', { type: 'button', class: 'top-icon', 'aria-label': 'Redo', title: 'Redo', onclick: () => this.workspace.undo(true) }, icon('redo')),
        h('a', { class: 'top-link', href: REFERENCE_HASH, title: 'What each block does' }, 'All blocks'),
        h('button', { type: 'button', class: 'top-primary', onclick: () => this.download() }, icon('download'), 'Download .lua'),
      ),
    );

    const zoom = h('div', { class: 'zoom' },
      h('button', { type: 'button', 'aria-label': 'Zoom out', title: 'Zoom out', onclick: () => this.workspace.zoomCenter(-1) }, icon('minus', 15)),
      h('button', { type: 'button', 'aria-label': 'Zoom in', title: 'Zoom in', onclick: () => this.workspace.zoomCenter(1) }, icon('plus', 15)),
      h('button', { type: 'button', 'aria-label': 'Fit the blocks on screen', title: 'Fit the blocks on screen', onclick: () => this.workspace.zoomToFit() }, icon('fit', 15)),
    );

    root.append(
      header,
      h('div', { class: 'main' },
        h('section', { class: 'editor' },
          this.sheetEl,
          h('div', { class: 'work' }, this.railEl, h('div', { class: 'canvas' }, this.blocklyEl, this.legendEl, zoom)),
          this.luaEl,
        ),
        this.benchEl,
      ),
    );
    this.renderRail('');
  }

  private renderRail(search: string): void {
    clear(this.railEl);
    this.railEl.append(
      h('label', { class: 'search' }, icon('search', 15),
        h('input', {
          type: 'search', placeholder: 'Find a block', 'aria-label': 'Find a block', value: search,
          oninput: (event: Event) => {
            const value = (event.target as HTMLInputElement).value;
            this.workspace.updateToolbox(toolboxDefinition(value));
            this.placeLegend();
            for (const button of this.railEl.querySelectorAll('button')) button.toggleAttribute('disabled', value.trim() !== '');
          },
        }),
      ),
      h('div', { class: 'rail-title' }, 'Blocks'),
      ...CATEGORIES.map((category, index) =>
        h('button', { type: 'button', class: 'rail-item', 'data-category': category.id, onclick: () => this.scrollPalette(index) },
          h('span', { class: `swatch${category.id === 'info' ? ' swatch-round' : ''}`, style: `background: ${category.color}` }),
          category.name,
        ),
      ),
    );
  }

  /** Scrolls the always-open palette to a category's label. */
  private scrollPalette(index: number): void {
    const flyout = this.workspace.getFlyout() as unknown as {
      getContents(): { getType(): string; getElement(): { getBoundingRectangle(): { top: number } } }[];
      getWorkspace(): Blockly.WorkspaceSvg;
    } | null;
    if (!flyout) return;
    const label = flyout.getContents().filter((item) => item.getType() === 'label')[index];
    if (!label) return;
    const inner = flyout.getWorkspace();
    const metrics = inner.getMetrics();
    const top = label.getElement().getBoundingRectangle().top * inner.scale - 10;
    inner.scrollbar?.setY(clamp(top, 0, Math.max(0, metrics.scrollHeight - metrics.viewHeight)));
    for (const button of this.railEl.querySelectorAll('.rail-item')) button.classList.toggle('active', button === this.railEl.querySelectorAll('.rail-item')[index]);
  }

  private initWorkspace(): void {
    this.workspace = Blockly.inject(this.blocklyEl, {
      toolbox: toolboxDefinition(),
      renderer: 'zelos',
      theme,
      grid: { spacing: 20, length: 1.4, colour: '#BFC8BD', snap: false },
      zoom: { controls: false, wheel: true, pinch: true, startScale: DEFAULT_SCALE, maxScale: 1.6, minScale: 0.4, scaleSpeed: 1.1 },
      move: { scrollbars: true, drag: true, wheel: true },
      trashcan: false,
      sounds: false,
      maxInstances: { mlab_on_use: 1, mlab_on_hit: 1, mlab_on_interrupt: 1, mlab_on_turn_start: 1 },
    });
    // The palette keeps its own size instead of following the workspace zoom.
    const flyout = this.workspace.getFlyout() as unknown as { getFlyoutScale: () => number; reflow(): void } | null;
    if (flyout) {
      flyout.getFlyoutScale = () => 0.72;
      flyout.reflow();
    }
    // Blocks left lying around outside a stack are greyed out and ignored.
    this.workspace.addChangeListener(Blockly.Events.disableOrphans);
    this.workspace.addChangeListener((event) => {
      if (event.isUiEvent || this.loading) return;
      this.schedule();
    });
    new ResizeObserver(() => {
      Blockly.svgResize(this.workspace);
      this.placeLegend();
    }).observe(this.blocklyEl);
  }

  // -------------------------------------------------------------------------
  // Moves

  private docFromInfo(info: MoveInfo): MoveDoc {
    return { uid: uid(), sheet: sheetFromInfo(info), blocks: blocksFromInfo(info), idFollowsName: false, lua: '', valid: false };
  }

  private freeId(wanted: string): string {
    const taken = (id: string) => this.builtin.has(id) || this.project.docs.some((doc) => doc !== this.doc && doc.sheet.id === id);
    if (!taken(wanted)) return wanted;
    for (let suffix = 2; ; suffix += 1) if (!taken(`${wanted}_${suffix}`)) return `${wanted}_${suffix}`;
  }

  private openDoc(doc: MoveDoc): void {
    window.clearTimeout(this.timer);
    this.doc = doc;
    this.project.current = doc.uid;
    this.traceOff = false;
    this.notice = null;
    this.stale = false;
    this.loading = true;
    Blockly.Events.disable();
    try {
      this.workspace.clear();
      Blockly.serialization.workspaces.load(doc.blocks as Parameters<typeof Blockly.serialization.workspaces.load>[0], this.workspace, { recordUndo: false });
      if (!this.workspace.getTopBlocks(false).some((block) => block.type === 'mlab_on_use')) {
        Blockly.serialization.blocks.append({ type: 'mlab_on_use', x: 28, y: 28, deletable: false }, this.workspace, { recordUndo: false });
      }
    } finally {
      Blockly.Events.enable();
      this.loading = false;
    }
    this.workspace.clearUndo();
    this.renderSheet();
    this.refresh(true);
    void Blockly.renderManagement.finishQueuedRenders().then(() => this.showStart());
  }

  /** Shows the stacks from their top-left corner, zoomed out a little if the widest row would not fit. */
  private showStart(): void {
    const workspace = this.workspace;
    const box = workspace.getBlocksBoundingBox();
    const view = workspace.getMetricsManager().getViewMetrics();
    // The view is measured in pixels, the blocks in workspace units.
    const fits = box.right > 0 ? view.width / (box.right + 36) : DEFAULT_SCALE;
    workspace.setScale(clamp(Math.min(fits, DEFAULT_SCALE), 0.55, DEFAULT_SCALE));
    workspace.scroll(0, 0);
    this.placeLegend();
  }

  private placeLegend(): void {
    const width = this.workspace.getFlyout()?.getWidth() ?? 0;
    this.legendEl.style.left = `${Math.round(width) + 16}px`;
  }

  private newMove(): void {
    const doc: MoveDoc = { uid: uid(), sheet: newMoveSheet('New Move', 'new_move'), blocks: newMoveBlocks(), idFollowsName: true, lua: '', valid: false };
    this.project.docs.push(doc);
    this.doc = doc;
    doc.sheet.id = this.freeId('new_move');
    this.openDoc(doc);
  }

  private deleteDoc(doc: MoveDoc): void {
    const builtin = this.builtin.has(doc.sheet.id);
    const question = builtin
      ? `Put ${doc.sheet.name} back to the built-in version? Your edits to it will be removed from this browser.`
      : `Delete ${doc.sheet.name}? It will be removed from this browser.`;
    if (!window.confirm(question)) return;
    this.project.docs.splice(this.project.docs.indexOf(doc), 1);
    if (this.project.docs.length === 0) this.project.docs.push(this.docFromInfo(this.builtin.get('tackle') ?? [...this.builtin.values()].find(canOpen)!));
    if (doc === this.doc) this.openDoc(this.project.docs[0]);
    else {
      this.save();
      this.refresh(true);
    }
    this.renderMenu();
  }

  private toggleMenu(open = this.moveMenu.hidden): void {
    this.moveMenu.hidden = !open;
    this.moveButton.setAttribute('aria-expanded', String(open));
    if (open) {
      this.menuSearch = '';
      this.renderMenu();
      this.moveMenu.querySelector('input')?.focus();
    }
  }

  private renderMenu(): void {
    const needle = this.menuSearch.trim().toLowerCase();
    const matches = (name: string, id: string) => !needle || name.toLowerCase().includes(needle) || id.includes(needle);
    const mine = this.project.docs.filter((doc) => matches(doc.sheet.name, doc.sheet.id));
    const overridden = new Set(this.project.docs.map((doc) => doc.sheet.id));
    const builtins = [...this.builtin.values()]
      .filter((move) => move.id !== 'struggle' && !overridden.has(move.id) && matches(move.name, move.id))
      .sort((a, b) => a.name.localeCompare(b.name));
    const dot = (type: string) => h('span', { class: 'dot', style: `background: ${TYPE_COLORS[type] ?? TYPE_COLORS.Normal}` });

    const input = h('input', {
      type: 'search', placeholder: 'Search moves', 'aria-label': 'Search moves', value: this.menuSearch,
      oninput: (event: Event) => {
        this.menuSearch = (event.target as HTMLInputElement).value;
        this.renderMenu();
        const again = this.moveMenu.querySelector('input');
        again?.focus();
        again?.setSelectionRange(this.menuSearch.length, this.menuSearch.length);
      },
    });

    clear(this.moveMenu);
    this.moveMenu.append(
      h('div', { class: 'menu-top' },
        h('label', { class: 'search' }, icon('search', 15), input),
        h('button', { type: 'button', class: 'btn btn-primary', onclick: () => { this.toggleMenu(false); this.newMove(); } }, icon('plus', 14), 'New move'),
      ),
      h('div', { class: 'menu-list' },
        h('div', { class: 'menu-title' }, 'Your moves'),
        mine.map((doc) =>
          h('div', { class: `menu-row${doc === this.doc ? ' current' : ''}` },
            h('button', { type: 'button', class: 'menu-open', onclick: () => { this.toggleMenu(false); if (doc !== this.doc) this.openDoc(doc); } },
              dot(doc.sheet.type), h('span', { class: 'menu-name' }, doc.sheet.name || 'Unnamed'),
              h('span', { class: 'menu-meta' }, this.builtin.has(doc.sheet.id) ? 'edited built-in' : `${doc.sheet.id}.lua`),
            ),
            h('button', { type: 'button', class: 'icon-btn', 'aria-label': this.builtin.has(doc.sheet.id) ? `Reset ${doc.sheet.name} to the built-in version` : `Delete ${doc.sheet.name}`,
              title: this.builtin.has(doc.sheet.id) ? 'Reset to the built-in version' : 'Delete', onclick: () => this.deleteDoc(doc) }, icon(this.builtin.has(doc.sheet.id) ? 'restart' : 'trash', 15)),
          ),
        ),
        mine.length === 0 ? h('div', { class: 'menu-none muted' }, 'No move of yours matches.') : null,
        h('div', { class: 'menu-title' }, 'Built-in moves', h('span', { class: 'muted' }, 'open one to change it')),
        builtins.map((move) =>
          h('div', { class: 'menu-row' },
            h('button', { type: 'button', class: 'menu-open', disabled: !canOpen(move), onclick: () => {
              this.toggleMenu(false);
              const doc = this.docFromInfo(move);
              this.project.docs.push(doc);
              this.openDoc(doc);
            } },
              dot(move.type), h('span', { class: 'menu-name' }, move.name), h('span', { class: 'menu-meta' }, `${move.type} · ${move.category}`),
            ),
          ),
        ),
        builtins.length === 0 ? h('div', { class: 'menu-none muted' }, 'No built-in move matches.') : null,
      ),
    );
  }

  // -------------------------------------------------------------------------
  // Move details

  private renderSheet(): void {
    const sheet = this.doc.sheet;
    const changed = () => {
      this.updateTop();
      this.schedule();
    };
    const field = (label: string, control: HTMLElement, cls = '') => h('label', { class: `field ${cls}` }, label, control);
    const select = (items: readonly string[], value: string, set: (value: string) => void, label: string) =>
      h('select', { class: 'in', 'aria-label': label, value, onchange: (event: Event) => { set((event.target as HTMLSelectElement).value); changed(); } },
        items.map((item) => h('option', { value: item }, item)),
      );
    const number = (value: number, min: number, max: number, set: (value: number) => void, extra: Record<string, unknown> = {}) =>
      h('input', {
        type: 'number', class: 'in', value: String(value), min: String(min), max: String(max), ...extra,
        onchange: (event: Event) => {
          const input = event.target as HTMLInputElement;
          const next = clamp(Number(input.value) || 0, min, max);
          input.value = String(next);
          set(next);
          changed();
        },
      });

    const check = (label: string, checked: boolean, set: (on: boolean) => void) =>
      h('label', { class: 'check' },
        h('input', { type: 'checkbox', checked, onchange: (event: Event) => { set((event.target as HTMLInputElement).checked); changed(); } }),
        label,
      );
    /** Adds or removes one word of a list kept on the sheet, leaving the others alone. */
    const toggled = (list: string[] | undefined, word: string, on: boolean) => {
      const rest = (list ?? []).filter((other) => other !== word);
      return on ? [...rest, word] : rest;
    };

    const percent = sheet.accuracy.kind === 'chance' ? sheet.accuracy.percent : 100;
    const accuracyInput = number(percent, 1, 100, (value) => { sheet.accuracy = { kind: 'chance', percent: value }; }, { step: 'any', disabled: sheet.accuracy.kind !== 'chance' });

    const more = h('details', { class: 'more' },
      h('summary', { class: 'btn' }, 'More', icon('caret', 13)),
      h('div', { class: 'more-panel' },
        field('Accuracy rule',
          h('select', { class: 'in', value: sheet.accuracy.kind, onchange: (event: Event) => {
            const kind = (event.target as HTMLSelectElement).value;
            sheet.accuracy = kind === 'chance' ? { kind: 'chance', percent } : { kind: kind as 'always' | 'ohko' };
            changed();
            this.renderSheet();
          } },
            h('option', { value: 'chance' }, 'Can miss (uses the %)'),
            h('option', { value: 'always' }, 'Never misses'),
            h('option', { value: 'ohko' }, 'One-hit KO rule'),
          ),
        ),
        field('Priority', number(sheet.priority, -7, 7, (value) => { sheet.priority = Math.round(value); })),
        h('p', { class: 'muted' }, 'Higher priority moves go first. Most moves are 0.'),
        check('Fails if the target has full HP', sheet.failOnFullHp, (on) => { sheet.failOnFullHp = on; }),
        check('Can be used while asleep', sheet.usableWhileAsleep, (on) => { sheet.usableWhileAsleep = on; }),
        check('Can be used while frozen, and thaws the user', sheet.usableWhileFrozen === true, (on) => { sheet.usableWhileFrozen = on; }),
        check('Makes contact', sheet.flags?.includes('contact') === true, (on) => { sheet.flags = toggled(sheet.flags, 'contact', on); }),
        h('div', { class: 'more-group' },
          h('span', { class: 'more-title' }, 'Reaches Pokémon hiding'),
          h('div', { class: 'more-row' },
            HIDING_PLACES.map(([place, label]) =>
              check(label, sheet.hitsHidden?.includes(place) === true, (on) => { sheet.hitsHidden = toggled(sheet.hitsHidden, place, on); }),
            ),
          ),
        ),
      ),
    );

    clear(this.sheetEl);
    this.sheetEl.append(
      field('Name',
        h('input', {
          type: 'text', class: 'in in-name', value: sheet.name, maxlength: '40',
          oninput: (event: Event) => {
            sheet.name = (event.target as HTMLInputElement).value;
            if (this.doc.idFollowsName) sheet.id = this.freeId(slug(sheet.name));
            changed();
          },
        }),
        'grow',
      ),
      field('Type', select(TYPES, sheet.type, (value) => { sheet.type = value; }, 'Type')),
      field('Category', select(MOVE_CATEGORIES, sheet.category, (value) => { sheet.category = value as Category; }, 'Category')),
      field('PP', number(sheet.pp, 1, 99, (value) => { sheet.pp = Math.round(value); }), 'narrow'),
      field('Accuracy %', accuracyInput, 'narrow'),
      more,
    );
  }

  private updateTop(): void {
    const sheet = this.doc.sheet;
    clear(this.moveButton);
    this.moveButton.append(
      h('span', { class: 'dot', style: `background: ${TYPE_COLORS[sheet.type] ?? TYPE_COLORS.Normal}` }),
      h('span', null, sheet.name || 'Unnamed move'),
      icon('caret', 14),
    );
    this.pathLabel.textContent = `moves/${sheet.id}.lua`;
    this.savedLabel.textContent = this.saved ? 'Saved in this browser' : 'Could not save in this browser';
  }

  private download(): void {
    const generated = this.generated;
    if (!generated) return;
    if (this.blocked && !window.confirm('This move has a problem and will not load in the game yet. Download it anyway?')) return;
    const url = URL.createObjectURL(new Blob([generated.lua], { type: 'text/plain' }));
    const link = h('a', { href: url, download: `${this.doc.sheet.id}.lua` });
    document.body.append(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  // -------------------------------------------------------------------------
  // From blocks to a running battle

  private schedule(): void {
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => {
      // A drag fires its own change when it ends.
      if (this.workspace.isDragging()) this.schedule();
      else this.refresh();
    }, 250);
  }

  private save(): void {
    this.saved = saveProject(this.project);
    this.savedLabel.textContent = this.saved ? 'Saved in this browser' : 'Could not save in this browser';
  }

  private movesFor(text: string): Record<string, string> {
    const files = { ...BUILTIN_FILES };
    for (const other of this.project.docs) {
      if (other !== this.doc && other.valid && other.lua) files[`${other.sheet.id}.lua`] = other.lua;
    }
    files[`${this.doc.sheet.id}.lua`] = text;
    return files;
  }

  /** Regenerates the file from the blocks and, if it loads, lets the battle pick it up. */
  private refresh(force = false): void {
    const doc = this.doc;
    const generated = generate(this.workspace, doc.sheet);
    this.generated = generated;
    doc.blocks = Blockly.serialization.workspaces.save(this.workspace);
    doc.lua = generated.lua;
    this.showProblems(generated);
    this.renderLua();
    this.updateTop();

    let blocked: Blocked | null = null;
    if (generated.problems.length) {
      const count = generated.problems.length;
      blocked = {
        title: `Not restarted: ${count} problem${count === 1 ? '' : 's'} in ${doc.sheet.name || 'this move'}`,
        lines: generated.problems.slice(0, 2).map((problem) => problem.message),
        blockId: generated.problems.find((problem) => problem.blockId)?.blockId ?? null,
      };
    } else {
      const text = this.traceOff ? generated.lua : generated.traced;
      const files = this.movesFor(text);
      this.engine.setMoves(files);
      try {
        const catalog = new Map(this.engine.catalog().map((move) => [move.id, move]));
        this.pending = {
          files,
          text,
          catalog,
          editedId: doc.sheet.id,
          trace: { moveName: doc.sheet.name, mode: generated.mode, marks: generated.marks, effectBlocks: generated.effectBlocks },
        };
      } catch (error) {
        blocked = { title: 'Not restarted: the engine could not load this move', lines: [error instanceof Error ? error.message : String(error)], blockId: null };
      }
    }
    doc.valid = blocked === null;
    this.blocked = blocked;
    this.save();

    const started = this.running !== null;
    if (blocked || !this.pending) {
      this.renderBench();
      return;
    }
    if (!force && started && this.pending.text === this.running?.text) {
      this.renderBench();
      return;
    }
    if (!force && started && this.project.setup.onEdit === 'manual') {
      this.stale = true;
      this.renderBench();
      return;
    }
    this.restart(!force && started && this.project.setup.onEdit === 'replay' ? 'replay' : 'fresh');
  }

  private restart(mode: 'fresh' | 'replay'): void {
    // While the edited move is broken, keep battling with the last version that worked.
    const build = this.blocked ? this.running : this.pending;
    if (!build) {
      this.renderBench();
      return;
    }
    this.engine.setMoves(build.files);
    // Remembered choices name moves by id; follow the edited move if it was renamed since.
    const previousId = this.running?.editedId;
    const picks = mode === 'replay' ? this.bench.view.picks.map((pick) => (pick.moveId === previousId ? { ...pick, moveId: build.editedId } : pick)) : [];
    const before = mode === 'replay' && picks.length ? this.bench.summary() : null;
    this.setupError = null;
    this.battleUi.pendingMove = null;
    try {
      this.bench.start({ setup: this.engineSetup(build), policy: this.project.setup.foePolicy, trace: build.trace, replay: picks, compareWith: before });
    } catch (error) {
      this.setupError = error instanceof Error ? error.message : String(error);
      this.renderBench();
      return;
    }
    const view = this.bench.view;
    // Markers count as script steps. If that alone trips the engine's limit, run without them.
    if (view.phase === 'crashed' && !this.traceOff && /exceeded \d+ operations/.test(view.error ?? '')) {
      this.traceOff = true;
      this.refresh(true);
      return;
    }
    this.running = build;
    this.stale = false;
    this.notice = null;
    if (mode === 'replay' && before) {
      const count = view.replayed;
      this.notice = {
        tone: 'good',
        title: 'Restarted with your edit',
        lines: [
          `Replayed your ${count} choice${count === 1 ? '' : 's'} on seed ${this.project.setup.seed}.${view.replayCut ? ' The rest were no longer possible.' : ''}`,
          ...(view.changes.length ? view.changes.slice(0, 3) : ['Nothing in this battle changed.']),
        ],
      };
    }
    this.renderBench();
  }

  private engineSetup(build: Build): EngineSetup {
    const setup = this.project.setup;
    const edited = this.doc.sheet.id;
    const known = (id: string) => build.catalog.has(id) && id !== 'struggle';
    const convert = (mon: MonSetup, lead: boolean): SetupMon => {
      const moves = [...(lead ? [edited] : []), ...mon.moves].filter((id, index, all) => known(id) && all.indexOf(id) === index).slice(0, 4);
      if (moves.length === 0) moves.push(known('tackle') ? 'tackle' : [...build.catalog.keys()].find(known) ?? edited);
      const maxHp = Math.max(1, mon.stats.max_hp);
      const result: SetupMon = {
        name: mon.name.trim() || mon.species,
        moves,
        level: mon.level,
        max_hp: maxHp,
        hp: clamp(mon.hp, 1, maxHp),
        attack: mon.stats.attack,
        defense: mon.stats.defense,
        sp_attack: mon.stats.sp_attack,
        sp_defense: mon.stats.sp_defense,
        speed: mon.stats.speed,
        types: mon.types.filter(Boolean),
        // The game names species its own way; here it is the name in small letters.
        species: slug(mon.species === 'Custom' ? mon.name : mon.species),
        gender: mon.gender ?? 'Genderless',
        weight: mon.weight ?? DEFAULT_WEIGHT,
      };
      if (mon.item) result.item = mon.item;
      if (mon.ability) result.ability = mon.ability;
      if (mon.status) {
        result.status = mon.status;
        // Sleep needs some turns left or the Pokémon wakes at once; bad poison starts at its first step.
        result.status_turns = mon.status === 'Asleep' ? 3 : mon.status === 'BadlyPoisoned' ? 1 : 0;
      }
      return result;
    };
    const result: EngineSetup = { seed: setup.seed, allies: setup.allies.map((mon, index) => convert(mon, index === 0)), foes: setup.foes.map((mon) => convert(mon, false)) };
    if (setup.environment) result.environment = setup.environment;
    return result;
  }

  // -------------------------------------------------------------------------
  // Feedback on the blocks

  private blockColor(id: string): string | null {
    const block = this.workspace.getBlockById(id) as Blockly.BlockSvg | null;
    return block ? (STYLE_COLORS[block.getStyleName()] ?? block.getColour()) : null;
  }

  private focusBlock(id: string): void {
    const block = this.workspace.getBlockById(id) as Blockly.BlockSvg | null;
    if (!block) return;
    this.workspace.centerOnBlock(id);
    Blockly.common.setSelected(block);
  }

  private showProblems(generated: Generated): void {
    const byBlock = new Map<string, string[]>();
    for (const problem of generated.problems) {
      if (problem.blockId) byBlock.set(problem.blockId, [...(byBlock.get(problem.blockId) ?? []), problem.message]);
    }
    for (const block of this.workspace.getAllBlocks(false)) {
      const messages = byBlock.get(block.id);
      block.setWarningText(messages ? messages.join('\n\n') : null);
      (block as Blockly.BlockSvg).removeClass('mlab-problem');
      if (messages) (block as Blockly.BlockSvg).addClass('mlab-problem');
    }
  }

  private showTrace(): void {
    for (const block of this.workspace.getAllBlocks(false)) (block as Blockly.BlockSvg).removeClass('mlab-ran');
    const view = this.bench.view;
    // The trace belongs to the version the battle runs, which is the one on screen unless it is blocked or stale.
    const current = !this.blocked && !this.stale && this.running?.text === this.pending?.text;
    const ran = current ? view.ran : [];
    for (const id of ran) (this.workspace.getBlockById(id) as Blockly.BlockSvg | null)?.addClass('mlab-ran');
    clear(this.legendEl);
    this.legendEl.append(
      h('span', { class: 'legend-swatch' }),
      this.traceOff
        ? 'Highlighting is off for this move'
        : ran.length && view.ranTurn
          ? `Glowing blocks ran on turn ${view.ranTurn}`
          : 'Blocks glow when they run in the battle',
    );
  }

  private renderLua(): void {
    const generated = this.generated;
    const open = this.project.luaOpen;
    clear(this.luaEl);
    this.luaEl.append(
      h('div', { class: 'lua-head' },
        icon('code'),
        h('strong', null, 'The move file'),
        h('span', { class: 'muted' }, 'Written from your blocks. Nothing to edit here.'),
        h('button', { type: 'button', class: 'btn btn-small lua-copy', onclick: () => { if (generated) void navigator.clipboard?.writeText(generated.lua); } }, icon('copy', 13), 'Copy'),
        h('button', { type: 'button', class: 'btn btn-small', 'aria-expanded': String(open), onclick: () => { this.project.luaOpen = !open; this.save(); this.renderLua(); } }, open ? 'Hide' : 'Show'),
      ),
    );
    if (!open || !generated) return;
    const lines = generated.annotated.trimEnd().split('\n');
    this.luaEl.append(
      h('div', { class: 'lua-code' },
        lines.map((line, index) => {
          const tag = / --@(\d+)$/.exec(line);
          const text = tag ? line.slice(0, tag.index) : line;
          const blockId = tag ? generated.marks[Number(tag[1])] : undefined;
          const color = blockId ? this.blockColor(blockId) : null;
          const parts = [
            h('span', { class: 'lua-no' }, index + 1),
            h('span', { class: 'dot', style: `background: ${color ?? 'transparent'}` }),
            h('span', { class: 'lua-text' }, text),
          ];
          return blockId
            ? h('button', { type: 'button', class: 'lua-line lua-link', title: 'Show the block that wrote this line', onclick: () => this.focusBlock(blockId) }, parts)
            : h('div', { class: 'lua-line' }, parts);
        }),
      ),
    );
  }

  // -------------------------------------------------------------------------
  // The battle panel

  private renderBench(): void {
    const view = this.bench.view;
    const banners: Banner[] = [];
    const blocked = this.blocked;
    if (blocked) {
      const blockId = blocked.blockId;
      banners.push({
        tone: 'bad',
        title: blocked.title,
        lines: [...blocked.lines, this.running ? 'This battle is still running the last version that worked.' : 'The battle starts once this is fixed.'],
        action: blockId ? { label: 'Show the block', run: () => this.focusBlock(blockId) } : undefined,
      });
    } else if (this.stale) {
      banners.push({ tone: 'warn', title: 'Move edited', lines: ['This battle still uses the previous version.'], action: { label: 'Restart with edits', run: () => this.restart('fresh') } });
    } else if (this.notice) {
      banners.push(this.notice);
    }
    if (this.setupError) {
      banners.push({ tone: 'bad', title: 'The battle could not start', lines: [this.setupError], action: { label: 'Open setup', run: () => this.showSetup() } });
    }
    if (view.phase === 'crashed') {
      banners.push({ tone: 'bad', title: 'The engine stopped this battle', lines: [view.error ?? 'No message.', 'Change the move or restart to try again.'] });
    }

    const catalog = (this.blocked ? this.running?.catalog : this.pending?.catalog) ?? this.builtin;
    renderBattle(
      this.benchEl,
      {
        view,
        catalog,
        editedId: this.doc.sheet.id,
        banners,
        seed: this.project.setup.seed,
        autoRestart: this.project.setup.onEdit !== 'manual',
        blockColor: (id) => this.blockColor(id),
      },
      {
        restart: () => this.restart('fresh'),
        openSetup: () => this.showSetup(),
        choose: (choice: EngineChoice) => {
          this.notice = null;
          this.bench.choose(choice);
          this.renderBench();
        },
        newRolls: () => {
          this.project.setup.seed = (this.project.setup.seed % 9999) + 1;
          this.save();
          this.restart('fresh');
        },
        setAutoRestart: (on) => {
          this.project.setup.onEdit = on ? 'replay' : 'manual';
          this.save();
          if (on && this.stale) this.restart('replay');
          else this.renderBench();
        },
        focusBlock: (id) => this.focusBlock(id),
      },
      this.battleUi,
      () => this.renderBench(),
    );
    this.showTrace();
  }

  private showSetup(): void {
    const catalog = (this.blocked ? this.running?.catalog : this.pending?.catalog) ?? this.builtin;
    openSetup({
      setup: this.project.setup,
      catalog: [...catalog.values()],
      editedId: this.doc.sheet.id,
      editedName: this.doc.sheet.name || 'This move',
      onApply: (setup) => {
        this.project.setup = setup;
        this.save();
        this.restart('fresh');
      },
    });
  }
}
