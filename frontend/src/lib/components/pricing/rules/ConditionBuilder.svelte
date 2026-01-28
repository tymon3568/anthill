<script lang="ts">
	import type { RuleConditions } from '$lib/types/pricing';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Card from '$lib/components/ui/card';
	import { Plus, X, Package, ShoppingCart, User, Calendar } from 'lucide-svelte';

	interface Category {
		id: string;
		name: string;
	}

	interface Props {
		conditions: RuleConditions;
		onchange?: (conditions: RuleConditions) => void;
		categories?: Category[];
		disabled?: boolean;
	}

	let { conditions = $bindable(), onchange, categories = [], disabled = false }: Props = $props();

	// Local form state
	let minQuantity = $state<number | undefined>(conditions.minQuantity);
	let maxQuantity = $state<number | undefined>(conditions.maxQuantity);
	let minOrderAmount = $state<number | undefined>(conditions.minOrderAmount);
	let maxOrderAmount = $state<number | undefined>(conditions.maxOrderAmount);
	let selectedCategories = $state<string[]>(conditions.categories || []);
	let selectedProducts = $state<string[]>(conditions.products || []);
	let selectedCustomerGroups = $state<string[]>(conditions.customerGroups || []);
	let firstOrderOnly = $state(conditions.firstOrderOnly || false);
	let validDays = $state<string[]>((conditions.validDays || []).map(String));
	let validHoursStart = $state<string>(String(conditions.validHoursStart ?? ''));
	let validHoursEnd = $state<string>(String(conditions.validHoursEnd ?? ''));

	const daysOfWeek = [
		{ value: 'monday', label: 'Mon' },
		{ value: 'tuesday', label: 'Tue' },
		{ value: 'wednesday', label: 'Wed' },
		{ value: 'thursday', label: 'Thu' },
		{ value: 'friday', label: 'Fri' },
		{ value: 'saturday', label: 'Sat' },
		{ value: 'sunday', label: 'Sun' }
	];

	function updateConditions() {
		const newConditions: RuleConditions = {};

		if (minQuantity) newConditions.minQuantity = minQuantity;
		if (maxQuantity) newConditions.maxQuantity = maxQuantity;
		if (minOrderAmount) newConditions.minOrderAmount = minOrderAmount;
		if (maxOrderAmount) newConditions.maxOrderAmount = maxOrderAmount;
		if (selectedCategories.length > 0) newConditions.categories = selectedCategories;
		if (selectedProducts.length > 0) newConditions.products = selectedProducts;
		if (selectedCustomerGroups.length > 0) newConditions.customerGroups = selectedCustomerGroups;
		if (firstOrderOnly) newConditions.firstOrderOnly = firstOrderOnly;
		if (validDays.length > 0) newConditions.validDays = validDays;
		if (validHoursStart) newConditions.validHoursStart = validHoursStart;
		if (validHoursEnd) newConditions.validHoursEnd = validHoursEnd;

		conditions = newConditions;
		onchange?.(newConditions);
	}

	function toggleCategory(categoryId: string) {
		if (disabled) return;
		if (selectedCategories.includes(categoryId)) {
			selectedCategories = selectedCategories.filter((id) => id !== categoryId);
		} else {
			selectedCategories = [...selectedCategories, categoryId];
		}
		updateConditions();
	}

	function toggleDay(day: string) {
		if (disabled) return;
		if (validDays.includes(day)) {
			validDays = validDays.filter((d) => d !== day);
		} else {
			validDays = [...validDays, day];
		}
		updateConditions();
	}

	function handleFirstOrderChange(checked: boolean) {
		firstOrderOnly = checked;
		updateConditions();
	}
</script>

