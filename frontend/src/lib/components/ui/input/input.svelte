<script lang="ts">
	import type { HTMLInputAttributes, HTMLInputTypeAttribute } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils.js';

	type InputType = Exclude<HTMLInputTypeAttribute, 'file'>;

	type Props = WithElementRef<
		Omit<HTMLInputAttributes, 'type'> &
			({ type: 'file'; files?: FileList } | { type?: InputType; files?: undefined })
	>;

	let {
		ref = $bindable(null),
		value = $bindable(),
		type,
		files = $bindable(),
		class: className,
		'data-slot': dataSlot = 'input',
		...restProps
	}: Props = $props();
</script>

{#if type === 'file'}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			'flex h-8 w-full min-w-0 rounded border border-gray-300 bg-white px-3 py-1.5 text-sm outline-none placeholder:text-gray-500 disabled:cursor-not-allowed disabled:opacity-50',
			'focus:border-gray-500 focus:outline-none',
			'aria-invalid:border-red-500',
			className
		)}
		type="file"
		bind:files
		bind:value
		{...restProps}
	/>
{:else}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			'flex h-8 w-full min-w-0 rounded border border-gray-300 bg-white px-3 py-1.5 text-sm outline-none placeholder:text-gray-500 disabled:cursor-not-allowed disabled:opacity-50',
			'focus:border-gray-500 focus:outline-none',
			'aria-invalid:border-red-500',
			className
		)}
		{type}
		bind:value
		{...restProps}
	/>
{/if}
