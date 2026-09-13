<script lang="ts">
  import { onMount } from "svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { rise } from "$lib/motion";
  import { splitTools, cutoffLabel, WINDOW_OPTIONS } from "$lib/components/mcp/toolGroups";
  import {
    mcpRecentCalls,
    mcpStatus,
    setMcpEnabled,
    setMcpTool,
    setMcpWindow,
  } from "$lib/api/mcp";
  import type { CallEntry, McpStatus } from "$lib/bindings";

  const t = locale.t;

  let status = $state<McpStatus | null>(null);
  let calls = $state<CallEntry[]>([]);
  let error = $state<string | null>(null);
  let copied = $state(false);

  let groups = $derived(splitTools(status?.tools ?? []));
  let cutoff = $derived(cutoffLabel(status?.window_months ?? 0));

  async function load() {
    try {
      status = await mcpStatus();
      calls = await mcpRecentCalls();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(load);

  // The log only grows while the screen is open, so it polls instead of
  // needing an event of its own. Five seconds is slow enough to cost nothing
  // and fast enough that a call shows up while the user is still watching.
  onMount(() => {
    const id = setInterval(async () => {
      try {
        calls = await mcpRecentCalls();
        error = null;
      } catch (e) {
        error = e instanceof Error ? e.message : String(e);
      }
    }, 5000);
    return () => clearInterval(id);
  });

  async function apply(fn: () => Promise<McpStatus>) {
    try {
      status = await fn();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function copyUrl() {
    if (!status?.url) return;
    try {
      await navigator.clipboard.writeText(status.url);
      copied = true;
      setTimeout(() => (copied = false), 1200);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<Page title={t("mcp.title")} subtitle={t("mcp.subtitle")} width="narrow">
  {#if error}
    <ErrorNote message={error} />
  {/if}

  <Card title={t("mcp.connection")}>
    <div class="flex items-start justify-between gap-4">
      <div class="flex flex-col gap-1 min-w-0">
        <p class="text-sub text-fg-muted leading-relaxed">{t("mcp.connection_desc")}</p>
        {#if status?.enabled && status.port !== null}
          <span class="flex items-center gap-1.5 text-cap text-fg-subtle">
            <span class="w-1.5 h-1.5 rounded-full bg-accent shrink-0"></span>
            {t("mcp.listening", { port: status.port })}
          </span>
        {/if}
      </div>
      <Switch
        checked={status?.enabled ?? false}
        label={t("mcp.title")}
        onChange={(v) => apply(() => setMcpEnabled(v))}
      />
    </div>

    {#if status?.url}
      <p class="text-cap text-fg-subtle">{t("mcp.url_label")}</p>
      <div class="card-inset flex items-center gap-2 p-2 animate-rise-in">
        <span class="selectable flex-1 truncate font-mono text-cap text-fg" title={status.url}>
          {status.url}
        </span>
        <button
          type="button"
          onclick={copyUrl}
          class="press-sm flex items-center gap-1 px-2 h-6 rounded-[var(--radius-sm)]
                 text-cap font-medium text-fg-muted hover:bg-hover hover:text-fg
                 transition-colors duration-[var(--dur-fast)]"
        >
          <Icon name={copied ? "check" : "copy"} size={12} stroke={1.8} />
          {copied ? t("mcp.copied") : t("mcp.copy")}
        </button>
      </div>
      <p class="text-cap text-fg-subtle">{t("mcp.only_while_open")}</p>
    {/if}
  </Card>

  <Card title={t("mcp.tools")}>
    {#each [{ key: "read", tools: groups.read }, { key: "write", tools: groups.write }] as group}
      <div class="flex flex-col gap-1.5">
        <h3 class="section-title">{t("mcp.tools_" + group.key)}</h3>
        <p class="text-sub text-fg-subtle">{t("mcp.tools_" + group.key + "_desc")}</p>
        {#each group.tools as tool}
          <div class="flex items-start justify-between gap-4 py-1.5">
            <div class="flex flex-col gap-0.5 min-w-0">
              <span class="font-mono text-cap text-fg">{tool.name}</span>
              <span class="text-sub text-fg-subtle leading-relaxed">{tool.description}</span>
            </div>
            <Switch
              checked={tool.enabled}
              label={tool.name}
              onChange={(v) => apply(() => setMcpTool(tool.name, v))}
            />
          </div>
        {/each}
      </div>
    {/each}
  </Card>

  <Card title={t("mcp.window")}>
    <p class="text-sub text-fg-muted leading-relaxed">{t("mcp.window_desc")}</p>
    <div class="flex flex-wrap gap-1.5">
      {#each WINDOW_OPTIONS as months}
        {@const active = status?.window_months === months}
        <button
          type="button"
          onclick={() => apply(() => setMcpWindow(months))}
          aria-pressed={active}
          class="press-sm px-2.5 h-7 rounded-[var(--radius-md)] text-callout font-medium
                 transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
                 {active ? 'bg-accent text-accent-on' : 'text-fg-muted hover:bg-hover hover:text-fg'}"
        >
          {months === 0 ? t("mcp.window_none") : t("mcp.window_months", { n: months })}
        </button>
      {/each}
    </div>
    {#if status}
      <p class="text-cap text-fg-subtle">
        {cutoff ? t("mcp.window_cutoff", { month: cutoff }) : t("mcp.window_cutoff_none")}
      </p>
    {/if}
  </Card>

  <Card title={t("mcp.activity")} note={t("mcp.activity_desc", { n: 50 })}>
    {#if calls.length === 0}
      <EmptyState
        icon="clock"
        compact
        title={t("mcp.activity_empty")}
        description={t("mcp.activity_empty_desc")}
      />
    {:else}
      <ul class="flex flex-col">
        {#each calls as call (call.at + call.tool + call.args)}
          <li
            transition:rise
            class="flex items-baseline gap-2.5 py-1.5 border-b border-border-subtle last:border-0"
          >
            <span class="tabular text-cap text-fg-subtle shrink-0">{call.at.slice(11)}</span>
            <span class="font-mono text-cap text-fg shrink-0">{call.tool}</span>
            <span class="text-sub text-fg-subtle truncate flex-1" title={call.args}>
              {call.args}
            </span>
            {#if !call.ok}
              <span class="text-cap text-neg shrink-0" title={call.error ?? ""}>
                {t("mcp.activity_failed")}
              </span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </Card>
</Page>
