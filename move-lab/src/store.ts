// The project lives in this browser's local storage: the moves being edited and the battle setup.
import type { BattleSetup, MoveDoc } from './model.ts';

export interface Project {
  version: 1;
  docs: MoveDoc[];
  /** uid of the open move. */
  current: string;
  setup: BattleSetup;
  luaOpen: boolean;
}

const KEY = 'move-lab:project:v1';

export function loadProject(): Project | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const project = JSON.parse(raw) as Project;
    if (project?.version !== 1 || !Array.isArray(project.docs) || project.docs.length === 0 || !project.setup) return null;
    return project;
  } catch {
    return null;
  }
}

/** Returns false when the browser refused to store the project (private mode, full storage). */
export function saveProject(project: Project): boolean {
  try {
    localStorage.setItem(KEY, JSON.stringify(project));
    return true;
  } catch {
    return false;
  }
}
