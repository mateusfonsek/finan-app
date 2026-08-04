import { describe, it, expect } from "vitest";
import {
  autoCollapse,
  autoCollapseArmed,
  collapse,
  expand,
  initialToastState,
  phaseOf,
  syncHash,
} from "./toastState";

/** Estado já sincronizado com uma descoberta, que é o ponto de partida real. */
function withDiscovery(hash = "h1") {
  return syncHash(initialToastState, hash);
}

describe("phaseOf", () => {
  it("esconde quando não há descoberta", () => {
    expect(phaseOf(initialToastState, false)).toBe("hidden");
  });

  it("nasce expandida — é o momento em que o app avisa o que chegou", () => {
    expect(phaseOf(withDiscovery(), false)).toBe("expanded");
  });

  it("esconde durante um import, mesmo com descoberta pendente", () => {
    expect(phaseOf(withDiscovery(), true)).toBe("hidden");
  });

  it("encolhe quando o timer dispara", () => {
    expect(phaseOf(autoCollapse(withDiscovery()), false)).toBe("collapsed");
  });

  it("encolhe quando o usuário clica no controle", () => {
    expect(phaseOf(collapse(withDiscovery()), false)).toBe("collapsed");
  });

  it("reabre quando o usuário clica na pastilha", () => {
    const s = expand(autoCollapse(withDiscovery()));
    expect(phaseOf(s, false)).toBe("expanded");
  });

  it("nunca fica escondida por conta própria: sem import, sempre é expanded ou collapsed", () => {
    // É a regressão que originou este bug — a notificação sumia e a descoberta
    // ficava inalcançável até reabrir o app.
    const estados = [
      withDiscovery(),
      autoCollapse(withDiscovery()),
      collapse(withDiscovery()),
      expand(autoCollapse(withDiscovery())),
    ];
    for (const s of estados) {
      expect(phaseOf(s, false)).not.toBe("hidden");
    }
  });
});

describe("syncHash", () => {
  it("descoberta nova volta a expandir e limpa o que o usuário fez na anterior", () => {
    const s = collapse(withDiscovery("h1"));
    const nova = syncHash(s, "h2");
    expect(phaseOf(nova, false)).toBe("expanded");
  });

  it("re-sincronizar a MESMA descoberta preserva o estado", () => {
    // Sem isso, qualquer re-render reabriria a pastilha que o usuário fechou.
    const s = collapse(withDiscovery("h1"));
    expect(phaseOf(syncHash(s, "h1"), false)).toBe("collapsed");
  });

  it("perder a descoberta esconde", () => {
    expect(phaseOf(syncHash(withDiscovery(), null), false)).toBe("hidden");
  });
});

describe("autoCollapseArmed", () => {
  it("arma numa descoberta recém-chegada", () => {
    expect(autoCollapseArmed(withDiscovery(), false, false)).toBe(true);
  });

  it("não arma sob o cursor — encolher enquanto se lê é hostil", () => {
    expect(autoCollapseArmed(withDiscovery(), false, true)).toBe(false);
  });

  it("não arma depois de já ter encolhido sozinha", () => {
    expect(autoCollapseArmed(autoCollapse(withDiscovery()), false, false)).toBe(false);
  });

  it("não arma depois que o usuário reabriu: aí foi escolha dele", () => {
    const s = expand(autoCollapse(withDiscovery()));
    expect(autoCollapseArmed(s, false, false)).toBe(false);
  });

  it("não arma durante um import", () => {
    expect(autoCollapseArmed(withDiscovery(), true, false)).toBe(false);
  });

  it("não arma sem descoberta", () => {
    expect(autoCollapseArmed(initialToastState, false, false)).toBe(false);
  });

  it("volta a armar quando chega uma descoberta diferente", () => {
    const s = syncHash(expand(autoCollapse(withDiscovery("h1"))), "h2");
    expect(autoCollapseArmed(s, false, false)).toBe(true);
  });
});
