<script lang="ts">
  import Spinner from "./Spinner.svelte";
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;

  type Props = { label?: string; compact?: boolean };
  let { label, compact = false }: Props = $props();
</script>

<div
  class="flex items-center justify-center gap-2.5 text-fg-faint {compact ? 'py-6' : 'py-16'}"
  role="status"
>
  <Spinner size={15} />
  <span class="text-sub">{label ?? t("common.loading")}</span>
</div>
