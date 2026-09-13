<script lang="ts">
  /**
   * Preferences.
   *
   * Only settings that change what the application does are here. The design
   * also listed a compression section, which has nothing to act on: an .rpa
   * stores its files uncompressed.
   */
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";

  import logo from "../../assets/logo-256.png";
  import Icon from "../Icon.svelte";
  import { LANGUAGES, t, type MessageKey } from "../i18n.svelte";
  import { DEFAULTS, WRITABLE_VERSIONS } from "../settings";
  import { app } from "../store.svelte";

  /** Sections of the sidebar, in the order they are offered. */
  const SECTIONS = [
    { id: "general", label: "settings.general" },
    { id: "archives", label: "settings.archives" },
    { id: "about", label: "settings.about" },
  ] as const;

  /** Themes offered. */
  const THEMES = [
    { id: "systeme", label: "settings.theme.system" },
    { id: "clair", label: "settings.theme.light" },
    { id: "sombre", label: "settings.theme.dark" },
  ] as const;

  let section = $state<(typeof SECTIONS)[number]["id"]>("general");
  let version = $state("");

  onMount(async () => {
    // The version the bundle was built with, read from the application itself
    // rather than repeated in the interface where it would drift.
    try {
      version = await getVersion();
    } catch {
      version = "";
    }
  });

  /** Catalog key naming a language in the current language. */
  function languageKey(code: string): MessageKey {
    return `lang.${code}` as MessageKey;
  }
</script>

