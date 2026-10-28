export type ShortcutContext = {
  page?: string; inputFocused?: boolean; coverflowFocused?: boolean;
  coverflowHovered?: boolean; separatorFocused?: boolean; nativeControl?: boolean; busy?: boolean;
};
export type ShortcutAction = (event: KeyboardEvent, phase: 'press' | 'release') => unknown;
export type ShortcutBinding = { key: string; action: string | ShortcutAction; scope?: string; hold?: boolean; repeat?: boolean; input?: boolean };
type Options = { getContext?: (event?: KeyboardEvent) => ShortcutContext; actions?: Record<string, ShortcutAction>; onError?: (error: unknown) => void };
type Scope = { name: string; bindings: ShortcutBinding[]; exclusive: boolean };

// The frozen module and its separator keep their original event ownership.
export const coverflowConflictTable = Object.freeze([
  { key: 'Space', owner: 'original-module', policy: 'reserve-on-focus-or-hover' },
  { key: 'ArrowLeft', owner: 'original-module-and-host', policy: 'capture-first-then-host-once' },
  { key: 'ArrowRight', owner: 'original-module-and-host', policy: 'capture-first-then-host-once' },
  { key: 'Escape', owner: 'original-module', policy: 'release-then-global' },
  ...['ArrowUp', 'ArrowDown', 'Home', 'End'].map(key => ({ key, owner: 'original-separator', policy: 'native-owner' })),
  { key: 'Enter', owner: 'original-button', policy: 'native-owner' },
  ...['1', '2', '3', '4', '5', 'r'].map(key => ({ key, owner: 'editor', policy: 'no-original-conflict' })),
]);
export const shortcutBindings: ShortcutBinding[] = [
  { key: 'Mod+k', action: 'composer.focus', scope: 'global' },
  { key: 'Mod+j', action: 'chat.toggle', scope: 'global' },
  { key: 'Mod+z', action: 'history.undo', scope: 'global' },
  { key: 'Mod+Shift+z', action: 'history.redo', scope: 'global' },
  { key: 'Escape', action: 'task.escape', scope: 'global' },
  { key: 'Mod+s', action: 'editor.save', scope: 'editor' },
  { key: 'Mod+a', action: 'library.selectAll', scope: 'library' },
  { key: 'Delete', action: 'library.trash', scope: 'library' },
  { key: 'Enter', action: 'library.open', scope: 'library' },
  ...['1', '2', '3', '4', '5'].map(key => ({ key, action: `editor.ratio.${key}`, scope: 'editor' })),
  { key: 'r', action: 'editor.ratio.swap', scope: 'editor' },
  { key: 'Space', action: 'editor.compare', scope: 'editor', hold: true },
  { key: 'Backslash', action: 'editor.compare', scope: 'editor', hold: true },
  { key: 'ArrowLeft', action: 'editor.previous', scope: 'editor', repeat: true },
  { key: 'ArrowRight', action: 'editor.next', scope: 'editor', repeat: true },
  { key: '[', action: 'editor.mask.smaller', scope: 'editor', repeat: true },
  { key: ']', action: 'editor.mask.larger', scope: 'editor', repeat: true },
  { key: 'b', action: 'editor.brush', scope: 'editor' },
  { key: 'q', action: 'editor.tool.previous', scope: 'editor' },
  { key: 'e', action: 'editor.tool.next', scope: 'editor' },
  { key: '0', action: 'editor.fit', scope: 'editor' },
];
const safeInputKeys = new Set(['Escape', 'Mod+k', 'Mod+j']);
export function shortcutKey(event: Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey'>) {
  let key = event.key;
  if (key === ' ' || key === 'Spacebar' || event.code === 'Space') key = 'Space';
  else if (event.code === 'Backslash' || key === '\\') key = 'Backslash';
  else if (key === 'Esc') key = 'Escape';
  else if (key.length === 1) key = key.toLowerCase();
  return `${event.ctrlKey || event.metaKey ? 'Mod+' : ''}${event.altKey ? 'Alt+' : ''}${event.shiftKey ? 'Shift+' : ''}${key}`;
}
export function validateBindings(bindings: readonly ShortcutBinding[]) {
  const seen = new Set<string>();
  for (const binding of bindings) {
    const signature = `${binding.scope || 'global'}:${binding.key}`;
    if (seen.has(signature)) throw Error(`快捷键冲突：${signature}`);
    seen.add(signature);
  }
}
export function createShortcutManager(initial: Options = {}, bindings = shortcutBindings) {
  validateBindings(bindings);
  let options = initial;
  const scopes: Scope[] = [], held = new Map<string, { binding: ShortcutBinding; action: ShortcutAction; event: KeyboardEvent }>();
  const report = (error: unknown) => options.onError?.(error);
  function invoke(action: ShortcutAction, event: KeyboardEvent, phase: 'press' | 'release') {
    try { const result = action(event, phase);if (result && typeof (result as Promise<unknown>).then === 'function') Promise.resolve(result).catch(report);return result; }
    catch (error) { report(error);return undefined; }
  }
  function consume(event: KeyboardEvent) { event.preventDefault();event.stopPropagation(); }
  function releaseAll() {
    const previous = [...held.values()];held.clear();
    for (const value of previous) invoke(value.action, value.event, 'release');
  }
  function allowed(binding: ShortcutBinding, key: string, context: ShortcutContext) {
    const inputNavigation = binding.input && ['ArrowUp', 'ArrowDown', 'Enter', 'Tab'].includes(key);
    if (context.inputFocused && !safeInputKeys.has(key) && !inputNavigation) return false;
    if (context.busy && !safeInputKeys.has(key)) return false;
    if (key === 'Space' && (context.coverflowFocused || context.coverflowHovered || context.nativeControl)) return false;
    if (context.separatorFocused && ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End', 'Space', 'Enter'].includes(key)) return false;
    if ((context.coverflowFocused || context.nativeControl) && key === 'Enter') return false;
    return true;
  }
  return {
    configure(value: Options) { releaseAll();options = value; },
    get scopes() { return scopes.map(scope => scope.name); },
    get heldCount() { return held.size; },
    pushScope(name: string, scopedBindings: ShortcutBinding[], { exclusive = true } = {}) {
      validateBindings(scopedBindings);releaseAll();
      const scope = { name, bindings: scopedBindings, exclusive };scopes.push(scope);
      return () => { const index = scopes.indexOf(scope);if (index >= 0) { scopes.splice(index, 1);releaseAll(); } };
    },
    handleKeyDown(event: KeyboardEvent) {
      if (event.defaultPrevented || event.isComposing || event.keyCode === 229 || event.getModifierState?.('AltGraph')) return null;
      const key = shortcutKey(event), context = options.getContext?.(event) || {};
      if (context.inputFocused) releaseAll();
      let binding: ShortcutBinding | undefined;
      if (context.inputFocused && ['Mod+k', 'Mod+j'].includes(key)) binding = bindings.find(value => value.scope === 'global' && value.key === key);
      else for (const scope of [...scopes].reverse()) { binding = scope.bindings.find(value => value.key === key);if (binding) break; }
      if (!binding && (!scopes.some(scope => scope.exclusive) || safeInputKeys.has(key))) {
        binding = bindings.find(value => value.scope === context.page && value.key === key) || bindings.find(value => value.scope === 'global' && value.key === key);
      }
      if (!binding || !allowed(binding, key, context)) return null;
      const action = typeof binding.action === 'function' ? binding.action : options.actions?.[binding.action];
      if (!action) return null;
      const code = event.code || key;
      if (event.repeat && !binding.repeat) { consume(event);return binding.action; }
      if (binding.hold && held.has(code)) { consume(event);return binding.action; }
      if (invoke(action, event, 'press') === false) return null;
      if (binding.hold) held.set(code, { binding, action, event });
      consume(event);return binding.action;
    },
    handleKeyUp(event: KeyboardEvent) {
      const code = event.code || shortcutKey(event), value = held.get(code);if (!value) return null;
      held.delete(code);invoke(value.action, event, 'release');consume(event);return value.binding.action;
    },
    releaseAll,
    dispose() { releaseAll();scopes.splice(0); },
  };
}

type Handlers = { keydown?: (event: KeyboardEvent) => void; keyup?: (event: KeyboardEvent) => void };
const targets = new Map<Element, Handlers>();
let appOptions: Options | null = null, stop: (() => void) | null = null;
function eventElement(event?: Event): Element | null {
  const path = event?.composedPath() || [];
  return path.find(node => node instanceof Element) as Element || document.activeElement;
}
export function isEditableTarget(target: Element | null) {
  if (!target) return false;
  if (target.closest('input,textarea,select,[role="textbox"]')) return true;
  return (target as HTMLElement).isContentEditable === true;
}
function domContext(event?: KeyboardEvent): ShortcutContext {
  const element = eventElement(event);
  return {
    ...appOptions?.getContext?.(event), inputFocused: isEditableTarget(element),
    coverflowFocused: !!element?.closest('.cf-root'),
    coverflowHovered: !!document.querySelector('.cf-root:not(.collapsed):hover'),
    separatorFocused: !!element?.closest('[role="separator"]'),
    nativeControl: !!element?.closest('button,a[href],[role="button"],[role="slider"],[role="separator"]'),
  };
}
const domManager = createShortcutManager({ getContext: domContext });
function listen() {
  if (stop) return;
  const dispatch = (event: KeyboardEvent) => {
    if (event.type === 'keyup' && domManager.handleKeyUp(event)) return;
    if (event.defaultPrevented || event.isComposing || event.keyCode === 229) return;
    for (const node of event.composedPath()) {
      targets.get(node as Element)?.[event.type as 'keydown' | 'keyup']?.(event);
      if (event.cancelBubble || event.defaultPrevented) return;
    }
    if (event.type === 'keydown') domManager.handleKeyDown(event);
  };
  const release = () => domManager.releaseAll();
  const hide = () => { if (document.hidden) release(); };
  const focus = () => { const context = domContext();if (context.inputFocused || context.coverflowFocused || context.nativeControl) release(); };
  document.addEventListener('keydown', dispatch);document.addEventListener('keyup', dispatch);
  document.addEventListener('focusin', focus);document.addEventListener('visibilitychange', hide);window.addEventListener('blur', release);
  stop = () => {
    release();document.removeEventListener('keydown', dispatch);document.removeEventListener('keyup', dispatch);
    document.removeEventListener('focusin', focus);document.removeEventListener('visibilitychange', hide);window.removeEventListener('blur', release);stop = null;
  };
}
function idle() { if (!targets.size && !appOptions && !domManager.scopes.length) stop?.(); }
export function installApplicationShortcuts(options: Options) {
  appOptions = options;domManager.configure({ ...options, getContext: domContext });listen();
  return { releaseHeld: () => domManager.releaseAll(), destroy() { if (appOptions === options) { appOptions = null;domManager.configure({ getContext: domContext });idle(); } } };
}
export function keyboard(node: Element, handlers: Handlers) {
  targets.set(node, handlers);listen();
  return { update(value: Handlers) { targets.set(node, value); }, destroy() { targets.delete(node);idle(); } };
}
export function shortcutScope(_node: Element, value: { name: string; bindings: ShortcutBinding[]; exclusive?: boolean }) {
  let pop = domManager.pushScope(value.name, value.bindings, { exclusive: value.exclusive ?? true });listen();
  return {
    update(next: typeof value) { pop();pop = domManager.pushScope(next.name, next.bindings, { exclusive: next.exclusive ?? true }); },
    destroy() { pop();idle(); },
  };
}
