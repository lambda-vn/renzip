<script lang="ts" module>
  /** Every icon of the design, as the path data of a 24x24 stroked glyph. */
  const PATHS = {
    archive: ["M4 7h16v13H4z", "M4 7l2-3h12l2 3M10 12h4"],
    folder: [
      "M3 8a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.6.8l.8 1.2H19a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
    ],
    file: ["M14 3H7a1 1 0 0 0-1 1v16a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V7z", "M14 3v4h4"],
    chevron: ["m9 6 6 6-6 6"],
    music: ["M9 18V5l12-2v13", "M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0z", "M21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z"],
    video: ["M3 6a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z", "m16 10 5-3v10l-5-3z"],
    image: [
      "M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
      "M3 16l4.5-4.5a2 2 0 0 1 2.8 0L15 16",
      "m14 15 1.5-1.5a2 2 0 0 1 2.8 0L21 16",
      "M9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0z",
    ],
    font: ["M6 20h5", "M13 20h5", "M9 20 15 4h1l6 16", "M11 14h8"],
    code: ["m8 8-4 4 4 4", "m16 8 4 4-4 4", "m13 5-2 14"],
    binary: [
      "M12 3 3 7.5v9L12 21l9-4.5v-9z",
      "M3 7.5 12 12l9-4.5",
      "M12 12v9",
    ],
    search: ["m20 20-3.5-3.5"],
    copy: [
      "M9 9a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-8a2 2 0 0 1-2-2z",
      "M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1",
    ],
    external: [
      "M13 3h8v8",
      "M21 3 11 13",
      "M19 14v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h5",
    ],
    download: ["M12 3v12m0 0 4-4m-4 4-4-4", "M4 21h16"],
    upload: ["M12 16V4m0 0 4 4m-4-4-4 4", "M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3"],
    plus: ["M12 5v14M5 12h14"],
    arrowRight: ["M5 12h14m0 0-6-6m6 6-6 6"],
    arrowLeft: ["M19 12H5m0 0 6-6m-6 6 6 6"],
    check: ["m4 13 5 5L20 7"],
    settings: [
      "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
      "M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.6 1.6 0 0 0 9 19.4a1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 4.6 9a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3H9a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8V9a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z",
    ],
  } as const;

  /** Name of one of the icons above. */
  export type IconName = keyof typeof PATHS;

  export { PATHS };
</script>

<script lang="ts">
  /** A single stroked icon, decorative unless it is given a label. */
  type Props = {
    /** Which glyph to draw. */
    name: IconName;
    /** Width and height in pixels. */
    size?: number;
    /** Stroke width, matching the weight the design uses per context. */
    stroke?: number;
    /** Accessible name; without one the icon is hidden from assistive tools. */
    label?: string;
    /** Extra classes for positioning. */
    class?: string;
    /** Inline style, used for opacity and flex behaviour. */
    style?: string;
  };

  let {
    name,
    size = 15,
    stroke = 2,
    label,
    class: className = "",
    style = "",
  }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  {style}
  role={label ? "img" : "presentation"}
  aria-hidden={label ? undefined : "true"}
  aria-label={label}
>
  {#if name === "search"}
    <circle cx="11" cy="11" r="7" />
  {/if}
  {#each PATHS[name] as d (d)}
    <path {d} />
  {/each}
</svg>
