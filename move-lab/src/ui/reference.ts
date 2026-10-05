// The "All blocks" page: every block with what it does and the line it writes into the move file.
// The pictures are the real blocks, drawn once by Blockly off screen and copied in as plain SVG.
import * as Blockly from 'blockly/core';
import { HAT_TYPES, optionSpecs } from '../blocks/definitions.ts';
import { formsOf } from '../blocks/generate.ts';
import type { BlockForms } from '../blocks/generate.ts';
import type { BlockState } from '../blocks/library.ts';
import { OPTION_FIELDS, REFERENCE } from '../blocks/reference.ts';
import type { ReferenceEntry, ReferenceSection } from '../blocks/reference.ts';
import { theme } from '../blocks/theme.ts';
import { brand, h, icon } from './dom.ts';

export interface ReferenceOptions {
  /** Name of the move being edited, for the way back. */
  moveName: string;
  onClose(): void;
}

/** One row that "+ option" can add, and the part of each line it is responsible for. */
interface OptionRow {
  menu: string;
  plain: string | null;
  script: string | null;
}

interface Drawn {
  picture: SVGSVGElement;
  /** The words on the block, for screen readers. */
  words: string;
  about: string;
  forms: BlockForms;
  options: OptionRow[];
}

/** Blocks are drawn a little smaller than in the editor (0.85) and larger than in the palette (0.72). */
const SCALE = 0.8;
const SVG = 'http://www.w3.org/2000/svg';

/** The part of a line that an option adds: what is left once the unchanged start and end are cut off. */
function added(base: string | null, full: string | null): string | null {
  if (full === null) return null;
  if (base === null) return full;
  let start = 0;
  while (start < base.length && base[start] === full[start]) start += 1;
  let end = 0;
  while (end < base.length - start && base[base.length - 1 - end] === full[full.length - 1 - end]) end += 1;
  return full.slice(start, full.length - end).replace(/^[\s,{]+|[\s,}]+$/g, '');
}

function picture(block: Blockly.BlockSvg): SVGSVGElement {
  const root = block.getSvgRoot();
  const box = root.getBBox();
  const pad = 2; // room for the outline
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', `${box.x - pad} ${box.y - pad} ${box.width + 2 * pad} ${box.height + 2 * pad}`);
  svg.setAttribute('width', String(Math.ceil((box.width + 2 * pad) * SCALE)));
  svg.setAttribute('height', String(Math.ceil((box.height + 2 * pad) * SCALE)));
  svg.setAttribute('aria-hidden', 'true');
  const copy = root.cloneNode(true) as SVGGElement;
  // The copy is a drawing: it sits where the picture starts and cannot be reached or focused.
  copy.removeAttribute('transform');
  for (const node of [copy, ...copy.querySelectorAll('[id], [tabindex]')]) {
    node.removeAttribute('id');
    node.removeAttribute('tabindex');
  }
  svg.append(copy);
  return svg;
}

/** Builds every block of the reference in a workspace nobody sees, and keeps a picture and the facts. */
function drawAll(): { skin: string; drawn: Map<ReferenceEntry, Drawn> } {
  const stage = h('div', { class: 'ref-stage', 'aria-hidden': 'true' });
  document.body.append(stage);
  const editor = Blockly.common.getMainWorkspace();
  const workspace = Blockly.inject(stage, {
    renderer: 'zelos',
    theme,
    sounds: false,
    trashcan: false,
    zoom: { controls: false, wheel: false, startScale: 1 },
    move: { scrollbars: false, drag: false, wheel: false },
  });
  // Blockly styles blocks through these classes on the element around them.
  const skin = [...workspace.getInjectionDiv().classList].filter((name) => /-(renderer|theme)$/.test(name)).join(' ');
  // Blocks with one option added are only asked what they write, so they need no drawing.
  const unseen = new Blockly.Workspace();
  const drawn = new Map<ReferenceEntry, Drawn>();
  const add = (state: BlockState) => Blockly.serialization.blocks.append(state, workspace, { recordUndo: false }) as Blockly.BlockSvg;

  Blockly.Events.disable();
  try {
    const built: [ReferenceEntry, Blockly.BlockSvg][] = [];
    for (const entry of REFERENCE.flatMap((section) => section.entries)) {
      const block = add(entry.state).getDescendants(false).find((inside) => inside.type === entry.type);
      if (block) built.push([entry, block]);
    }
    Blockly.renderManagement.triggerQueuedRenders(workspace);
    for (const [entry, block] of built) {
      const forms = formsOf(block);
      const options = entry.variant
        ? []
        : optionSpecs(entry.type).map((spec): OptionRow => {
            const id = (spec.repeat ?? 1) > 1 ? `${spec.key}#1` : spec.key;
            const fields = { ...entry.state.fields, ...OPTION_FIELDS[`${entry.type}/${spec.key}`] };
            const state: BlockState = { ...entry.state, fields, extraState: { options: [id] } };
            const withOption = formsOf(Blockly.serialization.blocks.append(state, unseen, { recordUndo: false }));
            return { menu: spec.menu, plain: added(forms.plain, withOption.plain), script: added(forms.script, withOption.script) };
          });
      drawn.set(entry, {
        picture: picture(block),
        words: block.toString().replace(/\s*\+ option/g, '').replace(/\s+/g, ' ').trim(),
        about: block.getTooltip(),
        forms,
        options,
      });
    }
  } finally {
    Blockly.Events.enable();
    unseen.dispose();
    workspace.dispose();
    stage.remove();
    if (editor) Blockly.common.setMainWorkspace(editor);
  }
  return { skin, drawn };
}