<div class="space-y-6">
	<!-- Quantity Conditions -->
	<Card.Root>
		<Card.Header class="pb-3">
			<div class="flex items-center gap-2">
				<ShoppingCart class="h-4 w-4 text-muted-foreground" />
				<Card.Title class="text-sm font-medium">Quantity Conditions</Card.Title>
			</div>
		</Card.Header>
		<Card.Content>
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="minQuantity">Minimum Qty</Label>
					<Input
						id="minQuantity"
						type="number"
						bind:value={minQuantity}
						oninput={updateConditions}
						placeholder="Any"
						min={1}
						{disabled}
					/>
				</div>
				<div class="space-y-2">
					<Label for="maxQuantity">Maximum Qty</Label>
					<Input
						id="maxQuantity"
						type="number"
						bind:value={maxQuantity}
						oninput={updateConditions}
						placeholder="Any"
						min={1}
						{disabled}
					/>
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Order Amount Conditions -->
	<Card.Root>
		<Card.Header class="pb-3">
			<div class="flex items-center gap-2">
				<ShoppingCart class="h-4 w-4 text-muted-foreground" />
				<Card.Title class="text-sm font-medium">Order Amount</Card.Title>
			</div>
		</Card.Header>
		<Card.Content>
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="minOrderAmount">Min Order Amount</Label>
					<div class="flex items-center gap-2">
						<Input
							id="minOrderAmount"
							type="number"
							bind:value={minOrderAmount}
							oninput={updateConditions}
							placeholder="Any"
							min={0}
							{disabled}
						/>
						<span class="text-sm text-muted-foreground">VND</span>
					</div>
				</div>
				<div class="space-y-2">
					<Label for="maxOrderAmount">Max Order Amount</Label>
					<div class="flex items-center gap-2">
						<Input
							id="maxOrderAmount"
							type="number"
							bind:value={maxOrderAmount}
							oninput={updateConditions}
							placeholder="Any"
							min={0}
							{disabled}
						/>
						<span class="text-sm text-muted-foreground">VND</span>
					</div>
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Category Conditions -->
	{#if categories.length > 0}
		<Card.Root>
			<Card.Header class="pb-3">
				<div class="flex items-center gap-2">
					<Package class="h-4 w-4 text-muted-foreground" />
					<Card.Title class="text-sm font-medium">Categories</Card.Title>
				</div>
				<p class="text-xs text-muted-foreground">Apply only to items in these categories</p>
			</Card.Header>
			<Card.Content>
				<div class="flex flex-wrap gap-2">
					{#each categories as category (category.id)}
						<button
							type="button"
							class="rounded-full border px-3 py-1 text-sm transition-colors {disabled
								? 'cursor-not-allowed opacity-50'
								: ''} {selectedCategories.includes(category.id)
								? 'border-primary bg-primary text-primary-foreground'
								: 'hover:bg-muted'}"
							onclick={() => toggleCategory(category.id)}
							{disabled}
						>
							{category.name}
						</button>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>
	{/if}

	<!-- Customer Conditions -->
	<Card.Root>
		<Card.Header class="pb-3">
			<div class="flex items-center gap-2">
				<User class="h-4 w-4 text-muted-foreground" />
				<Card.Title class="text-sm font-medium">Customer Conditions</Card.Title>
			</div>
		</Card.Header>
		<Card.Content>
			<div class="flex items-center gap-2">
				<Checkbox
					id="firstOrderOnly"
					checked={firstOrderOnly}
					onCheckedChange={handleFirstOrderChange}
					{disabled}
				/>
				<Label for="firstOrderOnly" class="font-normal">First order only (new customers)</Label>
			</div>
		</Card.Content>
	</Card.Root>

	<!-- Time-based Conditions -->
	<Card.Root>
		<Card.Header class="pb-3">
			<div class="flex items-center gap-2">
				<Calendar class="h-4 w-4 text-muted-foreground" />
				<Card.Title class="text-sm font-medium">Time-based Conditions</Card.Title>
			</div>
		</Card.Header>
		<Card.Content class="space-y-4">
			<div class="space-y-2">
				<Label>Valid Days</Label>
				<div class="flex flex-wrap gap-2">
					{#each daysOfWeek as day (day.value)}
						<button
							type="button"
							class="rounded-lg border px-3 py-1 text-sm transition-colors {disabled
								? 'cursor-not-allowed opacity-50'
								: ''} {validDays.includes(day.value)
								? 'border-primary bg-primary text-primary-foreground'
								: 'hover:bg-muted'}"
							onclick={() => toggleDay(day.value)}
							{disabled}
						>
							{day.label}
						</button>
					{/each}
				</div>
				<p class="text-xs text-muted-foreground">Leave empty for all days</p>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="validHoursStart">Valid Hours Start</Label>
					<Input
						id="validHoursStart"
						type="time"
						bind:value={validHoursStart}
						oninput={updateConditions}
						{disabled}
					/>
				</div>
				<div class="space-y-2">
					<Label for="validHoursEnd">Valid Hours End</Label>
					<Input
						id="validHoursEnd"
						type="time"
						bind:value={validHoursEnd}
						oninput={updateConditions}
						{disabled}
					/>
				</div>
			</div>
		</Card.Content>
	</Card.Root>
</div>
