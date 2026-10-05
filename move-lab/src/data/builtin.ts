// The move files shipped with the engine (../../../moves), bundled as text so they can be written
// into the engine's in-memory file system. Keys are file names such as `tackle.lua`.
const modules = import.meta.glob('../../../moves/*.lua', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;

export const BUILTIN_FILES: Record<string, string> = Object.fromEntries(
  Object.entries(modules).map(([path, source]) => [path.slice(path.lastIndexOf('/') + 1), source]),
);
