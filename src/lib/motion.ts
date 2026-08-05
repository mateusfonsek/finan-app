/**
 * Movimento da interface.
 *
 * Two HIG rules drive this file:
 *
 * 1. Enter and exit follow the SAME path. What slides in from the right leaves
 *    to the right; what grows from a point shrinks back into it. That is why
 *    each transition here is one function used in both directions.
 * 2. "Reduce motion" is not the absence of response, it is response without
 *    displacement. Every transition degrades to a short fade.
 */
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

/** Critically damped spring curve (no overshoot) — the house default. */
export const SNAP = cubicOut;

export const DUR = {
  fast: 140,
  base: 220,
  slow: 320,
  sheet: 340,
} as const;

export function reducedMotion(): boolean {
  if (typeof window === "undefined" || !window.matchMedia) return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function fadeOnly(duration: number): TransitionConfig {
  return { duration, easing: SNAP, css: (t) => `opacity: ${t}` };
}

/** Scrim dimming the background of a modal task. */
export function scrim(_node: Element): TransitionConfig {
  return fadeOnly(DUR.base);
}

/**
 * Popover/menu: grows from its origin rather than its own centre, making the
 * spatial relationship between trigger and content obvious. `origin` is a CSS
 * `transform-origin`.
 */
export function popover(
  node: Element,
  { origin = "top right" }: { origin?: string } = {},
): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.fast);
  (node as HTMLElement).style.transformOrigin = origin;
  return {
    duration: DUR.base,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: scale(${1 - u * 0.05}) translateY(${-u * 5}px)`,
  };
}

/** Modal dialog: materializes in place, no sliding. */
export function dialog(_node: Element): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.base);
  return {
    duration: DUR.slow,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: scale(${1 - u * 0.06})`,
  };
}

/** Side panel: enters from an edge and leaves by the same edge. */
export function sheet(
  _node: Element,
  { side = "right" }: { side?: "right" | "left" } = {},
): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.base);
  const sign = side === "right" ? 1 : -1;
  return {
    duration: DUR.sheet,
    easing: SNAP,
    css: (t, u) => `opacity: ${Math.min(1, t * 2)}; transform: translateX(${sign * u * 100}%)`,
  };
}

/** Notification: rises from the bottom corner with a slight grow. */
export function toast(_node: Element): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.base);
  return {
    duration: DUR.slow,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * 12}px) scale(${1 - u * 0.04})`,
  };
}

/** A block appearing in page flow (a result, an expanded form). */
export function rise(_node: Element, { delay = 0 } = {}): TransitionConfig {
  if (reducedMotion()) return { ...fadeOnly(DUR.base), delay };
  return {
    duration: DUR.base,
    delay,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * 6}px)`,
  };
}
