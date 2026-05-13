function currentMonth(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  return `${y}-${m}`;
}

function createFilterStore() {
  let month = $state<string | null>(currentMonth());
  let categoryId = $state<number | null>(null);
  let q = $state<string>("");

  return {
    get month() {
      return month;
    },
    set month(v: string | null) {
      month = v;
    },
    get categoryId() {
      return categoryId;
    },
    set categoryId(v: number | null) {
      categoryId = v;
    },
    get q() {
      return q;
    },
    set q(v: string) {
      q = v;
    },
    clear() {
      month = null;
      categoryId = null;
      q = "";
    },
    resetToCurrentMonth() {
      month = currentMonth();
      categoryId = null;
      q = "";
    },
  };
}

export const filters = createFilterStore();
