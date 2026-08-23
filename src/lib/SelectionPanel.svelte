<script lang="ts">
  /**
   * Right hand panel of the main window.
   *
   * A file clicked in the list is shown in full. With nothing clicked but
   * boxes ticked, what the selection adds up to is shown instead, since
   * rendering one file out of forty says nothing about the other thirty-nine.
   * The two never fight: ticking is done with the boxes, inspecting with a
   * click on the row.
   */
  import Icon from "./Icon.svelte";
  import { formatSize } from "./format";
  import { t, tn } from "./i18n.svelte";
  import { glyphOf, typeLabel } from "./kinds";
  import { app } from "./store.svelte";
  import { baseName } from "./tree";

  type Props = {
    /** Extract these entries, or the whole archive when given null. */
    onextract: (paths: string[] | null) => void;
    /** Ask for the destination directory. */
    onpickdest: () => void;
  };

  let { onextract, onpickdest }: Props = $props();

  let entry = $derived(app.current);
  let many = $derived(!entry && app.selectionSize > 0);
  let glyph = $derived(entry ? glyphOf(entry.path) : null);

  /** Entries the main extraction button acts on. */
  let target = $derived(app.selectionSize > 0 ? [...app.selected] : entry ? [entry.path] : []);

  /** Wording of that button, which says exactly what it will take. */
  let buttonLabel = $derived(
    app.selectionSize > 1
      ? t("panel.extract.many", { n: app.selectionSize })
      : app.selectionSize === 1 || entry
        ? t("panel.extract.one")
        : t("panel.extract.none"),
  );

  /** Offset as the design shows it: a padded hexadecimal word. */
  let offsetHex = $derived(
    entry ? `0x${entry.offset.toString(16).toUpperCase().padStart(8, "0")}` : "",
  );

  /**
   * Note under the icon of a file with nothing to show.
   *
   * The backend says why in its own words; only the size refusal is worth
   * repeating, since the icon already says what the file is.
   */
  function sizeNote(reason: string): string {
    return reason.includes("too large") ? t("panel.tooBig") : "";
  }
</script>

<aside
  class="flex w-73 shrink-0 flex-col gap-3.5 overflow-y-auto p-4"
  style="border-left:1px solid var(--color-divider)"
  aria-label={t("panel.title")}
