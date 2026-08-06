import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

// Caminho a partir da raiz do projeto: sob jsdom, `import.meta.url` é uma URL
// http do servidor do Vite, e `fileURLToPath` recusa qualquer esquema que não
// seja `file:`.
const css = readFileSync(resolve(process.cwd(), "src/app.css"), "utf8");

/** Corpo da primeira regra `:focus-visible` que estiver FORA de `@layer`. */
function unlayeredFocusVisibleBlock(): string {
  // Varre em profundidade de chaves para saber se a regra está dentro de um
  // `@layer` — regex sozinho não distingue aninhamento.
  let depth = 0;
  let inLayer: number | null = null;
  for (let i = 0; i < css.length; i++) {
    const rest = css.slice(i);
    if (rest.startsWith("@layer") && inLayer === null) inLayer = depth;
    if (css[i] === "{") depth++;
    if (css[i] === "}") {
      depth--;
      if (inLayer !== null && depth <= inLayer) inLayer = null;
    }
    if (inLayer === null && rest.startsWith(":focus-visible")) {
      const open = css.indexOf("{", i);
      const close = css.indexOf("}", open);
      const selectorEnd = css.slice(i, open);
      // Só a regra global, não variações como `.field:focus-visible`.
      if (selectorEnd.trim() === ":focus-visible") return css.slice(open + 1, close);
    }
  }
  return "";
}

describe("anel de foco", () => {
  /**
   * A regra global de `:focus-visible` vive fora de `@layer` para que a
   * aparência do anel nunca perca para um utilitário `shadow-*`. O efeito
   * colateral é que ela vence TUDO — inclusive a camada `utilities` do
   * Tailwind.
   *
   * Quando ela declarava `position: relative`, sobrescrevia o posicionamento de
   * qualquer elemento focável já posicionado. O X do diálogo Sobre é
   * `absolute right-3.5 top-3.5`: ao receber foco virava `relative`, caía no
   * fluxo normal (canto superior esquerdo do card) e o `right: 14px` — que em
   * elemento relativo empurra para a esquerda em vez de ancorar à direita — o
   * jogava 12px para fora do card. Medido, não suposto.
   */
  it("a regra global não sobrescreve position de quem já é posicionado", () => {
    const block = unlayeredFocusVisibleBlock();

    expect(block, "regra global :focus-visible não encontrada fora de @layer").not.toBe("");
    expect(
      block,
      "position em regra sem camada vence o utilities do Tailwind e quebra " +
        "elementos com absolute/fixed/sticky — mova para @layer base",
    ).not.toMatch(/(^|[;{\s])position\s*:/);
  });

  it("a promoção para empilhamento continua existindo, em camada baixa", () => {
    // Sem isso, o anel de elementos estáticos pode ficar por baixo de um irmão
    // posterior — que é o motivo de a promoção existir.
    expect(css).toMatch(/@layer\s+base\s*\{[^}]*:focus-visible\s*\{[^}]*position\s*:\s*relative/s);
    expect(css).toMatch(/@layer\s+base\s*\{[^}]*:focus-visible\s*\{[^}]*z-index\s*:\s*1/s);
  });
});
