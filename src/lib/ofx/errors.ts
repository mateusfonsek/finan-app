/** Falha ao *ler os bytes* de um `.ofx` do disco — distinta de "os bytes não
 *  são um OFX válido".
 *
 *  A distinção existe porque as duas falhas têm consequências opostas na pasta
 *  observada (spec §5.4): conteúdo inválido é permanente (o arquivo é marcado
 *  `invalid` e nunca mais avisa), enquanto não conseguir ler é transitório —
 *  placeholder do iCloud que voltou a ser stub, arquivo movido entre a
 *  varredura e a leitura, permissão negada. Tratar a segunda como a primeira
 *  enterraria o extrato pra sempre, já que a chave é o hash do conteúdo.
 *
 *  Mora num módulo próprio (e não em `load.ts`) pra que quem faz mock de
 *  `$lib/ofx/load` nos testes continue comparando contra a classe real.
 */
export class OfxReadError extends Error {
  readonly path: string;

  constructor(path: string, cause: unknown) {
    super(cause instanceof Error ? cause.message : String(cause));
    this.name = "OfxReadError";
    this.path = path;
  }
}
