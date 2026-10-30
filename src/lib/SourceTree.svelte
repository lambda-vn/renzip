<script lang="ts">
  /**
   * The staged content list of the creation screen.
   *
   * Mirrors EntryTree's fold-in-place list, but for a directory not yet
   * packed: there is no selection to tick and nothing to preview, only a
   * fold state per folder and a "remove" action a right click reaches on
   * any file or folder, at any depth.
   */
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";
  import Icon from "./Icon.svelte";
  import VirtualList from "./VirtualList.svelte";
  import type { SourceFile } from "./api";
  import { formatSize } from "./format";
  import { t, tn } from "./i18n.svelte";
  import { glyphOf, typeLabel } from "./kinds";
  import { app } from "./store.svelte";
  import { navigate, type TreeRow } from "./tree";

  /** Height of one row, shared with the virtual list. */
  const ROW_HEIGHT = 28;

  let list = $state<ReturnType<typeof VirtualList> | undefined>();
  let active = $state(0);
  let menu = $state<{ x: number; y: number; label: string; items: MenuItem[] } | null>(null);

  let rows = $derived(app.stagedRows);

  // Excluding or folding can shrink the list under the active row; pull it
  // back inside, or the next key acts on a row that no longer exists.
  $effect(() => {
    if (active >= rows.length) {
      active = Math.max(0, rows.length - 1);
    }
  });

  /**
   * Act on a click on a row: unfold or fold a folder.
   *
   * @param index - Index of the row among the visible ones.
   */
  function activate(index: number): void {
    const row = rows[index];
    active = index;
    if (row.kind === "dir") {
      app.toggleStagedFolder(row.key);
    }
  }

  /**
   * Walk, unfold and exclude with the keyboard.
   *
   * @param event - Key event from the tree.
   */
  function onkeydown(event: KeyboardEvent): void {
    if (rows.length === 0) {
      return;
    }
    const next = navigate(event.key, rows, active, (key) => app.toggleStagedFolder(key));
    if (next === undefined) {
      if (act(event, rows[active])) {
        event.preventDefault();
      }
      return;
    }
    event.preventDefault();
    if (next !== null) {
      active = next;
      list?.scrollToIndex(next);
    }
  }

  /**
   * Run what a key does to the active row besides moving through the tree.
   *
   * @param event - Key event from the tree.
   * @param row - The active row.
   * @returns Whether the key meant anything here.
   */
  function act(event: KeyboardEvent, row: TreeRow<SourceFile>): boolean {
    switch (event.key) {
      case "Enter":
        if (row.kind === "dir") {
          app.toggleStagedFolder(row.key);
        }
        return true;
      case "Delete":
        app.excludeFromSource(row.key);
        return true;
      default:
        return false;
    }
  }

  /**
   * Open the row menu.
   *
   * @param index - Row it is for.
   * @param x - Where to place it, in window coordinates: the pointer for a
   *   right click, under the row for the keyboard.
   * @param y - Same, vertically.
   */
  function openMenu(index: number, x: number, y: number): void {
    active = index;
    const row = rows[index];
    menu = {
      x,
      y,
      label: t("menu.label", { name: row.name }),
      items: [
        {
          label: t("menu.remove"),
          icon: "close",
          action: () => app.excludeFromSource(row.key),
        },
      ],
    };
  }

  /** Text a screen reader hears for one row. */
  function label(row: TreeRow<SourceFile>): string {
    if (row.kind === "dir") {
      return `${t("browse.folderAria", { name: row.name })}, ${tn("count.file", row.count)}, ${formatSize(row.size)}`;
    }
    return `${row.name}, ${typeLabel(row.key)}, ${formatSize(row.entry.size)}`;
  }
</script>

<VirtualList
  bind:this={list}
  items={rows}
  rowHeight={ROW_HEIGHT}
  role="tree"
  ariaLabel={t("create.staged")}
  activeDescendant={rows.length > 0 ? `staged-row-${active}` : undefined}
  {onkeydown}
  onmenukey={(x, y) => openMenu(active, x, y)}
>
  {#snippet row(item: TreeRow<SourceFile>, index: number)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      id={`staged-row-${index}`}
      role="treeitem"
      tabindex="-1"
      aria-level={item.depth + 1}
      aria-expanded={item.kind === "dir" ? item.open : undefined}
      aria-selected="false"
      aria-label={label(item)}
      class="row rounded-none px-4.5 {index === active ? 'row-active' : ''}"
      style="height:{ROW_HEIGHT}px;padding-left:{18 + item.depth * 16}px"
      onclick={() => activate(index)}
      oncontextmenu={(event) => {
        event.preventDefault();
        openMenu(index, event.clientX, event.clientY);
      }}
    >
      {#if item.kind === "dir"}
        <button
          type="button"
          tabindex="-1"
          aria-hidden="true"
          class="grid size-3.5 flex-none place-items-center transition-transform duration-150"
          style="opacity:{item.hasKids ? 0.7 : 0};transform:rotate({item.open ? 90 : 0}deg)"
          onclick={(event) => {
            event.stopPropagation();
            app.toggleStagedFolder(item.key);
          }}
        >
          <Icon name="chevron" size={12} stroke={2.4} />
        </button>
        <Icon name="folder" size={15} stroke={1.7} style="flex:none;opacity:.75" />
        <span class="flex-1 truncate" title={item.key}>{item.name}</span>
        <span class="text-muted w-24">{tn("count.file", item.count)}</span>
        <span class="nums w-24 text-right">{formatSize(item.size)}</span>
      {:else}
        <span class="size-3.5 flex-none"></span>
        <Icon name={glyphOf(item.key).icon} size={15} stroke={1.7} style="flex:none;opacity:.75" />
        <span class="flex-1 truncate" title={item.key}>{item.name}</span>
        <span class="text-muted w-24">{typeLabel(item.key)}</span>
        <span class="nums w-24 text-right">{formatSize(item.entry.size)}</span>
      {/if}
    </div>
  {/snippet}
</VirtualList>

{#if menu}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    label={menu.label}
    items={menu.items}
    onclose={() => (menu = null)}
  />
{/if}
