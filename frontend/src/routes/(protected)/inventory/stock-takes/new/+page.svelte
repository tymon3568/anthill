<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { stockTakeStore } from '$lib/stores/stock-take.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import SaveIcon from '@lucide/svelte/icons/save';
	import LoaderIcon from '@lucide/svelte/icons/loader';

	// Form state
	let warehouseId = $state('');
	let notes = $state('');
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	// Form validation
	let isValid = $derived(!!warehouseId);

	// Submit form
	async function handleSubmit() {
		if (!isValid) return;

		isSubmitting = true;
		error = null;

		const result = await stockTakeStore.create({
			warehouseId,
			notes: notes || null
		});

		isSubmitting = false;

		if (result) {
			toast.success(`Stock take ${result.stockTakeNumber} created successfully`);
			// Navigate to the detail page to start counting
			goto(`/inventory/stock-takes/${result.stockTakeId}`);
		} else {
			toast.error('Failed to create stock take. Please try again.');
			error = 'Failed to create stock take. Please try again.';
		}
	}

	onMount(async () => {
		await warehouseStore.load();
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<a href="/inventory/stock-takes" class="hover:text-foreground">Stock Takes</a>
		<span>/</span>
		<span class="text-foreground">New</span>
	</div>

	<!-- Header -->
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/inventory/stock-takes">
			<ArrowLeftIcon class="h-4 w-4" />
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Stock Take</h1>
			<p class="text-muted-foreground">Start a new physical inventory count</p>
		</div>
	</div>

	<!-- Error -->
	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{error}</p>
		</div>
	{/if}

	<!-- Form -->
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
	>
		<div class="grid max-w-2xl gap-6">
			<!-- Stock Take Details -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Stock Take Details</Card.Title>
					<Card.Description>
						Select the warehouse to perform a stock take. The system will create a snapshot of
						current inventory levels for you to count against.
					</Card.Description>
				</Card.Header>
				<Card.Content class="grid gap-4">
					<div class="grid gap-2">
						<Label for="warehouse">Warehouse *</Label>
						<Select.Root type="single" onValueChange={(v) => (warehouseId = v ?? '')}>
							<Select.Trigger id="warehouse" class="w-full">
								{warehouseState.items.find((w) => w.warehouseId === warehouseId)?.warehouseName ||
									'Select warehouse...'}
							</Select.Trigger>
							<Select.Content>
								{#each warehouseState.items.filter((w) => w.isActive) as warehouse (warehouse.warehouseId)}
									<Select.Item value={warehouse.warehouseId}>
										{warehouse.warehouseName} ({warehouse.warehouseCode})
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
						<p class="text-sm text-muted-foreground">
							All products with inventory in this warehouse will be included in the stock take.
						</p>
					</div>

					<div class="grid gap-2">
						<Label for="notes">Notes (optional)</Label>
						<Textarea
							id="notes"
							placeholder="e.g., Monthly inventory count, Year-end audit..."
							bind:value={notes}
							rows={3}
						/>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Info Card -->
			<Card.Root class="bg-muted/50">
				<Card.Content class="pt-6">
					<h4 class="mb-2 font-medium">What happens next?</h4>
					<ul class="list-inside list-disc space-y-1 text-sm text-muted-foreground">
						<li>A stock take will be created in Draft status</li>
						<li>Current inventory levels will be snapshot as expected quantities</li>
						<li>You can then start counting and entering actual quantities</li>
						<li>
							When complete, the system will calculate variances and optionally create adjustments
						</li>
					</ul>
				</Card.Content>
			</Card.Root>

			<!-- Actions -->
			<div class="flex justify-end gap-4">
				<Button type="button" variant="outline" href="/inventory/stock-takes">Cancel</Button>
				<Button type="submit" disabled={!isValid || isSubmitting}>
					{#if isSubmitting}
						<LoaderIcon class="mr-2 h-4 w-4 animate-spin" />
						Creating...
					{:else}
						<SaveIcon class="mr-2 h-4 w-4" />
						Create Stock Take
					{/if}
				</Button>
			</div>
		</div>
	</form>
</div>
