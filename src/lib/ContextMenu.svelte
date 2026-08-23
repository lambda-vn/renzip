<script lang="ts">
  /**
   * The menu a right click opens on a row.
   *
   * Positioned at the pointer and clamped to the window, so an entry near the
   * bottom edge does not open a menu half outside the frame. Closes on
   * Escape, on a click anywhere else, and once an item has run.
   */
  import Icon from "./Icon.svelte";
  import type { IconName } from "./Icon.svelte";

  /** One line of the menu. */
  export type MenuItem = {
    /** Wording, already translated. */
    label: string;
    /** Short note shown dimmed after the label. */
    hint?: string;
    /** Icon drawn at the start of the line. */
    icon: IconName;
    /** What the line does. */
    action: () => void;
  };

  type Props = {
    /** Where the pointer was, in window coordinates. */
    x: number;
    /** Where the pointer was, in window coordinates. */
    y: number;
    /** Accessible name of the menu. */
    label: string;
    /** Lines to offer. */
    items: MenuItem[];
    /** Called once the menu should disappear. */
    onclose: () => void;
  };

  let { x, y, label, items, onclose }: Props = $props();

  /** Room kept between the menu and the window edges. */
  const MARGIN = 8;

  let menu = $state<HTMLDivElement>();
  let active = $state(0);
  let size = $state({ width: 0, height: 0 });

  // Placed from the pointer, pulled back inside the window once the menu has
  // been measured. The first frame is drawn at the pointer, which is where it
  // belongs anyway unless the pointer sits near an edge.
  let placed = $derived({
    left: Math.max(MARGIN, Math.min(x, window.innerWidth - size.width - MARGIN)),
    top: Math.max(MARGIN, Math.min(y, window.innerHeight - size.height - MARGIN)),
  });

  $effect(() => {
    const element = menu;
    if (!element) {
      return;
    }
    const box = element.getBoundingClientRect();
    size = { width: box.width, height: box.height };
    element.focus();
  });

  /**
   * Run one line and close.
   *
   * @param item - Line that was chosen.
   */
  function run(item: MenuItem): void {
    onclose();
    item.action();
  }

  /**
   * Walk and choose with the keyboard.
   *
   * @param event - Key event from the menu.
   */
  function onkeydown(event: KeyboardEvent): void {
    switch (event.key) {
      case "Escape":
        onclose();
        break;
      case "ArrowDown":
        active = (active + 1) % items.length;
        break;
      case "ArrowUp":
        active = (active - 1 + items.length) % items.length;
        break;
      case "Home":
        active = 0;
        break;
      case "End":
        active = items.length - 1;
        break;
      case "Enter":
      case " ":
        run(items[active]);
        break;
      default:
        return;
    }
    event.preventDefault();
  }
</script>

<svelte:window onmousedown={onclose} onresize={onclose} />

<div
  bind:this={menu}
  role="menu"
  aria-label={label}
  aria-activedescendant={`menu-item-${active}`}
  tabindex="-1"
  {onkeydown}
  onmousedown={(event) => event.stopPropagation()}
  class="fixed z-50 min-w-56 rounded-[var(--radius-md)] py-1.5"
  style="left:{placed.left}px;top:{placed.top}px;background:var(--color-bg);border:1px solid var(--color-divider);box-shadow:var(--shadow-lg)"
>
  {#each items as item, index (item.label)}
    <button
      type="button"
      role="menuitem"
      id={`menu-item-${index}`}
      tabindex="-1"
      class="flex w-full items-center gap-2.5 px-3 py-2 text-left text-[13px]"
      style={index === active
        ? "background:var(--color-accent-100);color:var(--color-accent-800)"
        : ""}
      onmouseenter={() => (active = index)}
      onclick={() => run(item)}
    >
      <Icon name={item.icon} size={15} stroke={1.8} style="flex:none;opacity:.75" />
      <span class="flex-1 truncate">{item.label}</span>
      {#if item.hint}
        <span class="text-muted text-[11px]">{item.hint}</span>
      {/if}
    </button>
  {/each}
</div>
