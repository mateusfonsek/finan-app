<script lang="ts">
  import { link, router } from "svelte-spa-router";
  import logoUrl from "$lib/assets/logo.png";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";

  const t = locale.t;

  let { onAbout }: { onAbout: () => void } = $props();

  type NavItem = { href: string; labelKey: string; sectionKey: string };

  const navItems: NavItem[] = [
    { sectionKey: "overview", href: "/dashboard", labelKey: "dashboard" },
    { sectionKey: "overview", href: "/transactions", labelKey: "transactions" },
    { sectionKey: "overview", href: "/calendar", labelKey: "calendar" },
    { sectionKey: "import", href: "/import", labelKey: "import" },
    { sectionKey: "organize", href: "/categories", labelKey: "categories" },
    { sectionKey: "organize", href: "/rules", labelKey: "rules" },
    { sectionKey: "organize", href: "/suggestions", labelKey: "suggestions" },
  ];

  const sections = ["overview", "import", "organize"];

  function isActive(href: string, current: string): boolean {
    if (href === "/dashboard" && (current === "/" || current === "/dashboard")) return true;
    return current === href;
  }
</script>

<aside class="bg-surface border-r border-border-subtle flex flex-col py-3 px-2.5 select-none">
  <button
    type="button"
    onclick={onAbout}
    title={t("sidebar.about_title")}
    aria-label={t("sidebar.about_title")}
    class="flex items-center gap-2.5 px-2 py-1.5 mb-2 rounded-lg text-left hover:bg-hover transition-colors"
  >
    <img
      src={logoUrl}
      alt="finan app"
      class="w-7 h-7 rounded-md shrink-0"
      draggable="false"
    />
    <div>
      <div class="text-[13.5px] font-semibold tracking-tight" style="font-family: var(--font-display)">finan app</div>
      <div class="text-[10px] text-fg-faint mt-px">{t("sidebar.tagline")}</div>
    </div>
  </button>

  {#each sections as section}
    <div class="mt-2.5 flex flex-col gap-px">
      <div class="text-[10.5px] font-semibold uppercase tracking-wider text-fg-faint px-2 pt-2 pb-1">
        {t("sidebar." + section)}
      </div>
      {#each navItems.filter((i) => i.sectionKey === section) as item}
        {@const active = isActive(item.href, router.location)}
        <a use:link
           href={item.href}
           class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium transition-colors {active ? 'bg-accent-soft text-fg' : 'text-fg-muted hover:bg-hover hover:text-fg'}">
          <span class="flex-1">{t("nav." + item.labelKey)}</span>
          {#if item.labelKey === "import" && watch.pendingCount > 0}
            <span
              class="min-w-[16px] h-4 px-1 rounded-full bg-accent text-[10px] font-semibold text-white grid place-items-center tabular"
            >
              {watch.pendingCount}
            </span>
          {/if}
        </a>
      {/each}
    </div>
  {/each}

  <div class="flex-1"></div>

  <a use:link href="/settings"
     class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium text-fg-muted hover:bg-hover hover:text-fg transition-colors">
    {t("nav.settings")}
  </a>
</aside>