>
  <h6 class="m-0">{t("panel.title")}</h6>

  {#if many}
    <div
      class="flex flex-col items-center gap-1 rounded-[var(--radius-md)] px-4 py-5"
      style="background:var(--color-accent-100);color:var(--color-accent-800)"
    >
      <span class="heading nums text-[32px] leading-none">{app.selection.count}</span>
      <span class="text-[12px] uppercase tracking-[0.08em]">
        {tn("count.selected", app.selection.count)}
      </span>
      <span class="nums mt-1 text-[13px]">{formatSize(app.selection.size)}</span>
    </div>

    <div class="card-inset px-3 py-1 text-[12px]">
      {#each app.selection.kinds as kind, index (kind.label)}
        <div
          class="flex justify-between gap-3 py-1.5"
          style={index === 0 ? "" : "border-top:1px solid var(--color-divider)"}
        >
          <span class="text-muted">{kind.label}</span>
          <span class="nums">{kind.count}</span>
        </div>
      {/each}
    </div>

    <ul class="selectable text-muted flex flex-col gap-0.5 text-[11px]">
      {#each app.selection.names as name (name)}
        <li class="truncate" title={name}>{name}</li>
      {/each}
      {#if app.selection.count > app.selection.names.length}
        <li class="opacity-70">
          {tn("count.other", app.selection.count - app.selection.names.length)}
        </li>
      {/if}
    </ul>
  {:else if !entry}
    <p class="text-muted text-[13px]">{t("panel.empty")}</p>
  {:else}
    <div
      class="grid w-full place-items-center overflow-hidden rounded-[var(--radius-md)]"
      style="aspect-ratio:16/9;background:linear-gradient(140deg,var(--color-neutral-300),var(--color-neutral-200));box-shadow:var(--shadow-sm)"
      aria-live="polite"
    >
      {#if app.previewLoading}
        <span
          class="text-[11px] uppercase tracking-[0.08em]"
          style="color:var(--color-neutral-700)"
        >
          {t("panel.reading")}
        </span>
      {:else if app.preview?.kind === "image"}
        <img
          src={app.preview.data_url}
          alt={t("panel.previewOf", { name: entry.path })}
          class="max-h-full max-w-full"
        />
      {:else if app.preview?.kind === "text"}
        <pre
          class="selectable size-full overflow-auto p-2 text-left font-mono text-[10px] leading-relaxed"
          style="color:var(--color-neutral-800)">{app.preview.content}</pre>
      {:else if glyph}
        <!-- No preview to draw: name what the file is rather than why the
             decoder gave up. -->
        <div
          class="flex flex-col items-center gap-1.5 px-4 text-center"
          style="color:var(--color-neutral-700)"
        >
          <Icon name={glyph.icon} size={34} stroke={1.5} />
          <span class="text-[11px] leading-tight">{t(glyph.label)}</span>
          {#if app.preview?.kind === "binary" && sizeNote(app.preview.reason)}
            <span class="text-[10px] opacity-75">{sizeNote(app.preview.reason)}</span>
          {:else if !app.preview}
            <button
              type="button"
              class="btn btn-ghost text-[11px]"
              onclick={() => app.show(entry, true)}
            >
              {t("panel.showPreview")}
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <div class="heading text-[15px] leading-tight break-all">{baseName(entry.path)}</div>

    <div class="card-inset selectable px-3 py-1 text-[12px]">
      <div class="flex justify-between gap-3 py-1.5">
        <span class="text-muted">{t("panel.path")}</span>
        <span class="truncate text-right" title={entry.path}>{entry.path}</span>
      </div>
      <div
        class="flex justify-between gap-3 py-1.5"
        style="border-top:1px solid var(--color-divider)"
      >
        <span class="text-muted">{t("panel.type")}</span><span>{typeLabel(entry.path)}</span>
      </div>
      <div
        class="flex justify-between gap-3 py-1.5"
        style="border-top:1px solid var(--color-divider)"
      >
        <span class="text-muted">{t("panel.size")}</span>
        <span class="nums">{formatSize(entry.size)}</span>
      </div>
      <div
        class="flex justify-between gap-3 py-1.5"
        style="border-top:1px solid var(--color-divider)"
      >
        <span class="text-muted">{t("panel.offset")}</span><span class="nums">{offsetHex}</span>
      </div>
      <div
        class="flex justify-between gap-3 py-1.5"
        style="border-top:1px solid var(--color-divider)"
      >
        <span class="text-muted">{t("panel.stored")}</span><span>{t("panel.stored.value")}</span>
      </div>
      {#if entry.prefix_len > 0}
        <div
          class="flex justify-between gap-3 py-1.5"
          style="border-top:1px solid var(--color-divider)"
        >
          <span class="text-muted">{t("panel.prefix")}</span>
          <span class="nums" title={t("panel.prefix.hint")}>{entry.prefix_len} o</span>
        </div>
      {/if}
    </div>
  {/if}

  <div class="mt-auto flex flex-col gap-3 pt-4" style="border-top:1px solid var(--color-divider)">
    <div class="field">
      <label for="dest-dir">{t("panel.destination")}</label>
      <div class="flex">
        <input
          id="dest-dir"
          class="input truncate"
          value={app.destDir}
          readonly
          placeholder={t("panel.destination.pick")}
          title={app.destDir}
          style="border-radius:var(--radius-md) 0 0 var(--radius-md)"
        />
        <button
          type="button"
          class="btn btn-secondary shrink-0"
          style="border-left:0;border-radius:0 var(--radius-md) var(--radius-md) 0"
          onclick={onpickdest}
        >
          {t("panel.browse")}
        </button>
      </div>
    </div>

    <div class="flex flex-col gap-1.5">
      <label class="radio">
        <input
          type="radio"
          name="layout"
          checked={app.layout === "Tree"}
          onchange={() => (app.layout = "Tree")}
        />
        <span class="dot"></span><span class="text-[13px]">{t("panel.layout.tree")}</span>
      </label>
      <label class="radio">
        <input
          type="radio"
          name="layout"
          checked={app.layout === "Flat"}
          onchange={() => (app.layout = "Flat")}
        />
        <span class="dot"></span><span class="text-[13px]">{t("panel.layout.flat")}</span>
      </label>
      {#if app.layout === "Flat"}
        <p class="text-muted m-0 text-[11px]">{t("panel.layout.flatNote")}</p>
      {/if}
    </div>

    <button
      type="button"
      class="btn btn-primary btn-block"
      style="min-height:42px"
      disabled={app.busy || target.length === 0}
      onclick={() => onextract(target)}
    >
      <Icon name="download" size={15} />
      {buttonLabel}
    </button>
    <button
      type="button"
      class="btn btn-secondary btn-block"
      style="margin-top:0"
      disabled={app.busy || !app.archive}
      onclick={() => onextract(null)}
    >
      {t("panel.extractAll", { count: tn("count.file", app.archive?.entry_count ?? 0) })}
    </button>
  </div>
</aside>
