/**
 * Moves the node to the end of `<body>`.
 *
 * Exists because of a CSS rule that bites silently: an ancestor with
 * `backdrop-filter` (or `transform`, `filter`, `perspective`) becomes the
 * containing block for `position: fixed` descendants. Each screen's header is
 * translucent material, so a fixed popover declared inside it positions itself
 * against THE HEADER, and viewport-derived coordinates come out offset by the
 * sidebar's width.
 *
 * Moving the node to `body` restores the viewport as the reference.
 */
export function portal(node: HTMLElement) {
  document.body.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
