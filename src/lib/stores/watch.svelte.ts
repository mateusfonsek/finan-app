import {
  getAppSetting,
  markFile,
  scanWatchedFolders,
  setAppSetting,
  WATCH_ENABLED_KEY,
  type FileStatus,
} from "$lib/api/watch";
import { OfxReadError } from "$lib/ofx/errors";
import { loadOfxFromPath } from "$lib/ofx/load";

/** Uma descoberta já validada: só chega aqui o que parseou como OFX. */
export type Discovery = {
  hash: string;
  path: string;
  fileName: string;
  txCount: number;
  earliest: string | null;
  latest: string | null;
};

/** Varredura no foco da janela não pode virar rajada de I/O quando o usuário
 *  alterna entre apps. */
const THROTTLE_MS = 10_000;

export function createWatchStore() {
  let enabled = $state(false);
  let discoveries = $state<Discovery[]>([]);
  /** O toast não interrompe quem já está no meio de um import. */
  let suppressToast = $state(false);
  let lastScan = 0;
  let scanning = false;
  /** Ninguém é obrigado a chamar `loadEnabled()` antes do primeiro `refresh()`
   *  (isso é responsabilidade de quem monta a tela). Carregamos sob demanda
   *  aqui pra que o gate do `enabled` nunca rode contra um valor que simplesmente
   *  nunca foi lido — a alternativa (assumir habilitado até a leitura chegar)
   *  arriscaria varrer o disco antes de sabermos se o usuário quer isso.
   */
  let settingsLoaded = false;
  /** Promise em voo da leitura da flag. App.svelte (boot) e o listener de foco
   *  agora podem chamar `refresh()` quase ao mesmo tempo; sem isso, os dois
   *  disparariam sua própria leitura de `getAppSetting` antes que a primeira
   *  resolvesse. Não corrompe nada (leitura é idempotente), mas é I/O em
   *  dobro de graça — compartilhar a promise em voo resolve isso.
   */
  let loadingSettings: Promise<void> | null = null;
  /** Hashes resolvidos (toast/import) enquanto uma varredura está em voo. O
   *  scan trabalha com um snapshot tirado no início; sem isso, seu resultado
   *  no final (`discoveries = next`) reviveria uma descoberta que o usuário
   *  já resolveu enquanto o scan rodava. */
  const resolvedDuringScan = new Set<string>();

  async function refresh(opts: { force?: boolean } = {}) {
    if (!settingsLoaded) await loadEnabled();
    if (!enabled) return;
    const now = Date.now();
    if (!opts.force && now - lastScan < THROTTLE_MS) return;
    if (scanning) return;

    scanning = true;
    lastScan = now;
    resolvedDuringScan.clear();
    try {
      const files = await scanWatchedFolders();
      const next: Discovery[] = [];
      for (const f of files) {
        try {
          const { parsed } = await loadOfxFromPath(f.path);
          next.push({
            hash: f.content_hash,
            path: f.path,
            fileName: f.file_name,
            txCount: parsed.transactions.length,
            earliest: parsed.summary.earliest ?? null,
            latest: parsed.summary.latest ?? null,
          });
        } catch (e) {
          await noteLoadFailure(f.content_hash, e);
        }
      }
      // Descarta do snapshot qualquer hash que o toast/import já resolveu
      // enquanto o scan estava em voo — ver `resolvedDuringScan` acima.
      discoveries = next.filter((d) => !resolvedDuringScan.has(d.hash));
    } finally {
      scanning = false;
    }
  }

  async function loadEnabled() {
    if (loadingSettings) return loadingSettings;
    loadingSettings = (async () => {
      try {
        enabled = (await getAppSetting(WATCH_ENABLED_KEY)) === "1";
        settingsLoaded = true;
      } finally {
        loadingSettings = null;
      }
    })();
    return loadingSettings;
  }

  async function setEnabled(value: boolean) {
    await setAppSetting(WATCH_ENABLED_KEY, value ? "1" : "0");
    enabled = value;
    settingsLoaded = true;
    if (!value) discoveries = [];
    else await refresh({ force: true });
  }

  async function resolve(hash: string, status: FileStatus) {
    await markFile(hash, status);
    discoveries = discoveries.filter((d) => d.hash !== hash);
    // Se um scan estiver em voo, seu snapshot ainda não sabe dessa resolução —
    // avisamos aqui pra ele não reviver o item ao terminar.
    if (scanning) resolvedDuringScan.add(hash);
  }

  /** Tira a descoberta da lista **sem** tocar no banco: o arquivo continua
   *  `pending` e a próxima varredura tenta de novo. É o oposto de `resolve`,
   *  que é definitivo. */
  function skip(hash: string) {
    discoveries = discoveries.filter((d) => d.hash !== hash);
    if (scanning) resolvedDuringScan.add(hash);
  }

  /** Política única pra "não consegui abrir esta descoberta" (spec §5.4).
   *
   *  Ler falhou → transitório (stub do iCloud evictado depois da varredura,
   *  arquivo movido, permissão): fica `pending` e volta no próximo scan.
   *  Parse falhou → o conteúdo não é OFX e não vai virar: `invalid`, que é
   *  permanente, e o usuário nunca vê um erro que é nosso. */
  async function noteLoadFailure(hash: string, e: unknown) {
    if (e instanceof OfxReadError) skip(hash);
    else await resolve(hash, "invalid");
  }

  /** Pedido de "abrir esta descoberta no Import", vindo do toast.
   *
   *  Sinal em vez de navegação direta porque `push("/import")` estando já em
   *  `/import` não dispara `hashchange` — o Import não remonta e o `onMount`
   *  não roda de novo. A tela de Import observa este sinal e carrega no lugar,
   *  funcione ela recém-montada ou já na tela. */
  let openRequest = $state<Discovery | null>(null);

  function requestOpen(discovery: Discovery) {
    openRequest = discovery;
  }

  /** Consome o pedido (só pode ser atendido uma vez — sem isso, um pedido
   *  antigo reabriria um extrato ao voltar pra tela depois). */
  function takeOpenRequest(): Discovery | null {
    const req = openRequest;
    openRequest = null;
    return req;
  }

  return {
    get enabled() {
      return enabled;
    },
    get discoveries() {
      return discoveries;
    },
    get pendingCount() {
      return discoveries.length;
    },
    get suppressToast() {
      return suppressToast;
    },
    set suppressToast(v: boolean) {
      suppressToast = v;
    },
    get openRequest() {
      return openRequest;
    },
    refresh,
    loadEnabled,
    setEnabled,
    resolve,
    skip,
    noteLoadFailure,
    requestOpen,
    takeOpenRequest,
  };
}

export const watch = createWatchStore();
