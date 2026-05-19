<script lang="ts">
  import { link, router } from "svelte-spa-router";
  import logoUrl from "$lib/assets/logo.png";

  type NavItem = { href: string; label: string; section?: string };

  const navItems: NavItem[] = [
    { section: "Visão geral", href: "/dashboard", label: "Dashboard" },
    { section: "Visão geral", href: "/transactions", label: "Transações" },
    { section: "Visão geral", href: "/calendar", label: "Calendário" },
    { section: "Importar", href: "/import", label: "Importar OFX" },
    { section: "Organizar", href: "/categories", label: "Categorias" },
    { section: "Organizar", href: "/rules", label: "Regras" },
    { section: "Organizar", href: "/suggestions", label: "Sugestões" },
  ];

  const sections = ["Visão geral", "Importar", "Organizar"];

  function isActive(href: string, current: string): boolean {
    if (href === "/dashboard" && (current === "/" || current === "/dashboard")) return true;
    return current === href;
  }
</script>

<aside class="bg-surface border-r border-border-subtle flex flex-col py-3 px-2.5 select-none">
  <div class="flex items-center gap-2.5 px-2 pb-3.5">
    <img
      src={logoUrl}
      alt="finan"
      class="w-7 h-7 rounded-md shrink-0"
      draggable="false"
    />
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
