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
  import { formatSize } from "./format";
  import { t, tn } from "./i18n.svelte";
  import { glyphOf, typeLabel } from "./kinds";
  import type { SourceRow } from "./sourceTree";
  import { app } from "./store.svelte";

  /** Height of one row, shared with the virtual list. */
  const ROW_HEIGHT = 28;

  let list = $state<ReturnType<typeof VirtualList> | undefined>();
  let active = $state(0);
  let menu = $state<{ x: number; y: number; label: string; items: MenuItem[] } | null>(null);

  let rows = $derived(app.stagedRows);

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
   * Walk and unfold with the keyboard.
   *
   * @param event - Key event from the tree.
   */
  function onkeydown(event: KeyboardEvent): void {
    if (rows.length === 0) {
      return;
    }
    const row = rows[active];
    let next: number | null = null;

    switch (event.key) {
      case "ArrowDown":
        next = Math.min(rows.length - 1, active + 1);
        break;
      case "ArrowUp":
        next = Math.max(0, active - 1);
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = rows.length - 1;
        break;
      case "ArrowRight":
        if (row.kind === "dir" && !row.open && row.hasKids) {
          app.toggleStagedFolder(row.key);
        } else if (row.kind === "dir" && row.open) {
          next = Math.min(rows.length - 1, active + 1);
        }
        break;
      case "ArrowLeft":
        if (row.kind === "dir" && row.open) {
          app.toggleStagedFolder(row.key);
        }
        break;
      case "Enter":
        if (row.kind === "dir") {
          app.toggleStagedFolder(row.key);
        }
        break;
      case "Delete":
        app.excludeFromSource(row.key);
        break;
      default:
        return;
    }

    event.preventDefault();
    if (next !== null) {
      active = next;
      list?.scrollToIndex(next);
    }
  }

  /**
   * Open the row menu where the pointer is.
   *
   * @param event - The right click.
   * @param index - Row it landed on.
   */
  function openMenu(event: MouseEvent, index: number): void {
    event.preventDefault();
    active = index;
    const row = rows[index];
    menu = {
      x: event.clientX,
      y: event.clientY,
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
  function label(row: SourceRow): string {
    if (row.kind === "dir") {
      return `${t("browse.folderAria", { name: row.name })}, ${tn("count.file", row.count)}, ${formatSize(row.size)}`;
    }
    return `${row.name}, ${typeLabel(row.key)}, ${formatSize(row.size)}`;
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
>
  {#snippet row(item: SourceRow, index: number)}
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
      oncontextmenu={(event) => openMenu(event, index)}
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
        <span class="nums w-24 text-right">{formatSize(item.size)}</span>
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
