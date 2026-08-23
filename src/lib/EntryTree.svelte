<script lang="ts">
  /**
   * The single list of the main window: folders and files in one tree.
   *
   * A folder unfolds in place, its files sit under it, and a tick box on any
   * row builds the extraction selection: ticking a folder takes everything
   * below it. Only the visible rows are in the DOM, so an archive of fifty
   * thousand entries scrolls as well as one of ten.
   *
   * Follows the ARIA tree pattern: arrows walk and unfold, space ticks, and
   * the rows are never focused themselves (RGAA 7.1, 12.x).
   */
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";
  import Icon from "./Icon.svelte";
  import VirtualList from "./VirtualList.svelte";
  import { entryAssociation, openEntry, type Association } from "./api";
  import { formatSize } from "./format";
  import { t, tn } from "./i18n.svelte";
  import { glyphOf, typeLabel } from "./kinds";
  import { app } from "./store.svelte";
  import { descendants, type TreeRow } from "./tree";

  type Props = {
    /** Extract these entries, asking for a destination if none is set. */
    onextract: (paths: string[]) => void;
  };

  let { onextract }: Props = $props();

  /** Height of one row, shared with the virtual list. */
  const ROW_HEIGHT = 28;

  let list = $state<ReturnType<typeof VirtualList> | undefined>();
  let active = $state(0);
  let menu = $state<{ x: number; y: number; label: string; items: MenuItem[] } | null>(null);

  let rows = $derived(app.rows);

  /**
   * State of the tick box of a row.
   *
   * @param row - Row to look at.
   * @returns "on" when everything it stands for is picked, "part" when only
   *   some of it is, "off" otherwise.
   */
  function ticked(row: TreeRow): "on" | "part" | "off" {
    if (row.kind === "file") {
      return app.selected.has(row.key) ? "on" : "off";
    }
    const picked = app.tally.get(row.key) ?? 0;
    if (picked === 0) {
      return "off";
    }
    return picked >= row.count ? "on" : "part";
  }

  /**
   * Act on a click on a row: unfold a folder, or show a file.
   *
   * @param index - Index of the row among the visible ones.
   */
  function activate(index: number): void {
    const row = rows[index];
    active = index;
    if (row.kind === "dir") {
      app.toggleFolder(row.key);
    } else {
      void app.show(row.entry);
    }
  }

  /**
   * Walk, unfold and tick with the keyboard.
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
      case "PageDown":
        next = Math.min(rows.length - 1, active + 12);
        break;
      case "PageUp":
        next = Math.max(0, active - 12);
        break;
      case "ArrowRight":
        if (row.kind === "dir" && !row.open && row.hasKids) {
          app.toggleFolder(row.key);
        } else if (row.kind === "dir" && row.open) {
          next = Math.min(rows.length - 1, active + 1);
        }
        break;
      case "ArrowLeft":
        if (row.kind === "dir" && row.open) {
          app.toggleFolder(row.key);
        }
        break;
      case " ":
        app.toggleRow(row);
        break;
      case "Enter":
        if (row.kind === "file") {
          void app.show(row.entry, true);
        } else {
          app.toggleFolder(row.key);
        }
        break;
      case "a":
        if (event.ctrlKey || event.metaKey) {
          for (const entry of app.entries) {
            app.selected.add(entry.path);
          }
          break;
        }
        return;
      default:
        return;
    }

    event.preventDefault();
    if (next !== null) {
      active = next;
      list?.scrollToIndex(next);
      const target = rows[next];
      if (target.kind === "file") {
        void app.show(target.entry);
      }
    }
  }

  /**
   * What the desktop opens each extension with, asked once per extension.
   *
   * The answer costs a registry lookup and never changes while the window is
   * up, so the second right click on a .png is instant.
   */
  const associations = new Map<string, Association>();

  /**
   * Look the association up, through the cache.
   *
   * @param path - Entry path, whose extension is what actually matters.
   */
  async function associationOf(path: string): Promise<Association> {
    const name = path.slice(path.lastIndexOf("/") + 1);
    const dot = name.lastIndexOf(".");
    const extension = dot <= 0 ? "" : name.slice(dot + 1).toLowerCase();
    if (extension === "") {
      return "unknown";
    }
    const known = associations.get(extension);
    if (known) {
      return known;
    }
    const answer = await entryAssociation(extension).catch<Association>(() => "unknown");
    associations.set(extension, answer);
    return answer;
  }

  /** Entry paths a row stands for: itself, or everything under it. */
  function pathsOf(row: TreeRow): string[] {
    return row.kind === "file"
      ? [row.key]
      : descendants(app.tree, row.key).map((entry) => entry.path);
  }

  /**
   * Open the row menu where the pointer is.
   *
   * @param event - The right click.
   * @param index - Row it landed on.
   */
  async function openMenu(event: MouseEvent, index: number): Promise<void> {
    event.preventDefault();
    active = index;
    const row = rows[index];
    const items: MenuItem[] = [
      {
        label: t("menu.extract"),
        icon: "download",
        action: () => onextract(pathsOf(row)),
      },
      {
        label: t("menu.copy"),
        icon: "copy",
        action: () => void copyName(row),
      },
    ];
    if (row.kind === "file") {
      items.push({
        label: t("menu.open"),
        hint: t("menu.open.hint"),
        icon: "external",
        action: () => void openTemporaryCopy(row.key, false),
      });
      // "Open with" only earns its line when opening would do something
      // else: with no application registered, both entries end on the same
      // picker and the menu just repeats itself.
      if ((await associationOf(row.key)) === "runnable") {
        items.push({
          label: t("menu.openWith"),
          icon: "settings",
          action: () => void openTemporaryCopy(row.key, true),
        });
      }
    }
    menu = { x: event.clientX, y: event.clientY, label: t("menu.label", { name: row.name }), items };
  }

  /**
   * Put a row's name on the clipboard.
   *
   * The full path, not just the file name: that is the name the archive
   * itself uses, and the one another tool will be given.
   *
   * @param row - Row that was right clicked.
   */
  async function copyName(row: TreeRow): Promise<void> {
    try {
      await writeText(row.key);
      app.status = { tone: "info", text: t("status.copied", { name: row.key }) };
    } catch (error) {
      app.status = { tone: "warn", text: String(error) };
    }
  }

  /**
   * Write one entry to a temporary directory and hand it to the desktop.
   *
   * A .rpyc, or anything else this desktop has no application for, cannot be
   * opened; the backend then asks the desktop for its "open with" chooser, or
   * shows the file in its folder where there is no such chooser. The status
   * line says which of the three happened instead of reporting a failure.
   *
   * @param path - Entry to open.
   * @param chooser - Ask for the picker rather than the registered
   *   application, which is the only way out when an extension was once
   *   pointed at a program that is not the one wanted today.
   */
  async function openTemporaryCopy(path: string, chooser: boolean): Promise<void> {
    if (!app.archive) {
      return;
    }
    try {
      const outcome = await openEntry(app.archive.handle, path, chooser);
      app.status = {
        tone: outcome.action === "opened" ? "info" : "warn",
        text: t(`status.${outcome.action}` as const, { path: outcome.path }),
      };
    } catch (error) {
      app.status = { tone: "warn", text: String(error) };
    }
  }

  /** Text a screen reader hears for one row. */
  function label(row: TreeRow): string {
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
  ariaLabel={t("browse.tree")}
  activeDescendant={rows.length > 0 ? `row-${active}` : undefined}
  {onkeydown}
>
  {#snippet row(item: TreeRow, index: number)}
    <!--
      The keyboard lives on the tree itself, which is the ARIA pattern for
      aria-activedescendant: rows are never focused, so a key handler here
      would never fire.
    -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      id={`row-${index}`}
      role="treeitem"
      tabindex="-1"
      aria-level={item.depth + 1}
      aria-selected={ticked(item) === "on"}
      aria-expanded={item.kind === "dir" ? item.open : undefined}
      aria-label={label(item)}
      class="row rounded-none px-4.5 {index === active ? 'row-active' : ''}"
      style="height:{ROW_HEIGHT}px;padding-left:{18 + item.depth * 16}px"
      onclick={() => activate(index)}
      oncontextmenu={(event) => void openMenu(event, index)}
    >
      <!--
        Hit areas, not controls: space ticks the row and the arrows unfold it,
        so a second tab stop per row would only slow the keyboard down.
      -->
      <button
        type="button"
        tabindex="-1"
        aria-hidden="true"
        class="check-box"
        data-state={ticked(item)}
        onclick={(event) => {
          event.stopPropagation();
          app.toggleRow(item);
        }}
      >
        {#if ticked(item) === "on"}
          <Icon name="check" size={10} stroke={4} />
        {:else if ticked(item) === "part"}
          <span class="dash"></span>
        {/if}
      </button>

      {#if item.kind === "dir"}
        <button
          type="button"
          tabindex="-1"
          aria-hidden="true"
          class="grid size-3.5 flex-none place-items-center transition-transform duration-150"
          style="opacity:{item.hasKids ? 0.7 : 0};transform:rotate({item.open ? 90 : 0}deg)"
          onclick={(event) => {
            event.stopPropagation();
            app.toggleFolder(item.key);
          }}
        >
          <Icon name="chevron" size={12} stroke={2.4} />
        </button>
        <Icon name="folder" size={15} stroke={1.7} style="flex:none;opacity:.75" />
        <span class="flex-1 truncate" title={item.key}>{item.name}</span>
        <span class="text-muted w-20">{t("browse.folder")}</span>
        <span class="nums w-28 text-right">{formatSize(item.size)}</span>
        <span class="nums text-muted w-32 text-right">{tn("count.file", item.count)}</span>
      {:else}
        <span class="size-3.5 flex-none"></span>
        <Icon
          name={glyphOf(item.key).icon}
          size={15}
          stroke={1.7}
          style="flex:none;opacity:.75"
        />
        <span class="flex-1 truncate" title={item.key}>{item.name}</span>
        <span class="text-muted w-20">{typeLabel(item.key)}</span>
        <span class="nums w-28 text-right">{formatSize(item.entry.size)}</span>
        <span class="nums text-muted w-32 text-right">
          0x{item.entry.offset.toString(16).toUpperCase().padStart(8, "0")}
        </span>
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
