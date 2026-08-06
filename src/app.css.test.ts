import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

// Path from the project root: under jsdom, `import.meta.url` is an http URL
// from the Vite server, and `fileURLToPath` rejects any scheme other than
// `file:`.
const css = readFileSync(resolve(process.cwd(), "src/app.css"), "utf8");

/** Body of the first `:focus-visible` rule that sits OUTSIDE any `@layer`. */
function unlayeredFocusVisibleBlock(): string {
  // Walks brace depth to know whether the rule is inside a `@layer` — a regex
  // alone cannot tell nesting apart.
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
      // The global rule only, not variants such as `.field:focus-visible`.
      if (selectorEnd.trim() === ":focus-visible") return css.slice(open + 1, close);
    }
  }
  return "";
}

describe("focus ring", () => {
  /**
   * The global `:focus-visible` rule lives outside `@layer` so the ring's
   * appearance never loses to a `shadow-*` utility. The side effect is that it
   * beats EVERYTHING — Tailwind's `utilities` layer included.
   *
   * When it declared `position: relative`, it overrode the positioning of any
   * already-positioned focusable. The About dialog's close button is
   * `absolute right-3.5 top-3.5`: on focus it became `relative`, fell back into
   * normal flow (the card's top-left) and `right: 14px` — which on a relative
   * element pushes left instead of anchoring right — threw it 12px outside the
   * card. Measured, not assumed.
   */
  it("the global rule does not override position on already-positioned elements", () => {
    const block = unlayeredFocusVisibleBlock();

    expect(block, "global :focus-visible rule not found outside @layer").not.toBe("");
    expect(
      block,
      "position in an unlayered rule beats Tailwind's utilities and breaks " +
        "elements with absolute/fixed/sticky — move it to @layer base",
    ).not.toMatch(/(^|[;{\s])position\s*:/);
  });

  it("the stacking promotion still exists, in a low layer", () => {
    // Without it, the ring on static elements can end up under a later sibling
    // — which is the reason the promotion exists.
    expect(css).toMatch(/@layer\s+base\s*\{[^}]*:focus-visible\s*\{[^}]*position\s*:\s*relative/s);
    expect(css).toMatch(/@layer\s+base\s*\{[^}]*:focus-visible\s*\{[^}]*z-index\s*:\s*1/s);
  });
});
