/**
 * Movimento da interface.
 *
 * Duas regras da HIG guiam este arquivo:
 *
 * 1. Entrada e saída percorrem o MESMO caminho. O que entra pela direita sai
 *    pela direita; o que cresce a partir de um ponto encolhe de volta pra ele.
 *    Por isso tudo aqui é uma transição só, usada nos dois sentidos.
 * 2. "Reduzir movimento" não é ausência de resposta — é resposta sem
 *    deslocamento. Cada transição degrada pra um esmaecer curto.
 */
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

/** Curva de mola criticamente amortecida (sem overshoot) — o padrão da casa. */
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

/** Véu que escurece o fundo de uma tarefa modal. */
export function scrim(_node: Element): TransitionConfig {
  return fadeOnly(DUR.base);
}

/**
 * Popover/menu: cresce a partir da origem, não do próprio centro — a relação
 * espacial entre o botão e o que ele abriu fica óbvia.
 * `origin` é um `transform-origin` CSS.
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

/** Diálogo modal: materializa no centro, sem deslizar. */
export function dialog(_node: Element): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.base);
  return {
    duration: DUR.slow,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: scale(${1 - u * 0.06})`,
  };
}

/** Painel lateral: entra pela borda e sai pela mesma borda. */
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

/** Notificação: sobe do canto inferior com um leve crescimento. */
export function toast(_node: Element): TransitionConfig {
  if (reducedMotion()) return fadeOnly(DUR.base);
  return {
    duration: DUR.slow,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * 12}px) scale(${1 - u * 0.04})`,
  };
}

/** Bloco que aparece no fluxo da página (resultado, formulário expandido). */
export function rise(_node: Element, { delay = 0 } = {}): TransitionConfig {
  if (reducedMotion()) return { ...fadeOnly(DUR.base), delay };
  return {
    duration: DUR.base,
    delay,
    easing: SNAP,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * 6}px)`,
  };
}
