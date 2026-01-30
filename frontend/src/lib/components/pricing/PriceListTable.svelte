<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import type { PriceList } from '$lib/types/pricing';
	import {
		MoreHorizontal,
		Pencil,
		Trash2,
		Eye,
		Star,
		Copy,
		ToggleLeft,
		ToggleRight
	} from 'lucide-svelte';

	interface Props {
		priceLists: PriceList[];
		selectedIds?: Set<string>;
		onSelect?: (id: string, selected: boolean) => void;
		onSelectAll?: (selected: boolean) => void;
		onView?: (priceList: PriceList) => void;
		onEdit?: (priceList: PriceList) => void;
		onDelete?: (priceList: PriceList) => void;
		onDuplicate?: (priceList: PriceList) => void;
		onToggleActive?: (priceList: PriceList) => void;
		onSetDefault?: (priceList: PriceList) => void;
	}

	let {
		priceLists = [],
		selectedIds = new Set(),
		onSelect,
		onSelectAll,
		onView,
		onEdit,
		onDelete,
		onDuplicate,
		onToggleActive,
		onSetDefault
	}: Props = $props();

	const allSelected = $derived(
		priceLists.length > 0 && priceLists.every((pl) => selectedIds.has(pl.priceListId))
	);

	const someSelected = $derived(
		priceLists.some((pl) => selectedIds.has(pl.priceListId)) && !allSelected
	);

	function formatDate(date?: Date): string {
		if (!date) return '-';
		return new Date(date).toLocaleDateString('vi-VN', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric'
		});
	}

	function getValidityStatus(priceList: PriceList): {
		label: string;
		variant: 'default' | 'secondary' | 'destructive' | 'outline';
	} {
		const now = new Date();

		if (!priceList.validFrom && !priceList.validTo) {
			return { label: 'Always', variant: 'secondary' };
		}

		const from = priceList.validFrom ? new Date(priceList.validFrom) : null;
		const to = priceList.validTo ? new Date(priceList.validTo) : null;

		if (from && from > now) {
			return { label: 'Scheduled', variant: 'outline' };
		}

		if (to && to < now) {
			return { label: 'Expired', variant: 'destructive' };
		}

		return { label: 'Active', variant: 'default' };
	}

	function formatPercentage(value: number): string {
		if (value === 0) return '-';
		const sign = value > 0 ? '+' : '';
		return `${sign}${value}%`;
	}
</script>

<div class="rounded-md border">
	<Table.Root>
		<Table.Header>
			<Table.Row>
				<Table.Head class="w-12">
					<Checkbox
						checked={allSelected}
						indeterminate={someSelected}
						onCheckedChange={(checked) => onSelectAll?.(!!checked)}
					/>
				</Table.Head>
				<Table.Head>Name</Table.Head>
				<Table.Head>Code</Table.Head>
				<Table.Head>Type</Table.Head>
				<Table.Head class="text-center">Items</Table.Head>
				<Table.Head class="text-center">Adjustment</Table.Head>
				<Table.Head>Validity</Table.Head>
				<Table.Head>Status</Table.Head>
				<Table.Head class="w-12"></Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each priceLists as priceList (priceList.priceListId)}
				{@const validity = getValidityStatus(priceList)}
				<Table.Row class={selectedIds.has(priceList.priceListId) ? 'bg-muted/50' : ''}>
					<Table.Cell>
						<Checkbox
							checked={selectedIds.has(priceList.priceListId)}
							onCheckedChange={(checked) => onSelect?.(priceList.priceListId, !!checked)}
						/>
					</Table.Cell>
					<Table.Cell>
						<button
							type="button"
							class="flex flex-col items-start text-left hover:underline"
							onclick={() => onView?.(priceList)}
						>
							<span class="font-medium">
								{priceList.name}
								{#if priceList.isDefault}
									<Star class="ml-1 inline-block h-3 w-3 fill-yellow-400 text-yellow-400" />
								{/if}
							</span>
							{#if priceList.description}
								<span class="line-clamp-1 text-xs text-muted-foreground">
									{priceList.description}
								</span>
							{/if}
						</button>
					</Table.Cell>
					<Table.Cell>
						<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{priceList.code}</code>
					</Table.Cell>
					<Table.Cell>
						<Badge variant="outline" class="capitalize">
							{priceList.priceListType}
						</Badge>
					</Table.Cell>
					<Table.Cell class="text-center">
						{priceList.itemCount ?? 0}
					</Table.Cell>
					<Table.Cell class="text-center">
						<span
							class={priceList.defaultPercentage < 0
								? 'text-green-600'
								: priceList.defaultPercentage > 0
									? 'text-red-600'
									: 'text-muted-foreground'}
						>
							{formatPercentage(priceList.defaultPercentage)}
						</span>
					</Table.Cell>
					<Table.Cell>
						{#if priceList.validFrom || priceList.validTo}
							<div class="text-xs">
								<div>{formatDate(priceList.validFrom)} - {formatDate(priceList.validTo)}</div>
							</div>
						{:else}
							<span class="text-muted-foreground">Always</span>
						{/if}
					</Table.Cell>
					<Table.Cell>
						<div class="flex items-center gap-2">
							<Badge variant={validity.variant}>
								{validity.label}
							</Badge>
							{#if !priceList.isActive}
								<Badge variant="secondary">Inactive</Badge>
							{/if}
						</div>
					</Table.Cell>
					<Table.Cell>
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								{#snippet child({ props })}
									<Button {...props} variant="ghost" size="icon" class="h-8 w-8">
										<MoreHorizontal class="h-4 w-4" />
										<span class="sr-only">Open menu</span>
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>
							<DropdownMenu.Content align="end">
								<DropdownMenu.Item onclick={() => onView?.(priceList)}>
									<Eye class="mr-2 h-4 w-4" />
									View
								</DropdownMenu.Item>
								<DropdownMenu.Item onclick={() => onEdit?.(priceList)}>
									<Pencil class="mr-2 h-4 w-4" />
									Edit
								</DropdownMenu.Item>
								<DropdownMenu.Item onclick={() => onDuplicate?.(priceList)}>
									<Copy class="mr-2 h-4 w-4" />
									Duplicate
								</DropdownMenu.Item>
								<DropdownMenu.Separator />
								<DropdownMenu.Item onclick={() => onToggleActive?.(priceList)}>
									{#if priceList.isActive}
										<ToggleLeft class="mr-2 h-4 w-4" />
										Deactivate
									{:else}
										<ToggleRight class="mr-2 h-4 w-4" />
										Activate
									{/if}
								</DropdownMenu.Item>
								{#if !priceList.isDefault && priceList.priceListType === 'sale'}
									<DropdownMenu.Item onclick={() => onSetDefault?.(priceList)}>
										<Star class="mr-2 h-4 w-4" />
										Set as Default
									</DropdownMenu.Item>
								{/if}
								<DropdownMenu.Separator />
								<DropdownMenu.Item
									class="text-destructive focus:text-destructive"
									onclick={() => onDelete?.(priceList)}
								>
									<Trash2 class="mr-2 h-4 w-4" />
									Delete
								</DropdownMenu.Item>
							</DropdownMenu.Content>
						</DropdownMenu.Root>
					</Table.Cell>
				</Table.Row>
			{:else}
				<Table.Row>
					<Table.Cell colspan={9} class="h-24 text-center">No price lists found.</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
</div>
