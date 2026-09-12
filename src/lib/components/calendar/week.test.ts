import { describe, it, expect } from "vitest";
import { leadingBlanks, weekdayOrder } from "./week";

describe("leadingBlanks", () => {
  it("is zero when the month starts on the week's first day", () => {
    expect(leadingBlanks(0, 0)).toBe(0);
    expect(leadingBlanks(1, 1)).toBe(0);
  });

  it("counts the days from the week's start to the month's first day", () => {
    // Sunday-first week, month starting on a Wednesday.
    expect(leadingBlanks(3, 0)).toBe(3);
  });

  /** The case the `+ 7` exists for: a month starting on Sunday in a week that
   *  starts on Monday needs six blanks, not minus one. */
  it("wraps instead of going negative when the month starts before the week does", () => {
    expect(leadingBlanks(0, 1)).toBe(6);
    expect(leadingBlanks(2, 6)).toBe(3);
  });

  it("never exceeds a week", () => {
    for (let start = 0; start < 7; start++) {
      for (let first = 0; first < 7; first++) {
        const blanks = leadingBlanks(start, first);
        expect(blanks).toBeGreaterThanOrEqual(0);
        expect(blanks).toBeLessThan(7);
      }
    }
  });
});

describe("weekdayOrder", () => {
  it("is the plain Sunday-first order when the week starts on Sunday", () => {
    expect(weekdayOrder(0)).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it("rotates so the locale's first day leads and Sunday closes the week", () => {
    expect(weekdayOrder(1)).toEqual([1, 2, 3, 4, 5, 6, 0]);
  });

  it("covers every weekday exactly once, whatever the start", () => {
    for (let first = 0; first < 7; first++) {
      expect([...weekdayOrder(first)].sort()).toEqual([0, 1, 2, 3, 4, 5, 6]);
    }
  });
});
