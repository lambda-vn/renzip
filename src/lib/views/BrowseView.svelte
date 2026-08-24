<script lang="ts">
  /**
   * The main window: everything about an open archive on one page.
   *
   * One list holds folders and files together, foldable in place; the panel
   * on the right shows what is picked and extracts it. The design drew the
   * folders, the files and the extraction as three separate surfaces; they
   * were merged because moving between them said nothing the one list does
   * not already show.
   */
  import EntryTree from "../EntryTree.svelte";
  import Icon from "../Icon.svelte";
  import SelectionPanel from "../SelectionPanel.svelte";
  import { formatSize } from "../format";
  import { t, tn } from "../i18n.svelte";
  import { app } from "../store.svelte";

  type Props = {
    /** Ask for the open dialog. */
    onopen: () => void;
    /** Extract these entries, or the whole archive when given null. */
    onextract: (paths: string[] | null) => void;
    /** Ask for the destination directory. */
    onpickdest: () => void;
  };

  let { onopen, onextract, onpickdest }: Props = $props();

  let allTicked = $derived(app.entries.length > 0 && app.selectionSize === app.entries.length);

  /** Tick every entry of the archive, or none. */
  function toggleAll(): void {
    if (allTicked) {
      app.selected.clear();
    } else {
      for (const entry of app.entries) {
        app.selected.add(entry.path);
      }
    }
  }

  /** Unfold every folder of the archive, at every depth, or fold them all. */
  function toggleFolders(): void {
    if (app.open.size > 0) {
      app.open.clear();
      return;
    }
    // The whole tree, not only the rows on screen: unfolding what is visible
    // would stop one level down and need a second click per depth.
    for (const key of app.tree.keys()) {
      if (key !== "") {
        app.open.add(key);
      }
    }
  }
</script>

<div class="flex min-h-0 flex-1 flex-col">
  <div
    class="flex shrink-0 flex-wrap items-center gap-3 px-4.5 py-3"
    style="border-bottom:1px solid var(--color-divider)"
  >
    <label class="search w-80">
      <Icon name="search" size={14} style="opacity:.5;flex:none" />
      <input
        id="archive-search"
        type="search"
        bind:value={app.filter}
        placeholder={t("browse.filter")}
        aria-label={t("browse.filter.label")}
        disabled={!app.archive}
      />
    </label>

    <div class="ml-auto flex gap-2">
      <button type="button" class="btn btn-secondary" onclick={onopen} disabled={app.busy}>
        <Icon name="archive" size={15} />
        {t("browse.open")}
      </button>
      <button
        type="button"
        class="btn btn-primary"
        onclick={() => (app.view = "create")}
        disabled={app.busy}
      >
        <Icon name="plus" size={15} />
        {t("browse.create")}
      </button>
    </div>
  </div>

  {#if !app.archive}
    <div class="flex min-h-0 flex-1 items-center justify-center p-10">
      <p class="text-muted max-w-100 text-center text-[13px]">{t("browse.welcome")}</p>
    </div>
  {:else}
    <div class="flex min-h-0 flex-1">
      <div class="flex min-w-0 flex-1 flex-col" style="border-right:1px solid var(--color-divider)">
        <div
          class="flex shrink-0 items-center gap-2 px-4.5 py-1.5"
          style="border-bottom:1px solid var(--color-divider)"
        >
          <h2
            class="text-muted m-0 flex-1 text-[11px] font-semibold uppercase"
            style="letter-spacing:0.08em"
          >
            {t("browse.content")}
          </h2>
          <button type="button" class="btn btn-ghost" onclick={toggleFolders}>
            {app.open.size > 0 ? t("browse.collapseAll") : t("browse.expandAll")}
          </button>
          <button type="button" class="btn btn-ghost" onclick={toggleAll}>
            {allTicked ? t("browse.uncheckAll") : t("browse.checkAll")}
          </button>
        </div>

        <div
          class="text-muted flex shrink-0 items-center px-4.5 py-2 text-[11px] font-semibold uppercase"
          style="letter-spacing:0.08em;border-bottom:1px solid var(--color-divider)"
          aria-hidden="true"
        >
          <span class="flex-1">{t("browse.column.name")}</span>
          <span class="w-20">{t("browse.column.type")}</span>
          <span class="w-28 text-right">{t("browse.column.size")}</span>
          <span class="w-32 text-right">{t("browse.column.offset")}</span>
        </div>

        <div class="min-h-0 flex-1">
          {#if app.rows.length === 0}
            <p class="text-muted p-6 text-[13px]">
              {app.filter ? t("browse.noMatch") : t("browse.emptyArchive")}
            </p>
          {:else}
            <EntryTree onextract={(paths) => onextract(paths)} />
          {/if}
        </div>
      </div>

      <SelectionPanel {onextract} {onpickdest} />
    </div>

    <div class="bar-bottom nums flex h-11 shrink-0 items-center gap-3.5 px-4.5 text-[12px]">
      <span>{tn("count.entry", app.archive.entry_count)}</span>
      <span class="divider-x my-4"></span>
      <span>
        {t("browse.extracted", {
          size: formatSize(app.archive.total_size),
          file: formatSize(app.archive.file_size),
        })}
      </span>
      {#if app.selectionSize > 0}
        <span class="divider-x my-4"></span>
        <span class="text-accent">
          {tn("count.checked", app.selectionSize)} · {formatSize(app.selection.size)}
        </span>
      {/if}
      <div class="ml-auto flex items-center gap-2">
        <span class="tag tag-outline">{app.archive.version}</span>
        {#if app.archive.rejected.length === 0}
          <span class="tag tag-accent">{t("browse.indexValid")}</span>
        {:else}
          <span class="tag tag-warn">{tn("count.rejected", app.archive.rejected.length)}</span>
        {/if}
      </div>
    </div>
  {/if}
</div>
