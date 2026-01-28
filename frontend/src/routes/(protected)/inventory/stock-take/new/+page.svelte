<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { stockTakeStore } from '$lib/stores/stock-movements.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import { toast } from 'svelte-sonner';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import PlayIcon from '@lucide/svelte/icons/play';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';

	// Form state
	let warehouseId = $state('');
	let notes = $state('');
	let isSubmitting = $state(false);
	let formError = $state<string | null>(null);

	let isFormValid = $derived(!!warehouseId);

	async function handleSubmit() {
		if (!isFormValid) return;

		isSubmitting = true;
		formError = null;

		const result = await stockTakeStore.create({
			warehouseId,
			notes: notes || undefined
		});

		if (result) {
			toast.success(`Stock take ${result.stockTakeNumber} created successfully`);
			goto(`/inventory/stock-take/${result.stockTakeId}`);
		} else {
			formError = 'Failed to create stock take. Please try again.';
		}

		isSubmitting = false;
	}

	onMount(async () => {
		await warehouseStore.load();
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/inventory/stock-take">
			<ArrowLeftIcon class="h-4 w-4" />
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Stock Take</h1>
			<p class="text-muted-foreground">Start a physical inventory count session</p>
		</div>
	</div>

	{#if formError}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{formError}</p>
		</div>
	{/if}

	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-6 max-w-2xl">
		<!-- Stock Take Details -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Stock Take Details</Card.Title>
				<Card.Description>
					Select a warehouse to count. The system will snapshot current inventory levels.
				</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-6">
				<div class="space-y-2">
					<Label for="warehouse">Warehouse *</Label>
					<Select.Root type="single" onValueChange={(v) => (warehouseId = v ?? '')}>
						<Select.Trigger id="warehouse">
							{warehouseState.items.find((w) => w.warehouseId === warehouseId)?.warehouseName ||
								'Select warehouse'}
						</Select.Trigger>
						<Select.Content>
							{#each warehouseState.items as warehouse}
								<Select.Item value={warehouse.warehouseId}>
									{warehouse.warehouseName} ({warehouse.warehouseCode})
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					<p class="text-sm text-muted-foreground">
						All products with stock in this warehouse will be included in the count.
					</p>
				</div>

				<div class="space-y-2">
					<Label for="notes">Notes</Label>
					<Textarea
						id="notes"
						bind:value={notes}
						placeholder="Add any notes about this stock take (e.g., Monthly count, Annual audit)"
						rows={4}
					/>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Info Card -->
		<Card.Root class="bg-muted/50">
			<Card.Content class="pt-6">
				<div class="flex gap-4">
					<ClipboardListIcon class="h-10 w-10 text-muted-foreground shrink-0" />
					<div>
						<h3 class="font-semibold mb-2">How Stock Take Works</h3>
						<ol class="list-decimal list-inside space-y-1 text-sm text-muted-foreground">
							<li>System creates a snapshot of current inventory levels</li>
							<li>Count each product physically and enter actual quantities</li>
							<li>Review any discrepancies between expected and actual counts</li>
							<li>Finalize to automatically create inventory adjustments</li>
						</ol>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Actions -->
		<div class="flex justify-end gap-4">
			<Button type="button" variant="outline" href="/inventory/stock-take">Cancel</Button>
			<Button type="submit" disabled={!isFormValid || isSubmitting}>
				<PlayIcon class="mr-2 h-4 w-4" />
				{isSubmitting ? 'Creating...' : 'Start Stock Take'}
			</Button>
		</div>
	</form>
</div>
