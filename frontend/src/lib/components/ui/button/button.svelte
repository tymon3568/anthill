<script lang="ts" module>
	import { cn, type WithElementRef } from '$lib/utils.js';
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';
	import { type VariantProps, tv } from 'tailwind-variants';

	export const buttonVariants = tv({
		base: "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap text-sm font-medium outline-none transition-colors disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 border border-transparent focus-visible:border-gray-300 focus-visible:outline-none",
		variants: {
			variant: {
				default: 'bg-gray-900 text-white hover:bg-gray-800',
				destructive: 'bg-red-600 text-white hover:bg-red-700',
				outline: 'border-gray-300 bg-white text-gray-700 hover:bg-gray-50',
				secondary: 'bg-gray-100 text-gray-900 hover:bg-gray-200',
				ghost: 'text-gray-700 hover:bg-gray-100',
				link: 'text-blue-600 underline-offset-4 hover:underline'
			},
			size: {
				default: 'h-8 px-3 py-1.5',
				sm: 'h-7 gap-1.5 px-2.5 py-1 text-xs',
				lg: 'h-9 px-4 py-2 text-base',
				icon: 'size-8',
				'icon-sm': 'size-7',
				'icon-lg': 'size-9'
			}
		},
		defaultVariants: {
			variant: 'default',
			size: 'default'
		}
	});

	export type ButtonVariant = VariantProps<typeof buttonVariants>['variant'];
	export type ButtonSize = VariantProps<typeof buttonVariants>['size'];

	export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
		WithElementRef<HTMLAnchorAttributes> & {
			variant?: ButtonVariant;
			size?: ButtonSize;
		};
</script>

<script lang="ts">
	let {
		class: className,
		variant = 'default',
		size = 'default',
		ref = $bindable(null),
		href = undefined,
		type = 'button',
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
		tabindex={disabled ? -1 : undefined}
		{...restProps}
	>
		<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
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
