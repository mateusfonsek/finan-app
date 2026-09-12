import { describe, it, expect } from "vitest";
import { billState, daysOverdue, dueDateOf } from "./bill";
import type { CalendarEvent } from "$lib/bindings";

function bill(over: Partial<CalendarEvent> = {}): CalendarEvent {
  return {
    rule_id: 1,
    pattern: "ENERGIA",
    category_name: "Casa",
    category_color_token: null,
    due_day: 6,
    paid_date: null,
    paid_amount: null,
    paid_transaction_id: null,
    manually_settled: false,
    ...over,
  };
}

describe("dueDateOf", () => {
  it("joins the rule's day with the month being viewed", () => {
    expect(dueDateOf(bill(), "2026-08")).toBe("2026-08-06");
  });

  /** A bill due on the 31st still falls due in February — on its last day. */
  it("clamps the day to the length of the month", () => {
    expect(dueDateOf(bill({ due_day: 31 }), "2026-02")).toBe("2026-02-28");
    expect(dueDateOf(bill({ due_day: 31 }), "2024-02")).toBe("2024-02-29");
  });

  it("is null for a rule with no due day", () => {
    expect(dueDateOf(bill({ due_day: null }), "2026-08")).toBeNull();
  });
});

describe("billState", () => {
  it("is paid once a payment date exists", () => {
    expect(billState(bill({ paid_date: "2026-07-15" }), "2026-08", "2026-08-20")).toBe("paid");
  });

  /** Settled in cash: no date, and still paid. */
  it("is paid when settled by hand with no transaction", () => {
    expect(billState(bill({ manually_settled: true }), "2026-08", "2026-08-20")).toBe("paid");
  });

  it("is overdue once the due date has passed unpaid", () => {
    expect(billState(bill(), "2026-08", "2026-08-09")).toBe("overdue");
  });

  it("is pending on the due date itself", () => {
    expect(billState(bill(), "2026-08", "2026-08-06")).toBe("pending");
  });

  it("is pending in a month that has not arrived", () => {
    expect(billState(bill(), "2026-09", "2026-08-20")).toBe("pending");
  });
});

describe("daysOverdue", () => {
  it("counts whole days since the due date", () => {
    expect(daysOverdue("2026-08-06", "2026-08-09")).toBe(3);
  });

  it("is one on the day after", () => {
    expect(daysOverdue("2026-08-06", "2026-08-07")).toBe(1);
  });

  it("counts across a month boundary", () => {
    expect(daysOverdue("2026-07-30", "2026-08-02")).toBe(3);
  });
});
