<script lang="ts">
  import { push, router } from "svelte-spa-router";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import { toast as toastMotion } from "$lib/motion";
  import { watch } from "$lib/stores/watch.svelte";
  import {
    autoCollapse,
    autoCollapseArmed,
    collapse,
    expand,
    initialToastState,
    phaseOf,
    syncHash,
  } from "./toastState";

  const t = locale.t;

  /** Time until it collapses on its own. It does NOT disappear — it becomes a
   *  pill. */
  const AUTO_COLLAPSE_MS = 6000;

  // Not named `state`: `$state` would read as an auto-subscription to a store
  // called `state` (Svelte's `$` syntax), and the compiler complains about use
  // before declaration.
  let toast = $state(initialToastState);
  let hovering = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  let current = $derived(watch.discoveries[0] ?? null);

  // Aligning with the focused discovery is an effect, not a derivation:
  // `syncHash` preserves state when the hash is unchanged, so this is a no-op
  // on most renders and only resets when another file arrives.
  $effect(() => {
    toast = syncHash(toast, current?.hash ?? null);
  });

  let phase = $derived(phaseOf(toast, watch.suppressToast));

  $effect(() => {
    clearTimeout(timer);
    if (!autoCollapseArmed(toast, watch.suppressToast, hovering)) return;
    timer = setTimeout(() => (toast = autoCollapse(toast)), AUTO_COLLAPSE_MS);
    return () => clearTimeout(timer);
  });

  const IMPORT_ROUTE = "/import";

  function review() {
    if (!current) return;
    // The Import screen loads the file, via a signal in the store: navigating
    // to `/import` from `/import` does nothing (`push` fires no `hashchange`,
    // Import never remounts, and the click would be swallowed).
    watch.requestOpen(current);
    if (router.location !== IMPORT_ROUTE) void push(IMPORT_ROUTE);
    // Nothing is hidden here: resolving the discovery is what removes it from
    // the store, and the store decides what this notification shows. If opening
    // fails in Import, it stays pending — which is correct.
  }

  async function ignore() {
    if (!current) return;
    await watch.resolve(current.hash, "ignored");
  }
</script>

{#if current && phase !== "hidden"}
  <!-- A parallel notification, not modal: no scrim, because it interrupts
       nothing — the user can carry on with what they were doing. -->
  <!-- `grid` with no positioning: the ActivityCenter is what positions this.
       The grid stays because it is what stacks the two branches (expanded and
       pill) in the same cell, making the swap between them a cross-fade in
       place. -->
  <div
    class="grid"
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    role="presentation"
  >
    {#if phase === "expanded"}
      {@const meta = {
        n: current.txCount,
        from: current.earliest ?? "?",
        to: current.latest ?? "?",
      }}
      <div
        transition:toastMotion
        class="material-pop col-start-1 row-start-1 justify-self-end self-end
               w-[326px] p-3.5 flex flex-col gap-2 rounded-[var(--radius-xl)]"
        role="status"
      >
        <div class="flex items-start gap-2.5">
          <span
            class="w-7 h-7 shrink-0 grid place-items-center rounded-[var(--radius-md)] bg-accent-soft text-accent"
          >
            <Icon name="fileText" size={14} stroke={1.8} />
          </span>
          <span class="flex-1 min-w-0 flex flex-col gap-0.5 pt-px">
            <span class="text-callout font-semibold text-fg">
              {watch.pendingCount > 1
                ? t("watch.toast_title_many", { n: watch.pendingCount })
                : t("watch.toast_title")}
            </span>
            <span class="text-sub text-fg-muted truncate" title={current.fileName}>
              {current.fileName}
            </span>
          </span>
          <!-- Collapse now, without waiting for the timer. -->
          <button
            type="button"
            onclick={() => (toast = collapse(toast))}
            title={t("watch.toast_collapse")}
            aria-label={t("watch.toast_collapse")}
            class="press w-5 h-5 grid place-items-center rounded-full text-fg-faint
                   hover:text-fg hover:bg-hover transition-colors duration-[var(--dur-fast)]"
          >
            <Icon name="minus" size={11} stroke={2.4} />
          </button>
        </div>
        <div class="text-foot text-fg-subtle tabular pl-9.5">
          {current.txCount === 1
            ? t("watch.toast_meta_one", meta)
            : t("watch.toast_meta_many", meta)}
        </div>
        <div class="flex items-center justify-end gap-2 pt-1">
          <Button variant="ghost" size="sm" onclick={ignore}>{t("watch.toast_ignore")}</Button>
          <Button size="sm" onclick={review}>{t("watch.toast_review")}</Button>
        </div>
      </div>
    {:else}
      {@const pending =
        watch.pendingCount === 1
          ? t("watch.badge_pending_one", { n: watch.pendingCount })
          : t("watch.badge_pending_many", { n: watch.pendingCount })}
      <button
        type="button"
        transition:toastMotion
        onclick={() => (toast = expand(toast))}
        title={t("watch.toast_expand")}
        class="press material-pop col-start-1 row-start-1 justify-self-end self-end
               flex items-center gap-2 rounded-full pl-2 pr-3.5 h-8
               text-sub text-fg-muted hover:text-fg transition-colors duration-[var(--dur-fast)]"
      >
        <span class="w-5 h-5 grid place-items-center rounded-full bg-accent-soft text-accent">
          <Icon name="fileText" size={11} stroke={2} />
        </span>
        <span class="tabular">{pending}</span>
      </button>
    {/if}
  </div>
{/if}
