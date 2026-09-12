import { describe, it, expect } from "vitest";
import { placePopover } from "./placement";

const VIEW = { width: 1200, height: 800 };
const WIDTH = 320;

/** A chip somewhere in the middle of the grid. */
const mid = { top: 300, bottom: 320, left: 400 };

describe("placePopover", () => {
  it("hangs below the anchor when there is room", () => {
    const p = placePopover(mid, VIEW, WIDTH);
    expect(p.up).toBe(false);
    expect(p.offset).toBe(326);
  });

  /** The reported bug: a chip on the month's last row opened a panel whose
   *  actions were below the screen, where nothing can scroll them back. */
  it("flips above the anchor when the room below is cramped", () => {
    const p = placePopover({ top: 700, bottom: 720, left: 400 }, VIEW, WIDTH);
    expect(p.up).toBe(true);
    expect(p.offset).toBe(106);
    expect(p.maxHeight).toBe(686);
  });

  it("stays below when both sides are cramped but below is roomier", () => {
    const p = placePopover({ top: 120, bottom: 140, left: 400 }, { width: 1200, height: 340 }, WIDTH);
    expect(p.up).toBe(false);
    expect(p.maxHeight).toBeGreaterThan(0);
  });

  it("never claims more height than the side it landed on has", () => {
    for (let top = 0; top <= 780; top += 20) {
      const p = placePopover({ top, bottom: top + 20, left: 400 }, VIEW, WIDTH);
      const edge = p.up ? VIEW.height - p.offset : p.offset;
      const far = p.up ? edge - p.maxHeight : edge + p.maxHeight;
      expect(far).toBeGreaterThanOrEqual(0);
      expect(far).toBeLessThanOrEqual(VIEW.height);
    }
  });

  it("keeps the panel inside both side edges", () => {
    expect(placePopover({ ...mid, left: 0 }, VIEW, WIDTH).left).toBe(8);
    expect(placePopover({ ...mid, left: 1190 }, VIEW, WIDTH).left).toBe(872);
  });
});
