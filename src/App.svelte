<script lang="ts">
  /**
   * Window shell: header, the screen currently shown, and the footer that
   * reports what the last operation did.
   *
   * The screens live in lib/views; everything they share is in
   * lib/store.svelte.ts.
   */
  import { onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";

  import TitleBar from "./lib/TitleBar.svelte";
  import { onProgress } from "./lib/api";
  import { formatSize } from "./lib/format";
  import { t, tn } from "./lib/i18n.svelte";
  import { app } from "./lib/store.svelte";
  import BrowseView from "./lib/views/BrowseView.svelte";
  import CreateView from "./lib/views/CreateView.svelte";
  import SettingsView from "./lib/views/SettingsView.svelte";

  let dragging = $state(false);

  let subtitle = $derived(
    app.view === "create"
      ? t("title.create")
      : app.view === "settings"
        ? t("title.settings")
        : undefined,
  );

  let percent = $derived(
    app.progress && app.progress.total > 0
      ? Math.round((app.progress.done / app.progress.total) * 100)
      : 0,
  );

  onMount(() => {
    app.init();
    const stops = [
      // A block body, not an arrow returning the assignment: Svelte warns
      // that such an expression evaluates to the right-hand side rather than
      // to the state it just wrote, and the warning fired on every event.
      subscribe(() =>
        onProgress((payload) => {
          app.progress = payload;
        }),
      ),
      subscribe(() =>
        getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "over" || event.payload.type === "enter") {
            dragging = true;
            return;
          }
          if (event.payload.type === "leave") {
            dragging = false;
            return;
          }
          dragging = false;
          void receive(event.payload.paths);
        }),
      ),
    ];
    return () => {
      for (const stop of stops) {
        void stop?.then((unlisten) => unlisten());
      }
    };
  });

  /**
   * Start one subscription to the backend without betting the window on it.
   *
   * A bridge that throws while mounting takes the whole render loop down with
   * it: the window then keeps its first frame and answers nothing, which is
   * an expensive way to lose a drag and drop listener. Seen for real with the
   * page opened outside of the application, where no bridge exists at all.
   *
   * @param start - Opens the subscription and resolves to its canceller.
   */
  function subscribe(start: () => Promise<UnlistenFn>): Promise<UnlistenFn> | null {
    try {
      const stopping = start();
      void stopping.catch((error: unknown) => {
        console.warn("renzip: a backend subscription failed", error);
      });
      return stopping;
    } catch (error) {
      console.warn("renzip: a backend subscription could not start", error);
      return null;
    }
  }

  /**
   * Take what was dropped on the window.
   *
   * An .rpa is opened; anything else is tried as a directory to pack, which
   * is also how a directory is told apart from a file here, since the drop
   * only carries paths.
   *
   * @param paths - What the drop carried.
   */
  async function receive(paths: string[]): Promise<void> {
    const archive = paths.find((path) => path.toLowerCase().endsWith(".rpa"));
    if (archive) {
      await app.load(archive);
      return;
    }
    if (paths.length === 0) {
      return;
    }
    await app.stage(paths[0]);
    if (app.source) {
      app.view = "create";
    }
  }

  /** Ask for an archive and open it. */
  async function chooseArchive(): Promise<void> {
    const chosen = await open({
      multiple: false,
      title: t("dialog.openArchive"),
      filters: [{ name: "Ren'Py .rpa", extensions: ["rpa"] }],
    });
    if (typeof chosen === "string") {
      await app.load(chosen);
    }
  }

  /** Ask for the directory a new archive will be built from. */
  async function chooseSource(): Promise<void> {
    const chosen = await open({ directory: true, title: t("dialog.source") });
    if (typeof chosen === "string") {
      await app.stage(chosen);
    }
  }

  /** Ask where the new archive should be written. */
  async function chooseDestPath(): Promise<void> {
    const chosen = await save({
      title: t("dialog.saveArchive"),
      defaultPath: app.destPath || "archive.rpa",
      filters: [{ name: "Ren'Py .rpa", extensions: ["rpa"] }],
    });
    if (typeof chosen === "string") {
      app.destPath = chosen;
    }
  }

  /** Ask where an extraction should write, defaulting to the last one picked. */
  async function chooseDestDir(): Promise<boolean> {
    const chosen = await open({
      directory: true,
      title: t("dialog.extractTo"),
      defaultPath: app.destDir || undefined,
    });
    if (typeof chosen === "string") {
      app.destDir = chosen;
      return true;
    }
    return false;
  }

  /**
   * Extract entries right away, always asking where to.
   *
   * Every archive can go somewhere different, so the destination is never
   * assumed from a previous extraction; the dialog just opens on it.
   *
   * @param paths - Entries to extract, or null for the whole archive.
   */
  async function extractNow(paths: string[] | null): Promise<void> {
    if (paths !== null && paths.length === 0) {
      return;
    }
    if (!(await chooseDestDir())) {
      return;
    }
    await app.runExtraction(paths);
  }
