<script lang="ts">
  import { locale, LOCALE_CHOSEN_KEY } from "$lib/i18n/locale.svelte";
  import { setAppSetting } from "$lib/api/watch";
  import LocalePicker from "$lib/components/shell/LocalePicker.svelte";
  import { Button } from "$lib/components/ui/button";
  import logoUrl from "$lib/assets/logo.png";

  const t = locale.t;

  let { onConfirm }: { onConfirm: () => void } = $props();

  let busy = $state(false);

  // The picker already applies each pill on click (same control as Settings);
  // re-asserting the current code here just guarantees the pristine DB is
  // seeded from it even when the user never touched the picker at all.
  async function confirm() {
    busy = true;
    try {
      await locale.set(locale.code);
      await setAppSetting(LOCALE_CHOSEN_KEY, "1");
      onConfirm();
    } finally {
      busy = false;
    }
  }
</script>

<div class="h-screen flex flex-col">
  <!-- No page header exists on this screen either, so the window still needs
       a draggable strip at the top. -->
  <div data-tauri-drag-region class="w-full" style="height: var(--titlebar-h)"></div>

  <div class="flex-1 grid place-items-center px-8 pb-12">
    <div class="max-w-sm w-full flex flex-col items-center gap-5 text-center">
      <img
        src={logoUrl}
        alt=""
        draggable="false"
        class="w-16 h-16 rounded-[15px]"
        style="box-shadow: 0 8px 22px -10px oklch(0% 0 0 / 0.5)"
      />
      <div class="flex flex-col gap-2">
        <h1 class="text-display font-semibold text-fg text-balance">
          {t("onboarding.language_title")}
        </h1>
        <p class="text-body text-fg-muted leading-relaxed text-balance">
          {t("onboarding.language_desc")}
        </p>
      </div>

      <LocalePicker />

      <Button size="lg" onclick={confirm} disabled={busy}>
        {t("onboarding.language_continue")}
      </Button>
    </div>
  </div>
</div>
