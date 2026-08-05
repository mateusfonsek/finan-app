<script lang="ts" module>
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
	import { type VariantProps, tv } from "tailwind-variants";

	// A button in the macOS spirit: standard control height (28px), control
	// radius (7px), the app's body type — and the press highlight happens on
	// PRESS, not on release (HIG: latency kills the sense of directness).
	// The recoil is a subtle scale, not a 1px offset — real objects sink under
	// the finger, they do not jump down.
	export const buttonVariants = tv({
		base: [
			"relative inline-flex shrink-0 items-center justify-center whitespace-nowrap select-none",
			"rounded-[var(--radius-md)] border border-transparent bg-clip-padding",
			"text-callout font-medium",
			"transition-[background-color,border-color,color,box-shadow,transform] duration-[var(--dur-fast)] ease-[var(--ease-snap)]",
			"outline-none active:scale-[0.97]",
			"disabled:pointer-events-none disabled:opacity-40",
			"[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-3.5",
		],
		variants: {
			variant: {
				// Primary action: filled with the accent. One per screen.
				default: "bg-accent text-accent-on hover:bg-accent-hi shadow-[var(--shadow-raised)]",
				// Secondary action: the standard macOS push button.
				outline:
					"border-border bg-surface-2 text-fg hover:bg-hover shadow-[var(--shadow-raised)]",
				secondary: "bg-surface-2 text-fg-muted hover:bg-hover hover:text-fg",
				// Tertiary: no frame until touched.
				ghost: "text-fg-muted hover:bg-hover hover:text-fg",
				destructive: "bg-neg/10 text-neg hover:bg-neg/18",
				link: "text-accent underline-offset-4 hover:underline",
			},
			size: {
				default: "h-7 gap-1.5 px-2.5",
				xs: "h-5.5 gap-1 rounded-[var(--radius-sm)] px-1.5 text-cap [&_svg:not([class*='size-'])]:size-3",
				sm: "h-6.5 gap-1 px-2 text-sub [&_svg:not([class*='size-'])]:size-3",
				lg: "h-8 gap-2 px-3.5 text-body",
				icon: "size-7",
				"icon-xs": "size-5.5 rounded-[var(--radius-sm)] [&_svg:not([class*='size-'])]:size-3",
				"icon-sm": "size-6.5 [&_svg:not([class*='size-'])]:size-3",
				"icon-lg": "size-8",
			},
		},
		defaultVariants: {
			variant: "default",
			size: "default",
		},
	});

	export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
	export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

	export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
		WithElementRef<HTMLAnchorAttributes> & {
			variant?: ButtonVariant;
			size?: ButtonSize;
		};
</script>

<script lang="ts">
	let {
		class: className,
		variant = "default",
		size = "default",
		ref = $bindable(null),
		href = undefined,
		type = "button",
		disabled,
		children,
		...restProps
	}: ButtonProps = $props();
</script>

{#if href}
	<a
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		href={disabled ? undefined : href}
		aria-disabled={disabled}
		role={disabled ? "link" : undefined}
		tabindex={disabled ? -1 : undefined}
		{...restProps}
	>
		{@render children?.()}
	</a>
{:else}
	<button
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		{type}
		{disabled}
		{...restProps}
	>
		{@render children?.()}
	</button>
{/if}
