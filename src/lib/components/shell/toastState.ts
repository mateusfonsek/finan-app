/**
 * State machine for the "statement found" notification.
 *
 * Lives outside the `.svelte` because it is the only part with real rules — and
 * because the previous version had a bug no test caught: the notification
 * vanished on its own after 8s and the discovery became unreachable, since the
 * sidebar badge only counts and reopens nothing. Missing the window meant
 * restarting the app.
 *
 * The core rule: **this never hides on its own**. It shrinks. It leaves the
 * screen only when the discovery is resolved (review/ignore, which removes it
 * from the store) or while an import is running.
 */

/** `hidden` happens only with no discovery or during an import. */
export type ToastPhase = "expanded" | "collapsed" | "hidden";

export type ToastState = {
  /** The discovery this state belongs to. Changing it resets. */
  hash: string | null;
  /** What the user chose for THIS discovery. `null` means untouched. */
  manual: "expanded" | "collapsed" | null;
  /** The automatic shrink already happened for this discovery. */
  autoCollapsed: boolean;
};

export const initialToastState: ToastState = {
  hash: null,
  manual: null,
  autoCollapsed: false,
};

/**
 * Aligns state with the focused discovery. A different discovery starts from
 * scratch — expanded — because it is another file with another name and count:
 * new information deserves to be shown, not to inherit the pill the user
 * collapsed for the previous one.
 *
 * The same discovery is a deliberate no-op: the component calls this on every
 * render, and resetting here would reopen a just-collapsed pill on its own.
 */
export function syncHash(state: ToastState, hash: string | null): ToastState {
  if (state.hash === hash) return state;
  return { hash, manual: null, autoCollapsed: false };
}

/** The user collapsed it using the notification's own control. */
export function collapse(state: ToastState): ToastState {
  return { ...state, manual: "collapsed" };
}

/** The user clicked the pill to reopen. */
export function expand(state: ToastState): ToastState {
  return { ...state, manual: "expanded" };
}

/** Time ran out with no interaction. */
export function autoCollapse(state: ToastState): ToastState {
  return { ...state, autoCollapsed: true };
}

export function phaseOf(state: ToastState, suppressed: boolean): ToastPhase {
  if (state.hash === null || suppressed) return "hidden";
  if (state.manual !== null) return state.manual;
  return state.autoCollapsed ? "collapsed" : "expanded";
}

/**
 * The automatic shrink applies only to the first appearance, when the app is
 * *notifying*. After the user interacts, the shape is their choice.
 *
 * `hovering` disarms it: collapsing under the cursor mid-read is the most
 * common notification defect.
 */
export function autoCollapseArmed(
  state: ToastState,
  suppressed: boolean,
  hovering: boolean,
): boolean {
  if (hovering) return false;
  if (state.manual !== null || state.autoCollapsed) return false;
  return phaseOf(state, suppressed) === "expanded";
}