const tag = (shape: 'plain' | 'script') => h('span', { class: `ref-tag ref-tag-${shape}` }, shape);

function line(shape: 'plain' | 'script', code: string | null): HTMLElement | null {
  return code === null ? null : h('div', { class: 'ref-line' }, tag(shape), h('code', null, code));
}

function optionRow(option: OptionRow): HTMLElement {
  const same = option.plain !== null && option.plain === option.script;
  return h('li', { class: 'ref-option' },
    h('span', { class: 'ref-option-name' }, option.menu),
    same
      ? h('div', { class: 'ref-line' }, tag('plain'), tag('script'), h('code', null, option.plain))
      : [line('plain', option.plain), line('script', option.script)],
  );
}

function entryElement(entry: ReferenceEntry, facts: Drawn, skin: string): HTMLElement {
  const { forms, options } = facts;
  // A block with a plain line and no script line cannot be in a move that has logic.
  const plainOnly = forms.plain !== null && forms.script === null && !HAT_TYPES.includes(entry.type);
  return h('article', { class: 'ref-entry' },
    h('div', { class: `ref-pic ${skin}`, role: 'img', 'aria-label': facts.words }, facts.picture),
    entry.variant ? null : h('p', { class: 'ref-about' }, facts.about),
    entry.note ? h('p', { class: 'ref-note' }, entry.note) : null,
    h('div', { class: 'ref-lines' },
      line('plain', forms.plain),
      line('script', forms.script),
      plainOnly ? h('span', { class: 'ref-badge' }, 'no logic yet') : null,
    ),
    options.length
      ? h('div', { class: 'ref-options' }, h('div', { class: 'ref-options-title' }, '“+ option” can add'), h('ul', null, options.map(optionRow)))
      : null,
  );
}

function swatch(section: ReferenceSection): HTMLElement {
  const shape = section.id === 'starts' ? ' ref-swatch-hat' : section.id === 'info' ? ' ref-swatch-round' : '';
  return h('span', { class: `ref-swatch${shape}`, style: `background: ${section.color}` });
}

export function openReference(options: ReferenceOptions): { close(): void } {
  const { skin, drawn } = drawAll();
  const sections = REFERENCE.map((section) =>
    h('section', { class: 'ref-section', 'aria-labelledby': `ref-${section.id}` },
      h('div', { class: 'ref-section-head' }, swatch(section), h('h2', { id: `ref-${section.id}` }, section.name), h('span', { class: 'ref-section-about' }, section.about)),
      section.entries.map((entry) => {
        const facts = drawn.get(entry);
        return facts ? entryElement(entry, facts, skin) : null;
      }),
    ),
  );

  const scroller = h('div', { class: 'ref-scroll', tabindex: '-1', autofocus: true },
    h('div', { class: 'ref-intro' },
      h('h1', { id: 'reference-title' }, 'All blocks'),
      h('p', { class: 'ref-lead' }, 'What every block does, and the line it writes into the move file, so you can always see what the game will read.'),
      h('div', { class: 'ref-key' },
        h('p', null,
          'A move file comes in two shapes. A ', tag('plain'), ' move is a list of the game’s built-in effects. A ', tag('script'),
          ' is a small program, for moves that need more than that. Move Lab picks the shape for you: a move stays plain while every block in it has a plain line, and becomes a script as soon as one block has only a script line.',
        ),
        h('p', null,
          h('span', { class: 'ref-badge' }, 'no logic yet'),
          ' marks the blocks that have no script line for now. They cannot share a move with a block that needs a script.',
        ),
      ),
      h('nav', { class: 'ref-nav', 'aria-label': 'Block categories' },
        REFERENCE.map((section, index) =>
          h('button', { type: 'button', class: 'pill ref-jump', onclick: () => sections[index].scrollIntoView({ block: 'start' }) }, swatch(section), section.name),
        ),
      ),
    ),
    h('div', { class: 'ref-columns' }, sections),
  );

  const dialog = h('dialog', { class: 'reference', 'aria-labelledby': 'reference-title' },
    h('header', { class: 'topbar' },
      brand(),
      h('span', { class: 'ref-crumb-sep', 'aria-hidden': 'true' }, '/'),
      h('span', { class: 'ref-crumb' }, 'All blocks'),
      h('button', { type: 'button', class: 'top-link ref-back', onclick: () => options.onClose() }, icon('back', 15), `Back to ${options.moveName}`),
    ),
    scroller,
  );
  // Escape leaves the page the same way the button does, so the address stays in step.
  dialog.addEventListener('cancel', (event) => {
    event.preventDefault();
    options.onClose();
  });
  document.body.append(dialog);
  dialog.showModal();

  return {
    close() {
      dialog.close();
      dialog.remove();
    },
  };
}
