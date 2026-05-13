<script lang="ts">
  import { link, router } from "svelte-spa-router";

  type NavItem = { href: string; label: string; section?: string };

  const navItems: NavItem[] = [
    { section: "Visão geral", href: "/dashboard", label: "Dashboard" },
    { section: "Visão geral", href: "/transactions", label: "Transações" },
    { section: "Importar", href: "/import", label: "Importar OFX" },
    { section: "Organizar", href: "/categories", label: "Categorias" },
  ];

  const sections = ["Visão geral", "Importar", "Organizar"];

  function isActive(href: string, current: string): boolean {
    if (href === "/dashboard" && (current === "/" || current === "/dashboard")) return true;
    return current === href;
  }
</script>

<aside class="bg-surface border-r border-border-subtle flex flex-col py-3 px-2.5 select-none">
  <div class="flex items-center gap-2 px-2 pb-3.5">
    <div class="w-[22px] h-[22px] rounded-md grid place-items-center"
         style="background: linear-gradient(180deg, var(--color-accent-hi), var(--color-accent));">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-3 h-3" style="color: var(--color-accent-on);">
        <path d="M11 20A7 7 0 0 1 4 13c0-7 7-9 16-9 0 9-2 16-9 16Z"/>
        <path d="M4 13c4-1 9-1 13-5"/>
      </svg>
    </div>
    <div>
      <div class="text-[13.5px] font-semibold tracking-tight" style="font-family: var(--font-display)">finan</div>
      <div class="text-[10px] text-fg-faint mt-px">100% local</div>
    </div>
  </div>

  {#each sections as section}
    <div class="mt-2.5 flex flex-col gap-px">
      <div class="text-[10.5px] font-semibold uppercase tracking-wider text-fg-faint px-2 pt-2 pb-1">
        {section}
      </div>
      {#each navItems.filter((i) => i.section === section) as item}
        {@const active = isActive(item.href, router.location)}
        <a use:link
           href={item.href}
           class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium transition-colors {active ? 'bg-accent-soft text-fg' : 'text-fg-muted hover:bg-hover hover:text-fg'}">
          {item.label}
        </a>
      {/each}
    </div>
  {/each}

  <div class="flex-1"></div>

  <a use:link href="/settings"
     class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium text-fg-muted hover:bg-hover hover:text-fg transition-colors">
    Configurações
  </a>
</aside>
