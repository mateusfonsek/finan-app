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

/** State already synced to a discovery — the real starting point. */
function withDiscovery(hash = "h1") {
  return syncHash(initialToastState, hash);
}

describe("phaseOf", () => {
  it("hides when there is no discovery", () => {
    expect(phaseOf(initialToastState, false)).toBe("hidden");
  });

  it("starts expanded — this is the moment the app announces what arrived", () => {
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

  it("never hides on its own: with no import running it is expanded or collapsed", () => {
    // The regression that caused this bug — the notification vanished and the
    // discovery stayed unreachable until the app was reopened.
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
    // Without this, any re-render would reopen the pill the user closed.
    const s = collapse(withDiscovery("h1"));
    expect(phaseOf(syncHash(s, "h1"), false)).toBe("collapsed");
  });

  it("perder a descoberta esconde", () => {
    expect(phaseOf(syncHash(withDiscovery(), null), false)).toBe("hidden");
  });
});

describe("autoCollapseArmed", () => {
  it("arms on a freshly arrived discovery", () => {
    expect(autoCollapseArmed(withDiscovery(), false, false)).toBe(true);
  });

  it("does not arm under the cursor — collapsing mid-read is hostile", () => {
    expect(autoCollapseArmed(withDiscovery(), false, true)).toBe(false);
  });

  it("does not arm once it has already collapsed on its own", () => {
    expect(autoCollapseArmed(autoCollapse(withDiscovery()), false, false)).toBe(false);
  });

  it("does not arm after the user reopened it: the shape is their choice now", () => {
    const s = expand(autoCollapse(withDiscovery()));
    expect(autoCollapseArmed(s, false, false)).toBe(false);
  });

  it("does not arm while an import is running", () => {
    expect(autoCollapseArmed(withDiscovery(), true, false)).toBe(false);
  });

  it("does not arm with no discovery", () => {
    expect(autoCollapseArmed(initialToastState, false, false)).toBe(false);
  });

  it("volta a armar quando chega uma descoberta diferente", () => {
    const s = syncHash(expand(autoCollapse(withDiscovery("h1"))), "h2");
    expect(autoCollapseArmed(s, false, false)).toBe(true);
  });
});
