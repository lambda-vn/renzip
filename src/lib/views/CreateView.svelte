<script lang="ts">
  /**
   * Build an archive out of a directory.
   *
   * The staged list is a real walk of the chosen directory, done by the same
   * code that will pack it, so what is shown here is what ends up inside.
   */
  import Icon from "../Icon.svelte";
  import { formatSize } from "../format";
  import { t, tn } from "../i18n.svelte";
  import { app } from "../store.svelte";

  type Props = {
    /** Ask for the source directory. */
    onpicksource: () => void;
    /** Ask where the archive should be written. */
    onpickdest: () => void;
    /** True while something is being dragged over the window. */
    dragging: boolean;
  };

  let { onpicksource, onpickdest, dragging }: Props = $props();

  /**
   * Versions offered, which are the ones that can actually be written.
   *
   * RPA-3.2 used to sit here greyed out; an option that can never be picked
   * is furniture, so it is gone. Reading a 3.2 archive still works.
   */
  const VERSIONS = [
    { id: "RPA-2.0", label: "2.0", note: "create.note.20" },
    { id: "RPA-3.0", label: "3.0", note: "create.note.30" },
  ] as const;

  let note = $derived(
    t(VERSIONS.find((version) => version.id === app.version)?.note ?? "create.note.30"),
  );

  /** Staged content, grouped by first folder the way the design shows it. */
  let staged = $derived.by(() => {
    const groups = new Map<string, { count: number; size: number }>();
    for (const file of app.source?.report.files ?? []) {
      const cut = file.path.indexOf("/");
      const key = cut === -1 ? "." : `${file.path.slice(0, cut)}/`;
      const group = groups.get(key) ?? { count: 0, size: 0 };
      group.count += 1;
      group.size += file.size;
      groups.set(key, group);
    }
    return [...groups.entries()]
      .map(([name, group]) => ({ name, ...group }))
      .sort((a, b) => a.name.localeCompare(b.name));
  });

  let totalSize = $derived(
    (app.source?.report.files ?? []).reduce((total, file) => total + file.size, 0),
  );
</script>

<div class="grid min-h-0 flex-1" style="grid-template-columns:1fr 350px">
  <div
    class="flex min-h-0 flex-col gap-4 overflow-y-auto p-5"
    style="border-right:1px solid var(--color-divider)"
  >
    <button
      type="button"
      class="flex flex-col items-start justify-center rounded-[var(--radius-lg)] p-9 text-left transition-[background,border-color,box-shadow] duration-150"
      style="min-height:250px;border:2px dashed {dragging
        ? 'var(--color-accent)'
        : 'var(--color-neutral-300)'};background:{dragging
        ? 'var(--color-accent-100)'
        : 'var(--color-surface)'};box-shadow:{dragging
        ? '0 0 0 4px var(--color-accent-200)'
        : 'none'}"
      onclick={onpicksource}
    >
      <span
        class="grid size-13 place-items-center rounded-[var(--radius-md)]"
        style="background:{dragging
          ? 'var(--color-accent-fill)'
          : 'var(--color-bg)'};color:{dragging
          ? 'var(--color-on-accent)'
          : 'var(--color-accent-text)'};box-shadow:var(--shadow-sm)"
      >
        <Icon name="upload" size={26} stroke={1.8} />
      </span>
      <h2 class="mt-4 mb-1.5 max-w-[16ch]">
        {dragging ? t("create.drop.titleActive") : t("create.drop.title")}
      </h2>
      <p class="m-0 max-w-[46ch] text-[13px]">
        {app.source ? app.source.dir : t("create.root")}
      </p>
      <span class="btn btn-secondary mt-4.5">{t("create.pick")}</span>
    </button>

    <div>
      <h2 class="m-0 mb-2.5 text-[13px] tracking-[0.08em] uppercase">{t("create.staged")}</h2>
      {#if staged.length === 0}
        <p class="text-muted text-[13px]">{t("create.none")}</p>
      {:else}
        <ul
          class="nums overflow-hidden rounded-[var(--radius-md)] text-[13px]"
          style="border:1px solid var(--color-divider);background:var(--color-bg);box-shadow:var(--shadow-sm)"
        >
          {#each staged as group, index (group.name)}
            <li
              class="flex items-center gap-2.5 px-3.5 py-2.5"
              style={index === 0 ? "" : "border-top:1px solid var(--color-divider)"}
            >
              <Icon name="folder" size={15} stroke={1.7} style="opacity:.7;flex:none" />
              <span class="truncate">{group.name}</span>
              <span class="text-muted">{tn("count.file", group.count)}</span>
              <span class="ml-auto">{formatSize(group.size)}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>

  <div class="aside-fill flex min-h-0 flex-col gap-4.5 overflow-y-auto p-5">
    <div class="field">
      <label for="dest-path">{t("create.dest")}</label>
      <div class="flex">
        <input
          id="dest-path"
          class="input truncate"
          value={app.destPath}
          readonly
          placeholder="archive.rpa"
          title={app.destPath}
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

    <div>
      <h2 class="m-0 mb-2 text-[13px] tracking-[0.08em] uppercase">{t("create.version")}</h2>
      <div class="seg" role="group" aria-label={t("create.versionAria")}>
        {#each VERSIONS as version (version.id)}
          <button
            type="button"
            class="btn"
            aria-pressed={app.version === version.id}
            title={t(version.note)}
            onclick={() => (app.version = version.id)}
          >
            {version.label}
          </button>
        {/each}
      </div>
      <p class="text-muted m-0 mt-2 text-[12px]">{note}</p>
    </div>

    <div>
      <h2 class="m-0 mb-2 text-[13px] tracking-[0.08em] uppercase">{t("create.options")}</h2>
      <label class="check" style="background:var(--color-bg)">
        <input
          type="checkbox"
          checked={app.settings.verifyAfterBuild}
          onchange={(event) => app.updateSettings({ verifyAfterBuild: event.currentTarget.checked })}
        />
        <span class="box"><Icon name="check" size={10} stroke={4} /></span>
        {t("create.verify")}
      </label>
    </div>

    <div class="nums mt-auto pt-4 text-[12px]" style="border-top:1px solid var(--color-divider)">
      <div class="flex justify-between py-1">
        <span class="text-muted">{t("create.input")}</span>
        <span>
          {formatSize(totalSize)} · {tn("count.file", app.source?.report.files.length ?? 0)}
        </span>
      </div>
      <div class="flex justify-between py-1">
        <span class="text-muted">{t("create.output")}</span>
        <span title={t("create.output.note")}>{formatSize(totalSize)} + index</span>
      </div>
      <button
        type="button"
        class="btn btn-primary btn-block"
        style="min-height:46px;padding-inline:16px"
        disabled={app.busy || !app.source || app.destPath === ""}
        onclick={() => app.runBuild()}
      >
        {t("create.build")}
        <Icon name="arrowRight" size={16} style="margin-left:auto" />
      </button>
    </div>
  </div>
</div>