<div class="grid min-h-0 flex-1" style="grid-template-columns:238px 1fr">
  <div class="sidebar flex min-h-0 flex-col overflow-y-auto px-2.5 py-3.5">
    <nav aria-label={t("settings.sections")} class="flex flex-col">
      {#each SECTIONS as item (item.id)}
        <button
          type="button"
          class="rounded-[var(--radius-sm)] px-3 py-2.5 text-left text-[14px]"
          style={section === item.id
            ? "background:var(--color-accent-fill);color:var(--color-on-accent);box-shadow:var(--shadow-accent)"
            : ""}
          aria-current={section === item.id ? "page" : undefined}
          onclick={() => (section = item.id)}
        >
          {t(item.label)}
        </button>
      {/each}
    </nav>
  </div>

  <div class="flex min-h-0 flex-col overflow-y-auto p-7" style="max-width:680px">
    {#if section === "general"}
      <h2 class="m-0 mb-1 text-[25px]">{t("settings.general")}</h2>
      <p class="text-muted text-[13px]">{t("settings.general.intro")}</p>

      <div
        class="mt-3 rounded-[var(--radius-md)] px-4.5"
        style="border:1px solid var(--color-divider);background:var(--color-bg);box-shadow:var(--shadow-sm)"
      >
        <div class="grid items-center gap-5 py-4" style="grid-template-columns:1fr 260px">
          <div>
            <div class="heading text-[14px]">{t("settings.language")}</div>
            <p class="text-muted m-0 mt-0.5 text-[12px]">{t("settings.language.note")}</p>
          </div>
          <div class="seg justify-self-end" role="group" aria-label={t("settings.language")}>
            {#each LANGUAGES as code (code)}
              <button
                type="button"
                class="btn"
                aria-pressed={app.settings.language === code}
                onclick={() => app.updateSettings({ language: code })}
              >
                {t(languageKey(code))}
              </button>
            {/each}
          </div>
        </div>

        <div
          class="grid items-center gap-5 py-4"
          style="grid-template-columns:1fr 260px;border-top:1px solid var(--color-divider)"
        >
          <div>
            <div class="heading text-[14px]">{t("settings.theme")}</div>
            <p class="text-muted m-0 mt-0.5 text-[12px]">{t("settings.theme.note")}</p>
          </div>
          <div class="seg justify-self-end" role="group" aria-label={t("settings.theme")}>
            {#each THEMES as theme (theme.id)}
              <button
                type="button"
                class="btn"
                aria-pressed={app.settings.theme === theme.id}
                onclick={() => app.updateSettings({ theme: theme.id })}
              >
                {t(theme.label)}
              </button>
            {/each}
          </div>
        </div>

        <div
          class="grid items-center gap-5 py-4"
          style="grid-template-columns:1fr 260px;border-top:1px solid var(--color-divider)"
        >
          <div>
            <div class="heading text-[14px]">{t("settings.autoPreview")}</div>
            <p class="text-muted m-0 mt-0.5 text-[12px]">{t("settings.autoPreview.note")}</p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={app.settings.autoPreview}
            aria-label={t("settings.autoPreview")}
            class="switch justify-self-end"
            onclick={() => app.updateSettings({ autoPreview: !app.settings.autoPreview })}
          >
            <span class="knob"></span>
          </button>
        </div>
      </div>
    {:else if section === "archives"}
      <h2 class="m-0 mb-1 text-[25px]">{t("settings.archives")}</h2>
      <p class="text-muted text-[13px]">{t("settings.archives.intro")}</p>

      <div
        class="mt-3 rounded-[var(--radius-md)] px-4.5"
        style="border:1px solid var(--color-divider);background:var(--color-bg);box-shadow:var(--shadow-sm)"
      >
        <div class="grid items-start gap-5 py-4" style="grid-template-columns:1fr 260px">
          <div>
            <div class="heading text-[14px]">{t("settings.version")}</div>
            <p class="text-muted m-0 mt-0.5 text-[12px]">{t("settings.version.note")}</p>
          </div>
          <div class="seg justify-self-end" role="group" aria-label={t("settings.version")}>
            {#each WRITABLE_VERSIONS as writable (writable)}
              <button
                type="button"
                class="btn"
                aria-pressed={app.settings.defaultVersion === writable}
                onclick={() => app.updateSettings({ defaultVersion: writable })}
              >
                {writable.replace("RPA-", "")}
              </button>
            {/each}
          </div>
        </div>

        <div
          class="grid items-center gap-5 py-4"
          style="grid-template-columns:1fr 260px;border-top:1px solid var(--color-divider)"
        >
          <div>
            <div class="heading text-[14px]">{t("settings.verify")}</div>
            <p class="text-muted m-0 mt-0.5 text-[12px]">{t("settings.verify.note")}</p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={app.settings.verifyAfterBuild}
            aria-label={t("settings.verify")}
            class="switch justify-self-end"
            onclick={() => app.updateSettings({ verifyAfterBuild: !app.settings.verifyAfterBuild })}
          >
            <span class="knob"></span>
          </button>
        </div>
      </div>
    {:else}
      <div class="flex items-center gap-5">
        <img src={logo} alt="" width="104" height="104" />
        <div>
          <h2 class="m-0 text-[25px]">RenZip</h2>
          <p class="text-muted m-0 text-[13px]">{t("about.tagline")}</p>
          {#if version}
            <p class="nums m-0 mt-1 text-[13px]">{t("about.version", { version })}</p>
          {/if}
        </div>
      </div>

      <div
        class="mt-5 rounded-[var(--radius-md)] px-4.5 text-[13px]"
        style="border:1px solid var(--color-divider);background:var(--color-bg);box-shadow:var(--shadow-sm)"
      >
        <div class="flex justify-between gap-5 py-3">
          <span class="text-muted">{t("about.reads")}</span>
          <span class="nums">RPA-1.0 · RPA-2.0 · RPA-3.0 · RPA-3.2 · RPA-4.0</span>
        </div>
        <div
          class="flex justify-between gap-5 py-3"
          style="border-top:1px solid var(--color-divider)"
        >
          <span class="text-muted">{t("about.writes")}</span>
          <span class="nums">RPA-2.0 · RPA-3.0</span>
        </div>
        <div
          class="flex justify-between gap-5 py-3"
          style="border-top:1px solid var(--color-divider)"
        >
          <span class="text-muted">{t("about.license")}</span>
          <span>CeCILL-2.1</span>
        </div>
        <div class="py-3 text-[12px]" style="border-top:1px solid var(--color-divider)">
          <p class="text-muted m-0">{t("about.stack")}</p>
          <p class="text-muted m-0 mt-1">{t("about.rpyc")}</p>
        </div>
      </div>
    {/if}

    {#if section !== "about"}
      <div class="mt-5 flex gap-2">
        <button type="button" class="btn btn-secondary" onclick={() => app.updateSettings(DEFAULTS)}>
          <Icon name="arrowLeft" size={15} />
          {t("settings.reset")}
        </button>
      </div>
    {/if}
  </div>
</div>
