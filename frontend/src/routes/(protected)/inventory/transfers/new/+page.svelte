<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { transferStore } from '$lib/stores/stock-movements.svelte';
	import {
		warehouseState,
		warehouseStore,
		productState,
		productStore
	} from '$lib/stores/inventory.svelte';
	import type {
		TransferType,
		TransferPriority,
		CreateTransferRequest,
		CreateTransferItemRequest,
		WarehouseResponse,
		ProductResponse
	} from '$lib/types/inventory';
	import { toast } from 'svelte-sonner';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import * as Table from '$lib/components/ui/table';
	import LocationSelector from '$lib/components/inventory/LocationSelector.svelte';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import SaveIcon from '@lucide/svelte/icons/save';

	// Form state
	let sourceWarehouseId = $state('');
	let destinationWarehouseId = $state('');
	let transferType = $state<TransferType>('manual');
	let priority = $state<TransferPriority>('normal');
	let referenceNumber = $state('');
	let reason = $state('');
	let expectedShipDate = $state('');
	let expectedReceiveDate = $state('');
	let shippingMethod = $state('');
	let notes = $state('');
	let items = $state<(CreateTransferItemRequest & { productName?: string })[]>([]);

	let isSubmitting = $state(false);
	let formError = $state<string | null>(null);

	// Options
	const transferTypes: { value: TransferType; label: string }[] = [
		{ value: 'manual', label: 'Manual Transfer' },
		{ value: 'auto_replenishment', label: 'Auto Replenishment' },
		{ value: 'emergency', label: 'Emergency' },
		{ value: 'consolidation', label: 'Consolidation' }
	];

	const priorities: { value: TransferPriority; label: string }[] = [
		{ value: 'low', label: 'Low' },
		{ value: 'normal', label: 'Normal' },
		{ value: 'high', label: 'High' },
		{ value: 'urgent', label: 'Urgent' }
	];

	// Local state for data from external stores (Svelte 5 cross-module reactivity workaround)
	let warehouses = $state<WarehouseResponse[]>([]);
	let products = $state<ProductResponse[]>([]);

	// Effect to sync external state to local state for reactivity
	$effect(() => {
		warehouses = warehouseState.items;
	});
	$effect(() => {
		products = productState.items;
	});

	let availableDestinations = $derived(
		warehouses.filter((w) => w.warehouseId !== sourceWarehouseId)
	);

	let isFormValid = $derived(
		sourceWarehouseId &&
			destinationWarehouseId &&
			items.length > 0 &&
			items.every((item) => item.productId && item.quantity > 0)
	);

	function addItem() {
		items = [
			...items,
			{
				productId: '',
				quantity: 1,
				uomId: '',
				lineNumber: items.length + 1,
				productName: ''
			}
		];
	}

	function removeItem(index: number) {
		items = items.filter((_, i) => i !== index).map((item, i) => ({ ...item, lineNumber: i + 1 }));
	}

	function updateItemProduct(index: number, productId: string) {
		const product = products.find((p) => p.productId === productId);
		items = items.map((item, i) =>
			i === index
				? {
						...item,
						productId,
						productName: product?.name || '',
						uomId: product?.defaultUomId || ''
					}
				: item
		);
	}

	function updateItemQuantity(index: number, quantity: number) {
		items = items.map((item, i) => (i === index ? { ...item, quantity } : item));
	}

	function updateItemSourceLocation(
		index: number,
		zoneId: string | null,
		locationId: string | null
	) {
		items = items.map((item, i) =>
			i === index ? { ...item, sourceZoneId: zoneId, sourceLocationId: locationId } : item
		);
	}

	function updateItemDestinationLocation(
		index: number,
		zoneId: string | null,
		locationId: string | null
	) {
		items = items.map((item, i) =>
			i === index ? { ...item, destinationZoneId: zoneId, destinationLocationId: locationId } : item
		);
	}

	async function handleSubmit() {
		if (!isFormValid) return;

		isSubmitting = true;
		formError = null;

		const request: CreateTransferRequest = {
			sourceWarehouseId,
			destinationWarehouseId,
			transferType,
			priority,
			referenceNumber: referenceNumber || undefined,
			reason: reason || undefined,
			expectedShipDate: expectedShipDate ? `${expectedShipDate}T00:00:00Z` : undefined,
			expectedReceiveDate: expectedReceiveDate ? `${expectedReceiveDate}T00:00:00Z` : undefined,
			shippingMethod: shippingMethod || undefined,
			notes: notes || undefined,
			items: items.map(
				({
					productId,
					quantity,
					uomId,
					lineNumber,
					sourceZoneId,
					sourceLocationId,
					destinationZoneId,
					destinationLocationId
				}) => ({
					productId,
					quantity,
					uomId: uomId || undefined,
					lineNumber,
					sourceZoneId: sourceZoneId || undefined,
					sourceLocationId: sourceLocationId || undefined,
					destinationZoneId: destinationZoneId || undefined,
					destinationLocationId: destinationLocationId || undefined
				})
			)
		};

		const result = await transferStore.create(request);

		if (result) {
			toast.success(`Transfer ${result.transferNumber} created successfully`);
			goto(`/inventory/transfers/${result.transferId}`);
		} else {
			formError = 'Failed to create transfer. Please try again.';
		}

		isSubmitting = false;
	}

	onMount(async () => {
		await Promise.all([warehouseStore.load(), productStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/inventory/transfers">
			<ArrowLeftIcon class="h-4 w-4" />
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Stock Transfer</h1>
			<p class="text-muted-foreground">Create a new transfer between warehouses</p>
		</div>
	</div>

	{#if formError}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{formError}</p>
		</div>
	{/if}

	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
		class="space-y-6"
	>
		<!-- Transfer Details -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Transfer Details</Card.Title>
				<Card.Description>Specify the source and destination warehouses</Card.Description>
			</Card.Header>
			<Card.Content class="grid gap-6 md:grid-cols-2">
				<div class="space-y-2">
					<Label for="source">Source Warehouse *</Label>
					<Select.Root type="single" onValueChange={(v) => (sourceWarehouseId = v ?? '')}>
						<Select.Trigger id="source">
							{warehouses.find((w) => w.warehouseId === sourceWarehouseId)?.warehouseName ||
								'Select source warehouse'}
						</Select.Trigger>
						<Select.Content>
							{#each warehouses as warehouse}
								<Select.Item value={warehouse.warehouseId}>{warehouse.warehouseName}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="destination">Destination Warehouse *</Label>
					<Select.Root type="single" onValueChange={(v) => (destinationWarehouseId = v ?? '')}>
						<Select.Trigger id="destination">
							{warehouses.find((w) => w.warehouseId === destinationWarehouseId)?.warehouseName ||
								'Select destination warehouse'}
						</Select.Trigger>
						<Select.Content>
							{#each availableDestinations as warehouse}
								<Select.Item value={warehouse.warehouseId}>{warehouse.warehouseName}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="type">Transfer Type</Label>
					<Select.Root
						type="single"
						onValueChange={(v) => (transferType = (v as TransferType) ?? 'manual')}
					>
						<Select.Trigger id="type">
							{transferTypes.find((t) => t.value === transferType)?.label || 'Manual Transfer'}
						</Select.Trigger>
						<Select.Content>
							{#each transferTypes as type}
								<Select.Item value={type.value}>{type.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="priority">Priority</Label>
					<Select.Root
						type="single"
						onValueChange={(v) => (priority = (v as TransferPriority) ?? 'normal')}
					>
						<Select.Trigger id="priority">
							{priorities.find((p) => p.value === priority)?.label || 'Normal'}
						</Select.Trigger>
						<Select.Content>
							{#each priorities as p}
								<Select.Item value={p.value}>{p.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="reference">Reference Number</Label>
					<Input id="reference" bind:value={referenceNumber} placeholder="Optional reference" />
				</div>

				<div class="space-y-2">
					<Label for="shipping">Shipping Method</Label>
					<Input id="shipping" bind:value={shippingMethod} placeholder="e.g., Truck, Air, etc." />
				</div>

				<div class="space-y-2">
					<Label for="shipDate">Expected Ship Date</Label>
					<Input id="shipDate" type="date" bind:value={expectedShipDate} />
				</div>

				<div class="space-y-2">
					<Label for="receiveDate">Expected Receive Date</Label>
					<Input id="receiveDate" type="date" bind:value={expectedReceiveDate} />
				</div>

				<div class="space-y-2 md:col-span-2">
					<Label for="reason">Reason</Label>
					<Input id="reason" bind:value={reason} placeholder="Reason for transfer" />
				</div>

				<div class="space-y-2 md:col-span-2">
					<Label for="notes">Notes</Label>
					<Textarea id="notes" bind:value={notes} placeholder="Additional notes" rows={3} />
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Transfer Items -->
		<Card.Root>
			<Card.Header>
				<div class="flex items-center justify-between">
					<div>
						<Card.Title>Transfer Items</Card.Title>
						<Card.Description>Add products to transfer</Card.Description>
					</div>
					<Button type="button" variant="outline" onclick={addItem}>
						<PlusIcon class="mr-2 h-4 w-4" />
						Add Item
					</Button>
				</div>
			</Card.Header>
			<Card.Content>
				{#if items.length === 0}
					<div class="flex flex-col items-center justify-center py-8 text-center">
						<p class="mb-4 text-muted-foreground">No items added yet</p>
						<Button type="button" variant="outline" onclick={addItem}>
							<PlusIcon class="mr-2 h-4 w-4" />
							Add First Item
						</Button>
					</div>
				{:else}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head class="w-[50px]">#</Table.Head>
								<Table.Head>Product</Table.Head>
								<Table.Head class="w-[120px]">Qty</Table.Head>
								<Table.Head class="w-[200px]">Source Location</Table.Head>
								<Table.Head class="w-[200px]">Dest. Location</Table.Head>
								<Table.Head class="w-[80px]">Actions</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each items as item, index (index)}
								<Table.Row>
									<Table.Cell>{item.lineNumber}</Table.Cell>
									<Table.Cell>
										<Select.Root
											type="single"
											onValueChange={(v) => updateItemProduct(index, v ?? '')}
										>
											<Select.Trigger>
												{item.productName || 'Select product'}
											</Select.Trigger>
											<Select.Content>
												{#each products as product}
													<Select.Item value={product.productId}>
														{product.sku} - {product.name}
													</Select.Item>
												{/each}
											</Select.Content>
										</Select.Root>
									</Table.Cell>
									<Table.Cell>
										<Input
											type="number"
											min="1"
											value={item.quantity}
											onchange={(e) =>
												updateItemQuantity(
													index,
													parseInt((e.target as HTMLInputElement).value) || 1
												)}
										/>
									</Table.Cell>
									<Table.Cell>
										<LocationSelector
											warehouseId={sourceWarehouseId}
											label=""
											disabled={!sourceWarehouseId}
											onZoneChange={(zoneId) =>
												updateItemSourceLocation(index, zoneId, item.sourceLocationId ?? null)}
											onLocationChange={(locationId) =>
												updateItemSourceLocation(index, item.sourceZoneId ?? null, locationId)}
										/>
									</Table.Cell>
									<Table.Cell>
										<LocationSelector
											warehouseId={destinationWarehouseId}
											label=""
											disabled={!destinationWarehouseId}
											onZoneChange={(zoneId) =>
												updateItemDestinationLocation(
													index,
													zoneId,
													item.destinationLocationId ?? null
												)}
											onLocationChange={(locationId) =>
												updateItemDestinationLocation(
													index,
													item.destinationZoneId ?? null,
													locationId
												)}
										/>
									</Table.Cell>
									<Table.Cell>
										<Button
											type="button"
											variant="ghost"
											size="icon"
											onclick={() => removeItem(index)}
										>
											<TrashIcon class="h-4 w-4 text-destructive" />
										</Button>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Actions -->
		<div class="flex justify-end gap-4">
			<Button type="button" variant="outline" href="/inventory/transfers">Cancel</Button>
			<Button type="submit" disabled={!isFormValid || isSubmitting}>
				<SaveIcon class="mr-2 h-4 w-4" />
				{isSubmitting ? 'Creating...' : 'Create Transfer'}
			</Button>
		</div>
	</form>
</div>
