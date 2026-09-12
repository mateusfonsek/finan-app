/** The part of an anchor's box that decides where its popover goes. */
export type AnchorRect = { top: number; bottom: number; left: number };

export type Viewport = { width: number; height: number };

export type Placement = {
  /** The panel opens upward, growing from its bottom edge. */
  up: boolean;
  left: number;
  /** Distance from the viewport's top edge, or from its bottom when `up`. */
  offset: number;
  /** Room the panel actually has. Its content scrolls inside this. */
  maxHeight: number;
};

/** Gap between the anchor and the panel. */
const GAP = 6;

/** Closest the panel ever comes to a viewport edge. */
const MARGIN = 8;

/** Below this, the room under the anchor is too cramped to be worth using when
 *  there is more of it above. */
const MIN_HEIGHT = 240;

/**
 * A popover is `fixed`, so whatever leaves the viewport is unreachable — no
 * scroll brings it back. Both axes are therefore bounded here: the panel opens
 * above its anchor when the room below is cramped, and never claims more height
 * than the side it landed on has.
 */
export function placePopover(anchor: AnchorRect, view: Viewport, width: number): Placement {
  const below = view.height - anchor.bottom - GAP - MARGIN;
  const above = anchor.top - GAP - MARGIN;
  const up = below < MIN_HEIGHT && above > below;
  return {
    up,
    left: Math.min(Math.max(MARGIN, anchor.left), view.width - width - MARGIN),
    offset: up ? view.height - anchor.top + GAP : anchor.bottom + GAP,
    maxHeight: up ? above : below,
  };
}
