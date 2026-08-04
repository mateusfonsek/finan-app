/**
 * Máquina de estados da notificação de extrato encontrado.
 *
 * Vive fora do `.svelte` porque é a única parte com regra de verdade — e
 * porque a versão anterior desta tela tinha um bug que nenhum teste pegava:
 * a notificação sumia sozinha depois de 8s e a descoberta ficava
 * inalcançável, já que o badge da sidebar só conta, não reabre nada. Quem
 * perdesse a janela precisava fechar e reabrir o app.
 *
 * A regra central, portanto: **isto nunca se esconde por conta própria**. Ela
 * encolhe. Só sai da tela quando a descoberta é resolvida (Revisar/ignorar,
 * o que a tira da store) ou quando um import está em andamento.
 */

/** `hidden` só acontece por ausência de descoberta ou por import em curso. */
export type ToastPhase = "expanded" | "collapsed" | "hidden";

export type ToastState = {
  /** Descoberta a que este estado pertence. Trocou, reseta. */
  hash: string | null;
  /** O que o usuário escolheu para ESTA descoberta. `null` = não mexeu. */
  manual: "expanded" | "collapsed" | null;
  /** O encolhimento automático já aconteceu para esta descoberta. */
  autoCollapsed: boolean;
};

export const initialToastState: ToastState = {
  hash: null,
  manual: null,
  autoCollapsed: false,
};

/**
 * Alinha o estado com a descoberta em foco. Descoberta diferente começa do
 * zero — expandida —, porque é outro arquivo, com outro nome e outra
 * contagem: informação nova merece ser mostrada, não herdar a pastilha que o
 * usuário fechou para a anterior.
 *
 * Mesma descoberta é no-op de propósito: o componente chama isto a cada
 * render, e resetar aqui reabriria sozinha a pastilha recém-fechada.
 */
export function syncHash(state: ToastState, hash: string | null): ToastState {
  if (state.hash === hash) return state;
  return { hash, manual: null, autoCollapsed: false };
}

/** Usuário encolheu no controle da própria notificação. */
export function collapse(state: ToastState): ToastState {
  return { ...state, manual: "collapsed" };
}

/** Usuário clicou na pastilha para reabrir. */
export function expand(state: ToastState): ToastState {
  return { ...state, manual: "expanded" };
}

/** O tempo acabou sem interação. */
export function autoCollapse(state: ToastState): ToastState {
  return { ...state, autoCollapsed: true };
}

export function phaseOf(state: ToastState, suppressed: boolean): ToastPhase {
  if (state.hash === null || suppressed) return "hidden";
  if (state.manual !== null) return state.manual;
  return state.autoCollapsed ? "collapsed" : "expanded";
}

/**
 * O encolhimento automático vale só para a primeira aparição, que é quando o
 * app está *avisando*. Depois que o usuário mexeu, a forma é escolha dele.
 *
 * `hovering` desarma: encolher debaixo do cursor, no meio da leitura, é o
 * defeito mais comum de notificação.
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
