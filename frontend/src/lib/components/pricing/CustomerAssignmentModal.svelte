<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import type { AssignCustomerInput, CustomerPriceList } from '$lib/types/pricing';
	import { Search, UserPlus } from 'lucide-svelte';

	interface Customer {
		id: string;
		name: string;
		email: string;
		group?: string;
	}

	interface Props {
		open: boolean;
		priceListId: string;
		assignedCustomers?: CustomerPriceList[];
		onClose: () => void;
		onAssign: (customerId: string, data: AssignCustomerInput) => void;
		onUnassign: (customerId: string) => void;
	}

	let {
		open = false,
		priceListId,
		assignedCustomers = [],
		onClose,
		onAssign,
		onUnassign
	}: Props = $props();

	// Search and selection state
	let searchQuery = $state('');
	let selectedCustomers = $state<Set<string>>(new Set());
	let priority = $state(0);
	let validFrom = $state('');
	let validTo = $state('');

	// Mock customer data
	const mockCustomers: Customer[] = [
		{ id: 'cust-001', name: 'ABC Corporation', email: 'abc@example.com', group: 'Enterprise' },
		{ id: 'cust-002', name: 'XYZ Trading', email: 'xyz@example.com', group: 'Wholesale' },
		{ id: 'cust-003', name: 'Tech Solutions Ltd', email: 'tech@example.com', group: 'Enterprise' },
		{ id: 'cust-004', name: 'Local Shop', email: 'local@example.com', group: 'Retail' },
		{ id: 'cust-005', name: 'Global Industries', email: 'global@example.com', group: 'Enterprise' },
		{ id: 'cust-006', name: 'Quick Mart', email: 'quick@example.com', group: 'Retail' }
	];

	// Filter customers based on search
	const filteredCustomers = $derived(
		mockCustomers.filter((customer) => {
			const query = searchQuery.toLowerCase();
			return (
				customer.name.toLowerCase().includes(query) ||
				customer.email.toLowerCase().includes(query) ||
				(customer.group?.toLowerCase().includes(query) ?? false)
			);
		})
	);

	// Get assigned customer IDs
	const assignedCustomerIds = $derived(new Set(assignedCustomers.map((ac) => ac.customerId)));

	// Available customers (not yet assigned)
	const availableCustomers = $derived(
		filteredCustomers.filter((c) => !assignedCustomerIds.has(c.id))
	);

	// Reset form when dialog opens
	$effect(() => {
		if (open) {
			searchQuery = '';
			selectedCustomers = new Set();
			priority = 0;
			validFrom = '';
			validTo = '';
		}
	});

	function toggleCustomer(customerId: string) {
		const newSelected = new Set(selectedCustomers);
		if (newSelected.has(customerId)) {
			newSelected.delete(customerId);
		} else {
			newSelected.add(customerId);
		}
		selectedCustomers = newSelected;
	}

	function handleAssign() {
		const data: AssignCustomerInput = {
			priority,
			validFrom: validFrom ? new Date(validFrom) : undefined,
			validTo: validTo ? new Date(validTo) : undefined
		};

		for (const customerId of selectedCustomers) {
			onAssign(customerId, data);
		}

		selectedCustomers = new Set();
	}

	function formatDate(date?: Date): string {
		if (!date) return '-';
		return new Date(date).toLocaleDateString('vi-VN');
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => !value && onClose()}>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-[700px]">
		<Dialog.Header>
			<Dialog.Title>Assign Customers</Dialog.Title>
			<Dialog.Description>Select customers to assign to this price list</Dialog.Description>
		</Dialog.Header>

		<div class="space-y-6">
			<!-- Currently Assigned -->
			{#if assignedCustomers.length > 0}
				<div class="space-y-2">
					<Label>Currently Assigned ({assignedCustomers.length})</Label>
					<div class="max-h-32 overflow-y-auto rounded-md border">
						<Table.Root>
							<Table.Body>
								{#each assignedCustomers as assignment (assignment.id)}
									{@const customer = mockCustomers.find((c) => c.id === assignment.customerId)}
									<Table.Row>
										<Table.Cell class="py-2">
											<div class="font-medium">{customer?.name ?? 'Unknown'}</div>
											<div class="text-xs text-muted-foreground">
												{customer?.email}
											</div>
										</Table.Cell>
										<Table.Cell class="py-2 text-center text-xs">
											Priority: {assignment.priority}
										</Table.Cell>
										<Table.Cell class="py-2 text-right">
											<Button
												variant="ghost"
												size="sm"
												class="h-7 text-xs text-destructive hover:text-destructive"
												onclick={() => onUnassign(assignment.customerId)}
											>
												Remove
											</Button>
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				</div>
			{/if}

			<!-- Search -->
			<div class="space-y-2">
				<Label>Search Customers</Label>
				<div class="relative">
					<Search class="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground" />
					<Input
						bind:value={searchQuery}
						placeholder="Search by name, email, or group..."
						class="pl-8"
					/>
				</div>
			</div>

			<!-- Available Customers -->
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<Label>Available Customers ({availableCustomers.length})</Label>
					{#if selectedCustomers.size > 0}
						<Badge variant="secondary">
							{selectedCustomers.size} selected
						</Badge>
					{/if}
				</div>
				<div class="max-h-48 overflow-y-auto rounded-md border">
					{#if availableCustomers.length > 0}
						<Table.Root>
							<Table.Body>
								{#each availableCustomers as customer (customer.id)}
									<Table.Row class={selectedCustomers.has(customer.id) ? 'bg-muted/50' : ''}>
										<Table.Cell class="w-10 py-2">
											<Checkbox
												checked={selectedCustomers.has(customer.id)}
												onCheckedChange={() => toggleCustomer(customer.id)}
											/>
										</Table.Cell>
										<Table.Cell class="py-2">
											<button
												type="button"
												class="text-left"
												onclick={() => toggleCustomer(customer.id)}
											>
												<div class="font-medium">{customer.name}</div>
												<div class="text-xs text-muted-foreground">
													{customer.email}
												</div>
											</button>
										</Table.Cell>
										<Table.Cell class="py-2 text-right">
											{#if customer.group}
												<Badge variant="outline" class="text-xs">
													{customer.group}
												</Badge>
											{/if}
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					{:else}
						<div class="p-4 text-center text-sm text-muted-foreground">
							{#if searchQuery}
								No customers found matching "{searchQuery}"
							{:else}
								All customers are already assigned
							{/if}
						</div>
					{/if}
				</div>
			</div>

			<!-- Assignment Options -->
			{#if selectedCustomers.size > 0}
				<div class="space-y-4 rounded-lg border bg-muted/50 p-4">
					<Label>Assignment Options</Label>
					<div class="grid grid-cols-3 gap-4">
						<div class="space-y-1">
							<Label class="text-xs">Priority</Label>
							<Input type="number" bind:value={priority} min={0} class="h-8" />
						</div>
						<div class="space-y-1">
							<Label class="text-xs">Valid From</Label>
							<Input type="date" bind:value={validFrom} class="h-8" />
						</div>
						<div class="space-y-1">
							<Label class="text-xs">Valid To</Label>
							<Input type="date" bind:value={validTo} class="h-8" />
						</div>
					</div>
				</div>
			{/if}
		</div>

		<Dialog.Footer class="mt-6">
			<Button type="button" variant="outline" onclick={onClose}>Cancel</Button>
			<Button type="button" disabled={selectedCustomers.size === 0} onclick={handleAssign}>
				<UserPlus class="mr-2 h-4 w-4" />
				Assign {selectedCustomers.size > 0 ? `(${selectedCustomers.size})` : ''}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
