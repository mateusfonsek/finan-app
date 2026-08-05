<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;

  /**
   * Só o que muda a decisão de quem está escolhendo o tipo. Sem cartão e sem
   * fundo: é texto de apoio, não um bloco de conteúdo disputando atenção com o
   * formulário logo abaixo.
   */
  const KINDS = ["expense", "income", "transfer"] as const;
</script>

<details class="group px-1">
  <summary
    class="text-foot text-fg-faint flex items-center gap-1.5 list-none marker:hidden w-fit
           hover:text-fg-muted transition-colors duration-[var(--dur-fast)]"
  >
    <Icon
      name="chevronRight"
      size={11}
      stroke={2.2}
      class="transition-transform duration-[var(--dur)] ease-[var(--ease-snap)] group-open:rotate-90"
    />
    <span>{t("kind_guide.title")}</span>
  </summary>

  <dl class="pt-2 pl-[18px] flex flex-col gap-1">
    {#each KINDS as k (k)}
      <div class="grid grid-cols-[92px_1fr] gap-2 items-baseline">
        <dt class="text-foot font-medium text-fg-muted">{t("kind." + k)}</dt>
        <dd class="text-foot text-fg-subtle leading-snug">{t("kind_guide." + k)}</dd>
      </div>
    {/each}
  </dl>
</details>
