/**
 * Estado de ordenação de uma tabela.
 *
 * Vive aqui pra que duas tabelas diferentes não inventem regras diferentes pro
 * mesmo gesto: clicar numa coluna nova cai no sentido mais útil dela, clicar na
 * coluna ativa inverte. É a mesma promessa em toda tabela do app.
 */
export type SortDir = "asc" | "desc";

export type Sort<K extends string> = {
  readonly key: K;
  readonly dir: SortDir;
  /** Alterna a coluna: inverte se já ativa, senão adota o sentido inicial dela. */
  toggle(key: K): void;
  /** Valor de `aria-sort` pro `<th>`. */
  aria(key: K): "ascending" | "descending" | "none";
  /** Sentido que o PRÓXIMO clique nesta coluna vai produzir. */
  next(key: K): SortDir;
  /** `1` pra crescente, `-1` pra decrescente — multiplica o comparador. */
  readonly sign: 1 | -1;
};

/**
 * @param firstDir sentido do primeiro clique de cada coluna. Não é uma regra
 *   única porque a coluna decide o que é útil: data começa pela mais recente,
 *   nome começa por A. É o que o Finder faz.
 * @param initial coluna e sentido em que a tabela abre — normalmente espelhando
 *   a ordem que o backend já devolve, pra abrir sem reordenar nada.
 */
export function createSort<K extends string>(
  firstDir: Record<K, SortDir>,
  initial: { key: K; dir?: SortDir },
): Sort<K> {
  let key = $state<K>(initial.key);
  let dir = $state<SortDir>(initial.dir ?? firstDir[initial.key]);

  return {
    get key() {
      return key;
    },
    get dir() {
      return dir;
    },
    get sign() {
      return dir === "asc" ? 1 : -1;
    },
    toggle(next: K) {
      if (key === next) {
        dir = dir === "asc" ? "desc" : "asc";
      } else {
        key = next;
        dir = firstDir[next];
      }
    },
    aria(k: K) {
      if (k !== key) return "none";
      return dir === "asc" ? "ascending" : "descending";
    },
    next(k: K) {
      if (k !== key) return firstDir[k];
      return dir === "asc" ? "desc" : "asc";
    },
  };
}

/**
 * Comparador de texto no idioma do usuário — "Água" tem que cair perto de
 * "Agua", não no fim do alfabeto.
 */
export function compareText(a: string, b: string, localeCode: string): number {
  return a.localeCompare(b, localeCode, { sensitivity: "base", numeric: true });
}

/**
 * Desempate para colunas que podem não ter valor (um vencimento em branco).
 *
 * Ausência não é "menor": ela vai pro FIM nos dois sentidos, senão inverter a
 * ordem encheria o topo da tabela de traços.
 *
 * Devolve o resultado FINAL do comparador (não multiplique por `sign`), ou
 * `null` quando os dois têm valor — aí quem chamou compara normalmente.
 */
export function nullsLast(a: unknown, b: unknown): number | null {
  const aEmpty = a === null || a === undefined;
  const bEmpty = b === null || b === undefined;
  if (aEmpty && bEmpty) return 0;
  if (aEmpty) return 1;
  if (bEmpty) return -1;
  return null;
}
