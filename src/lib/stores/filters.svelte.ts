function currentMonth(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  return `${y}-${m}`;
}

function createFilterStore() {
  let month = $state<string | null>(currentMonth());
  let categoryId = $state<number | null>(null);

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
    clear() {
      month = null;
      categoryId = null;
    },
    resetToCurrentMonth() {
      month = currentMonth();
      categoryId = null;
    },
  };
}

export const filters = createFilterStore();
