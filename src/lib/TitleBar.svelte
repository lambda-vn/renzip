<script lang="ts">
  /**
   * Header row of every screen: what is open, and the way out.
   *
   * The design draws a window frame with its own traffic lights, a badge and
   * the application name; all three belong to the operating system's frame,
   * which this window keeps rather than redraw (snapping and the window menu
   * are worth more than a set of dots). What is left here is what the frame
   * does not already say: which archive, or which screen.
   */
  import Icon from "./Icon.svelte";
  import { t } from "./i18n.svelte";

  type Props = {
    /** Screen name shown after the divider, when there is one. */
    subtitle?: string;
    /** Name of the open archive. */
    archiveName?: string;
    /** Folder the archive sits in, shown as a tag. */
    archivePath?: string;
    /** Called when the back arrow is pressed; no arrow without it. */
    onback?: () => void;
    /** Called when the preferences button is pressed. */
    onsettings?: () => void;
  };

  let { subtitle, archiveName, archivePath, onback, onsettings }: Props = $props();
</script>

<header class="bar flex h-13 shrink-0 items-center gap-3.5 px-4.5">
  {#if onback}
    <button type="button" class="btn btn-ghost" onclick={onback}>
      <Icon name="arrowLeft" size={14} />
      {t("title.back")}
    </button>
  {/if}

  {#if subtitle}
    <span class="heading text-[15px]">{subtitle}</span>
  {:else if archiveName}
    <span class="heading text-[15px]">{archiveName}</span>
  {/if}
  {#if archivePath}
    <span class="tag tag-neutral max-w-100 truncate" title={archivePath}>{archivePath}</span>
  {/if}

  <div class="ml-auto flex items-center gap-2">
    {#if onsettings}
      <button
        type="button"
        class="btn btn-secondary btn-icon"
        onclick={onsettings}
        aria-label={t("title.settings")}
        title={t("title.settings")}
      >
        <Icon name="settings" size={16} stroke={1.7} />
      </button>
    {/if}
  </div>
</header>