</script>

<div class="flex h-full flex-col">
  <TitleBar
    {subtitle}
    archiveName={app.view === "browse" ? app.archive?.name : undefined}
    archivePath={app.view === "browse" ? app.archive?.path : undefined}
    onback={app.view === "browse" ? undefined : () => (app.view = "browse")}
    onsettings={app.view === "settings" ? undefined : () => (app.view = "settings")}
  />

  <main class="flex min-h-0 flex-1 flex-col">
    {#if app.view === "browse"}
      <BrowseView
        onopen={chooseArchive}
        onextract={extractNow}
        onpickdest={() => void chooseDestDir()}
      />
    {:else if app.view === "create"}
      <CreateView onpicksource={chooseSource} onpickdest={chooseDestPath} {dragging} />
    {:else}
      <SettingsView />
    {/if}
  </main>

  <footer class="bar-bottom shrink-0 px-4.5 py-2.5 text-[12px]">
    {#if app.progress}
      <div class="mb-2 flex items-center gap-3.5">
        <div
          class="progress-track flex-1"
          role="progressbar"
          aria-label={app.progress.phase === "build"
            ? t("status.progress.build")
            : t("status.progress.extract")}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent}
        >
          <div class="progress-bar" style="width:{percent}%"></div>
        </div>
        <span class="nums text-muted w-64 truncate" title={app.progress.current}>
          {app.progress.done} / {app.progress.total} · {formatSize(app.progress.bytes_done)}
        </span>
        <button type="button" class="btn btn-secondary" onclick={() => app.cancel()}>
          {t("status.interrupt")}
        </button>
      </div>
    {/if}

    <p aria-live="polite" class="m-0 min-h-5">
      {#if app.status}
        <span class={app.status.tone === "warn" ? "text-accent" : ""}>{app.status.text}</span>
      {:else if app.archive && app.view === "browse"}
        <span class="text-muted">
          {app.archive.version} · {tn("count.entry", app.archive.entry_count)}
        </span>
      {/if}
    </p>

    {#if app.archive && app.archive.multi_segment.length > 0 && app.view === "browse"}
      <p class="text-accent m-0 text-[11px]">
        {t("status.multiSegment", {
          count: tn("count.entry", app.archive.multi_segment.length),
        })}
      </p>
    {/if}

    {#if app.skipped.length > 0}
      <details class="mt-1 text-[11px]">
        <summary class="cursor-pointer">
          {t("status.skippedDetail", { count: app.skipped.length })}
        </summary>
        <ul class="selectable mt-1 max-h-32 overflow-auto">
          {#each app.skipped as item (item.path)}
            <li class="truncate"><code>{item.path}</code> : {item.reason}</li>
          {/each}
        </ul>
      </details>
    {/if}
  </footer>
</div>
