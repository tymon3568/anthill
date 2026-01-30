<script lang="ts">
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardDescription
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		type CreatePriceListInput,
		type PriceListType,
		type BasedOn,
		mockPriceLists
	} from '$lib/types/pricing';
	import { goto } from '$app/navigation';

	// Form state based on database schema
	let formData = $state<CreatePriceListInput>({
		name: '',
		code: '',
		description: '',
		currencyCode: 'VND',
		priceListType: 'sale',
		basedOn: 'fixed',
		parentPriceListId: undefined,
		defaultPercentage: 0,
		validFrom: undefined,
		validTo: undefined,
		priority: 100,
		isDefault: false,
		isActive: true
	});

	// UI state
	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	let submitError = $state('');
	let alwaysValid = $state(true);

	// Date binding helpers
	let validFromStr = $state('');
	let validToStr = $state('');

	// Auto-generate code from name
	let autoGenerateCode = $state(true);
	$effect(() => {
		if (autoGenerateCode && formData.name) {
			formData.code = formData.name
				.toUpperCase()
				.replace(/[^A-Z0-9]/g, '-')
				.replace(/-+/g, '-')
				.slice(0, 20);
		}
	});

	// Get available parent price lists (for "other_pricelist" basedOn)
	const availableParentLists = $derived(
		mockPriceLists.filter((pl) => pl.priceListType === formData.priceListType)
	);

	function validateForm(): boolean {
		errors = {};

		if (!formData.name.trim()) {
			errors.name = 'Name is required';
		} else if (formData.name.length > 255) {
			errors.name = 'Name must be 255 characters or less';
		}

		if (!formData.code.trim()) {
			errors.code = 'Code is required';
		} else if (formData.code.length > 50) {
			errors.code = 'Code must be 50 characters or less';
		} else if (!/^[A-Z0-9-]+$/.test(formData.code)) {
			errors.code = 'Code can only contain uppercase letters, numbers, and hyphens';
		}

		if (formData.basedOn === 'other_pricelist' && !formData.parentPriceListId) {
			errors.parentPriceListId = 'Please select a parent price list';
		}

		if (!alwaysValid) {
			if (validFromStr && validToStr) {
				const from = new Date(validFromStr);
				const to = new Date(validToStr);
				if (from > to) {
					errors.validTo = 'End date must be after start date';
				}
			}
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;
		submitError = '';

		try {
			// Convert dates
			const payload = {
				...formData,
				validFrom: !alwaysValid && validFromStr ? new Date(validFromStr) : undefined,
				validTo: !alwaysValid && validToStr ? new Date(validToStr) : undefined
			};

			// TODO: Call API
			console.log('Creating price list:', payload);

			// Simulate API call
			await new Promise((resolve) => setTimeout(resolve, 500));

			goto('/inventory/pricing/price-lists');
		} catch (error) {
			submitError = error instanceof Error ? error.message : 'Failed to create price list';
		} finally {
			isSubmitting = false;
		}
	}

	function handleCodeInput() {
		autoGenerateCode = false;
	}
</script>

<svelte:head>
	<title>New Price List - Anthill</title>
</svelte:head>

<div class="mx-auto max-w-3xl space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">New Price List</h1>
			<p class="text-muted-foreground">Create a new pricing structure for your products</p>
		</div>
		<Button variant="outline" href="/inventory/pricing/price-lists">Cancel</Button>
	</div>

	<form onsubmit={handleSubmit} class="space-y-6">
		<!-- Basic Information -->
		<Card>
			<CardHeader>
				<CardTitle>Basic Information</CardTitle>
				<CardDescription>Enter the price list's basic details</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="name">Name *</Label>
						<Input
							id="name"
							bind:value={formData.name}
							placeholder="Wholesale Pricing"
							class={errors.name ? 'border-destructive' : ''}
						/>
						{#if errors.name}
							<p class="text-sm text-destructive">{errors.name}</p>
						{/if}
					</div>
					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<Label for="code">Code *</Label>
							<label class="flex items-center gap-2 text-xs text-muted-foreground">
								<input type="checkbox" bind:checked={autoGenerateCode} class="rounded" />
								Auto-generate
							</label>
						</div>
						<Input
							id="code"
							bind:value={formData.code}
							placeholder="WHOLESALE"
							oninput={handleCodeInput}
							class={errors.code ? 'border-destructive' : ''}
						/>
						{#if errors.code}
							<p class="text-sm text-destructive">{errors.code}</p>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<Label for="description">Description</Label>
					<textarea
						id="description"
						bind:value={formData.description}
						placeholder="Special pricing for wholesale customers"
						rows="2"
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
					></textarea>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="priceListType">Type *</Label>
						<select
							id="priceListType"
							bind:value={formData.priceListType}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							<option value="sale">Sale</option>
							<option value="purchase">Purchase</option>
						</select>
						<p class="text-xs text-muted-foreground">
							{#if formData.priceListType === 'sale'}
								Pricing for selling products to customers
							{:else}
								Pricing for purchasing from suppliers
							{/if}
						</p>
					</div>
					<div class="space-y-2">
						<Label for="currencyCode">Currency *</Label>
						<select
							id="currencyCode"
							bind:value={formData.currencyCode}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							<option value="VND">VND - Vietnamese Dong</option>
							<option value="USD">USD - US Dollar</option>
						</select>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Pricing Method -->
		<Card>
			<CardHeader>
				<CardTitle>Pricing Method</CardTitle>
				<CardDescription>How prices are calculated in this price list</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-3">
					<label
						class="flex cursor-pointer items-start gap-3 rounded-lg border p-4 hover:bg-muted/50"
					>
						<input type="radio" bind:group={formData.basedOn} value="fixed" class="mt-1" />
						<div>
							<p class="font-medium">Fixed prices</p>
							<p class="text-sm text-muted-foreground">
								Set specific prices for each product in this list
							</p>
						</div>
					</label>

					<label
						class="flex cursor-pointer items-start gap-3 rounded-lg border p-4 hover:bg-muted/50"
					>
						<input type="radio" bind:group={formData.basedOn} value="base_price" class="mt-1" />
						<div>
							<p class="font-medium">Base price with adjustment</p>
							<p class="text-sm text-muted-foreground">
								Apply a percentage adjustment to the product's base price
							</p>
						</div>
					</label>

					<label
						class="flex cursor-pointer items-start gap-3 rounded-lg border p-4 hover:bg-muted/50"
					>
						<input
							type="radio"
							bind:group={formData.basedOn}
							value="other_pricelist"
							class="mt-1"
						/>
						<div>
							<p class="font-medium">Based on another price list</p>
							<p class="text-sm text-muted-foreground">
								Calculate prices based on another price list with adjustment
							</p>
						</div>
					</label>
				</div>

				{#if formData.basedOn === 'base_price' || formData.basedOn === 'other_pricelist'}
					<div class="space-y-4 rounded-lg bg-muted p-4">
						{#if formData.basedOn === 'other_pricelist'}
							<div class="space-y-2">
								<Label for="parentPriceListId">Parent Price List *</Label>
								<select
									id="parentPriceListId"
									bind:value={formData.parentPriceListId}
									class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm {errors.parentPriceListId
										? 'border-destructive'
										: ''}"
								>
									<option value="">Select a price list...</option>
									{#each availableParentLists as pl}
										<option value={pl.priceListId}>{pl.name} ({pl.code})</option>
									{/each}
								</select>
								{#if errors.parentPriceListId}
									<p class="text-sm text-destructive">{errors.parentPriceListId}</p>
								{/if}
							</div>
						{/if}

						<div class="space-y-2">
							<Label for="defaultPercentage">Default Adjustment (%)</Label>
							<div class="flex items-center gap-2">
								<Input
									id="defaultPercentage"
									type="number"
									step="0.1"
									bind:value={formData.defaultPercentage}
									class="max-w-[120px]"
								/>
								<span class="text-sm text-muted-foreground">%</span>
							</div>
							<p class="text-xs text-muted-foreground">
								Negative values = discount, positive = markup. Example: -15 means 15% discount from {formData.basedOn ===
								'base_price'
									? 'base price'
									: 'parent price list'}.
							</p>
						</div>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Validity -->
		<Card>
			<CardHeader>
				<CardTitle>Validity Period</CardTitle>
				<CardDescription>When this price list is active</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<label class="flex items-center gap-2">
					<input type="checkbox" bind:checked={alwaysValid} class="rounded" />
					<span>Always valid (no date restrictions)</span>
				</label>

				{#if !alwaysValid}
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="validFrom">Valid From</Label>
							<Input id="validFrom" type="date" bind:value={validFromStr} />
						</div>
						<div class="space-y-2">
							<Label for="validTo">Valid To</Label>
							<Input
								id="validTo"
								type="date"
								bind:value={validToStr}
								class={errors.validTo ? 'border-destructive' : ''}
							/>
							{#if errors.validTo}
								<p class="text-sm text-destructive">{errors.validTo}</p>
							{/if}
						</div>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Settings -->
		<Card>
			<CardHeader>
				<CardTitle>Settings</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="priority">Priority</Label>
						<Input id="priority" type="number" min="1" max="1000" bind:value={formData.priority} />
						<p class="text-xs text-muted-foreground">
							Lower number = higher priority. When multiple price lists apply, the one with lowest
							priority number wins.
						</p>
					</div>
				</div>

				<div class="flex flex-wrap gap-6 pt-2">
					<label class="flex items-center gap-2">
						<input type="checkbox" bind:checked={formData.isActive} class="rounded" />
						<span>Active</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="checkbox" bind:checked={formData.isDefault} class="rounded" />
						<span>Set as default price list</span>
					</label>
				</div>

				{#if formData.isDefault}
					<div
						class="rounded-md bg-yellow-50 p-3 text-sm text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-200"
					>
						Setting this as default will remove the default status from any existing default price
						list.
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Submit -->
		<div class="flex flex-col gap-4">
			{#if submitError}
				<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
					{submitError}
				</div>
			{/if}
			<div class="flex justify-end gap-4">
				<Button type="button" variant="outline" href="/inventory/pricing/price-lists">Cancel</Button
				>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						Creating...
					{:else}
						Create Price List
					{/if}
				</Button>
			</div>
		</div>
	</form>
</div>
