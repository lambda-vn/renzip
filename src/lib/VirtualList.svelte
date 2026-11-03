<script lang="ts" generics="T">
  /**
   * Fixed height virtual list.
   *
   * Only the rows in view are in the DOM, which is what keeps an archive of
   * fifty thousand entries scrollable. The scroll container carries the
   * tree role, so the rows rendered by the caller must carry the treeitem
   * role (RGAA 7.1).
   */
  import type { Snippet } from "svelte";

  type Props = {
    /** Everything the list holds, in display order. */
    items: T[];
    /** Height of one row in pixels, identical for every row. */
    rowHeight?: number;
    /** Accessible name of the list. */
    ariaLabel: string;
    /** Id of the row that has virtual focus. */
    activeDescendant?: string;
    /** Row renderer, called with the item and its absolute index. */
    row: Snippet<[T, number]>;
    /** Keyboard handling, owned by the caller since selection lives there. */
    onkeydown?: (event: KeyboardEvent) => void;
    /**
     * The menu key or Shift+F10 asked for the active row's menu, to be
     * placed at `x`, `y` in window coordinates.
     */
    onmenukey?: (x: number, y: number) => void;
  };

  let {
    items,
    rowHeight = 28,
    ariaLabel,
    activeDescendant,
    row,
    onkeydown,
    onmenukey,
  }: Props = $props();

  /**
   * Turn a contextmenu event the keyboard raised into a menu for the active
   * row.
   *
   * The menu key and Shift+F10 fire on the element holding the focus, which
   * is this container and never a row: rows only have virtual focus. A
   * right click lands on a row first, whose own handler marks the event as
   * handled, and is left alone here.
   *
   * @param event - The contextmenu event, bubbled or not.
   */
  function oncontextmenu(event: MouseEvent): void {
    if (event.defaultPrevented || !onmenukey || !activeDescendant) {
      return;
    }
    const box = document.getElementById(activeDescendant)?.getBoundingClientRect();
    if (!box) {
      return;
    }
    event.preventDefault();
    onmenukey(box.left + 24, box.bottom);
  }

  /** Rows kept on each side of the viewport, to hide the recycling. */
  const OVERSCAN = 8;

  let viewport = $state<HTMLDivElement>();
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  let first = $derived(
    Math.max(0, Math.floor(scrollTop / rowHeight) - OVERSCAN),
  );
  let visible = $derived(
    Math.ceil(viewportHeight / rowHeight) + OVERSCAN * 2 + 1,
  );
  let last = $derived(Math.min(items.length, first + visible));
  let slice = $derived(items.slice(first, last));

  /** Put the focus back on the tree itself, where the keyboard lives. */
  export function focus(): void {
    viewport?.focus();
  }

  /**
   * Bring a row into view, scrolling as little as possible.
   *
   * @param index - Absolute index of the row.
   */
  export function scrollToIndex(index: number): void {
    if (!viewport) {
      return;
    }
    const top = index * rowHeight;
    const bottom = top + rowHeight;
    if (top < viewport.scrollTop) {
      viewport.scrollTop = top;
    } else if (bottom > viewport.scrollTop + viewport.clientHeight) {
      viewport.scrollTop = bottom - viewport.clientHeight;
    }
  }
</script>

<!--
  The tree owns the keyboard for its rows, which only have virtual focus
  through aria-activedescendant, so the container is the tab stop.
-->
<div
  bind:this={viewport}
  bind:clientHeight={viewportHeight}
  onscroll={(event) => (scrollTop = event.currentTarget.scrollTop)}
  {onkeydown}
  {oncontextmenu}
  role="tree"
  aria-label={ariaLabel}
  aria-activedescendant={activeDescendant}
  tabindex="0"
  class="h-full overflow-y-auto focus:outline-none"
  style="scrollbar-gutter:stable"
>
  <div style="height: {items.length * rowHeight}px; position: relative;">
    <div
      style="position: absolute; top: 0; left: 0; right: 0; transform: translateY({first *
        rowHeight}px);"
    >
      {#each slice as item, offset (first + offset)}
        {@render row(item, first + offset)}
      {/each}
    </div>
  </div>
</div>
