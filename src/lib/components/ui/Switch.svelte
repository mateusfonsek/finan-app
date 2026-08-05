<script lang="ts">
  type Props = {
    checked: boolean;
    onChange: (value: boolean) => void;
    disabled?: boolean;
    /** Required: a switch with no label says nothing about what it toggles. */
    label: string;
  };

  let { checked, onChange, disabled = false, label }: Props = $props();
</script>

<!-- Track colour and knob move in the same frame: one state change, not two. -->
<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onChange(!checked)}
  class="press relative w-[34px] h-[20px] shrink-0 rounded-full
         transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
         disabled:opacity-40
         {checked ? 'bg-accent' : 'bg-border'}"
>
  <span
    class="absolute top-[2px] left-[2px] w-[16px] h-[16px] rounded-full bg-white
           shadow-[0_1px_2px_rgba(0,0,0,.28)]
           transition-transform duration-[var(--dur-fast)] ease-[var(--ease-snap)]
           {checked ? 'translate-x-[14px]' : 'translate-x-0'}"
  ></span>
</button>
