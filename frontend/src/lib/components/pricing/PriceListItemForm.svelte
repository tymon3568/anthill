<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Badge } from '$lib/components/ui/badge';
	import type {
		CreatePriceListItemInput,
		UpdatePriceListItemInput,
		PriceListItem,
		ApplyTo,
		ComputeMethod,
		RoundingMethod
	} from '$lib/types/pricing';
	import { Plus, Trash2 } from 'lucide-svelte';

	interface QuantityTier {
		minQuantity: number;
		maxQuantity?: number;
		computeMethod: ComputeMethod;
		fixedPrice?: number;
		percentage?: number;
	}

	interface Props {
		open: boolean;
		item?: PriceListItem | null;
		basePrice?: number;
		currencyCode?: string;
		onClose: () => void;
		onSubmit: (data: CreatePriceListItemInput | CreatePriceListItemInput[]) => void;
	}

	let {
		open = false,
		item = null,
		basePrice = 1000000,
		currencyCode = 'VND',
		onClose,
		onSubmit
	}: Props = $props();

	const isEditing = $derived(!!item);

	// Form state
	let applyTo = $state<ApplyTo>('product');
	let productId = $state('');
	let variantId = $state('');
	let categoryId = $state('');
	let roundingMethod = $state<RoundingMethod>('none');
	let roundingPrecision = $state(0);

	// Quantity tiers
	let tiers = $state<QuantityTier[]>([
		{ minQuantity: 1, computeMethod: 'percentage', percentage: 0 }
	]);

	// Validation errors
	let errors = $state<Record<string, string>>({});

	// Initialize form from existing item
	let isInitialized = $state(false);

	$effect(() => {
		if (open && item && !isInitialized) {
			applyTo = item.applyTo;
			productId = item.productId ?? '';
			variantId = item.variantId ?? '';
			categoryId = item.categoryId ?? '';
			roundingMethod = item.roundingMethod;
			roundingPrecision = item.roundingPrecision;
			tiers = [
				{
					minQuantity: item.minQuantity,
					maxQuantity: item.maxQuantity,
					computeMethod: item.computeMethod,
					fixedPrice: item.fixedPrice,
					percentage: item.percentage
				}
			];
			isInitialized = true;
		} else if (open && !item && !isInitialized) {
			// Reset for new item
			applyTo = 'product';
			productId = '';
			variantId = '';
			categoryId = '';
			roundingMethod = 'none';
			roundingPrecision = 0;
			tiers = [{ minQuantity: 1, computeMethod: 'percentage', percentage: 0 }];
			isInitialized = true;
		}
	});

	// Reset initialization when dialog closes
	$effect(() => {
		if (!open) {
			isInitialized = false;
		}
	});

	// Mock data for selectors
	const mockProducts = [
		{ id: 'prod-001', name: 'Laptop Pro 15"', sku: 'LP-15' },
		{ id: 'prod-002', name: 'Wireless Mouse', sku: 'WM-01' },
		{ id: 'prod-003', name: 'USB-C Hub', sku: 'UC-HUB' }
	];

	const mockCategories = [
		{ id: 'cat-001', name: 'Electronics' },
		{ id: 'cat-002', name: 'Accessories' },
		{ id: 'cat-003', name: 'Office Supplies' }
	];

	function addTier() {
		const lastTier = tiers[tiers.length - 1];
		const newMinQty = lastTier ? (lastTier.maxQuantity ?? lastTier.minQuantity * 2) + 1 : 1;
		tiers = [
			...tiers,
			{
				minQuantity: newMinQty,
				computeMethod: lastTier?.computeMethod ?? 'percentage',
				percentage: lastTier?.percentage ?? 0
			}
		];
	}

	function removeTier(index: number) {
		if (tiers.length > 1) {
			tiers = tiers.filter((_, i) => i !== index);
		}
	}

	function updateTier(index: number, field: keyof QuantityTier, value: unknown) {
		tiers = tiers.map((tier, i) => {
			if (i === index) {
				return { ...tier, [field]: value };
			}
			return tier;
		});
	}

	function calculatePrice(tier: QuantityTier): number {
		if (tier.computeMethod === 'fixed' && tier.fixedPrice !== undefined) {
			return tier.fixedPrice;
		}
		if (tier.computeMethod === 'percentage' && tier.percentage !== undefined) {
			return basePrice * (1 + tier.percentage / 100);
		}
		return basePrice;
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: currencyCode,
			maximumFractionDigits: 0
		}).format(value);
	}

	function validate(): boolean {
		const newErrors: Record<string, string> = {};

		if (applyTo === 'product' && !productId) {
			newErrors.productId = 'Please select a product';
		}
		if (applyTo === 'variant' && !variantId) {
			newErrors.variantId = 'Please select a variant';
		}
		if (applyTo === 'category' && !categoryId) {
			newErrors.categoryId = 'Please select a category';
		}

		// Validate tiers
		for (let i = 0; i < tiers.length; i++) {
			const tier = tiers[i];
			if (tier.minQuantity < 1) {
				newErrors[`tier_${i}_minQty`] = 'Min quantity must be at least 1';
			}
			if (
				tier.computeMethod === 'fixed' &&
				(tier.fixedPrice === undefined || tier.fixedPrice < 0)
			) {
				newErrors[`tier_${i}_price`] = 'Fixed price is required';
			}
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	function handleSubmit() {
		if (!validate()) return;

		// Create items for each tier
		const items: CreatePriceListItemInput[] = tiers.map((tier) => ({
			applyTo,
			productId: applyTo === 'product' ? productId : undefined,
			variantId: applyTo === 'variant' ? variantId : undefined,
			categoryId: applyTo === 'category' ? categoryId : undefined,
			minQuantity: tier.minQuantity,
			maxQuantity: tier.maxQuantity,
			computeMethod: tier.computeMethod,
			fixedPrice: tier.computeMethod === 'fixed' ? tier.fixedPrice : undefined,
			percentage: tier.computeMethod === 'percentage' ? tier.percentage : undefined,
			roundingMethod,
			roundingPrecision
		}));

		onSubmit(isEditing ? items[0] : items);
		onClose();
	}

	function handleApplyToChange(value: string | undefined) {
		if (value) {
			applyTo = value as ApplyTo;
			productId = '';
			variantId = '';
			categoryId = '';
		}
	}

	function handleProductChange(value: string | undefined) {
		productId = value ?? '';
	}

	function handleCategoryChange(value: string | undefined) {
		categoryId = value ?? '';
	}

	function handleComputeMethodChange(index: number, value: string | undefined) {
		if (value) {
			updateTier(index, 'computeMethod', value as ComputeMethod);
		}
	}

	function handleRoundingChange(value: string | undefined) {
		if (value) {
			roundingMethod = value as RoundingMethod;
		}
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => !value && onClose()}>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-[600px]">
		<Dialog.Header>
			<Dialog.Title>{isEditing ? 'Edit Price List Item' : 'Add Price List Item'}</Dialog.Title>
			<Dialog.Description>
				Configure pricing for products, variants, or categories
			</Dialog.Description>
		</Dialog.Header>

		<form
			onsubmit={(e) => {
				e.preventDefault();
				handleSubmit();
			}}
			class="space-y-6"
		>
			<!-- Apply To -->
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>Apply To *</Label>
					<Select.Root type="single" value={applyTo} onValueChange={handleApplyToChange}>
						<Select.Trigger class="w-full">
							{#if applyTo === 'product'}
								Specific Product
							{:else if applyTo === 'variant'}
								Product Variant
							{:else if applyTo === 'category'}
								Product Category
							{:else}
								All Products
							{/if}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="product">Specific Product</Select.Item>
							<Select.Item value="variant">Product Variant</Select.Item>
							<Select.Item value="category">Product Category</Select.Item>
							<Select.Item value="all">All Products</Select.Item>
						</Select.Content>
					</Select.Root>
				</div>

				{#if applyTo === 'product'}
					<div class="space-y-2">
						<Label>Select Product *</Label>
						<Select.Root type="single" value={productId} onValueChange={handleProductChange}>
							<Select.Trigger class={errors.productId ? 'w-full border-destructive' : 'w-full'}>
								{#if productId}
									{mockProducts.find((p) => p.id === productId)?.name ?? 'Select product'}
								{:else}
									Select product
								{/if}
							</Select.Trigger>
							<Select.Content>
								{#each mockProducts as product (product.id)}
									<Select.Item value={product.id}>
										{product.name} ({product.sku})
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
						{#if errors.productId}
							<p class="text-sm text-destructive">{errors.productId}</p>
						{/if}
					</div>
				{/if}

				{#if applyTo === 'category'}
					<div class="space-y-2">
						<Label>Select Category *</Label>
						<Select.Root type="single" value={categoryId} onValueChange={handleCategoryChange}>
							<Select.Trigger class={errors.categoryId ? 'w-full border-destructive' : 'w-full'}>
								{#if categoryId}
									{mockCategories.find((c) => c.id === categoryId)?.name ?? 'Select category'}
								{:else}
									Select category
								{/if}
							</Select.Trigger>
							<Select.Content>
								{#each mockCategories as category (category.id)}
									<Select.Item value={category.id}>
										{category.name}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
						{#if errors.categoryId}
							<p class="text-sm text-destructive">{errors.categoryId}</p>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Quantity Tiers -->
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<Label>Quantity Tiers</Label>
					<Button type="button" variant="outline" size="sm" onclick={addTier}>
						<Plus class="mr-1 h-3 w-3" />
						Add Tier
					</Button>
				</div>

				<div class="space-y-3">
					{#each tiers as tier, index (index)}
						<div class="rounded-lg border p-3">
							<div class="mb-2 flex items-center justify-between">
								<Badge variant="secondary">Tier {index + 1}</Badge>
								{#if tiers.length > 1}
									<Button
										type="button"
										variant="ghost"
										size="icon"
										class="h-6 w-6"
										onclick={() => removeTier(index)}
									>
										<Trash2 class="h-3 w-3" />
									</Button>
								{/if}
							</div>

							<div class="grid grid-cols-2 gap-3">
								<div class="space-y-1">
									<Label class="text-xs">Min Qty</Label>
									<Input
										type="number"
										value={tier.minQuantity}
										oninput={(e) =>
											updateTier(
												index,
												'minQuantity',
												parseInt((e.target as HTMLInputElement).value) || 1
											)}
										min={1}
										class="h-8"
									/>
								</div>
								<div class="space-y-1">
									<Label class="text-xs">Max Qty (optional)</Label>
									<Input
										type="number"
										value={tier.maxQuantity ?? ''}
										oninput={(e) => {
											const val = (e.target as HTMLInputElement).value;
											updateTier(index, 'maxQuantity', val ? parseInt(val) : undefined);
										}}
										placeholder="Unlimited"
										class="h-8"
									/>
								</div>
							</div>

							<div class="mt-3 grid grid-cols-2 gap-3">
								<div class="space-y-1">
									<Label class="text-xs">Method</Label>
									<Select.Root
										type="single"
										value={tier.computeMethod}
										onValueChange={(v) => handleComputeMethodChange(index, v)}
									>
										<Select.Trigger class="h-8">
											{tier.computeMethod === 'fixed' ? 'Fixed Price' : 'Percentage'}
										</Select.Trigger>
										<Select.Content>
											<Select.Item value="fixed">Fixed Price</Select.Item>
											<Select.Item value="percentage">Percentage</Select.Item>
										</Select.Content>
									</Select.Root>
								</div>
								<div class="space-y-1">
									<Label class="text-xs">
										{tier.computeMethod === 'fixed' ? 'Price' : 'Adjustment (%)'}
									</Label>
									{#if tier.computeMethod === 'fixed'}
										<Input
											type="number"
											value={tier.fixedPrice ?? ''}
											oninput={(e) =>
												updateTier(
													index,
													'fixedPrice',
													parseFloat((e.target as HTMLInputElement).value) || 0
												)}
											placeholder="0"
											class="h-8"
										/>
									{:else}
										<Input
											type="number"
											value={tier.percentage ?? 0}
											oninput={(e) =>
												updateTier(
													index,
													'percentage',
													parseFloat((e.target as HTMLInputElement).value) || 0
												)}
											class="h-8"
										/>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- Price Preview -->
			<div class="rounded-lg border bg-muted/50 p-4">
				<Label class="mb-2 block">Price Preview (Base: {formatCurrency(basePrice)})</Label>
				<div class="space-y-1 text-sm">
					{#each tiers as tier, index (index)}
						{@const calculatedPrice = calculatePrice(tier)}
						{@const savings = basePrice - calculatedPrice}
						<div class="flex justify-between">
							<span>
								Qty {tier.minQuantity}{tier.maxQuantity ? `-${tier.maxQuantity}` : '+'}:
							</span>
							<span class={savings > 0 ? 'text-green-600' : savings < 0 ? 'text-red-600' : ''}>
								{formatCurrency(calculatedPrice)}
								{#if savings !== 0}
									<span class="text-xs">
										({savings > 0 ? 'save' : 'add'}
										{formatCurrency(Math.abs(savings))})
									</span>
								{/if}
							</span>
						</div>
					{/each}
				</div>
			</div>

			<!-- Rounding -->
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label>Rounding Method</Label>
					<Select.Root type="single" value={roundingMethod} onValueChange={handleRoundingChange}>
						<Select.Trigger class="w-full">
							{#if roundingMethod === 'none'}
								None
							{:else if roundingMethod === 'round_up'}
								Round Up
							{:else if roundingMethod === 'round_down'}
								Round Down
							{:else if roundingMethod === 'round_nearest'}
								Round Nearest
							{:else}
								Round to .99
							{/if}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="none">None</Select.Item>
							<Select.Item value="round_up">Round Up</Select.Item>
							<Select.Item value="round_down">Round Down</Select.Item>
							<Select.Item value="round_nearest">Round Nearest</Select.Item>
							<Select.Item value="round_to_99">Round to .99</Select.Item>
						</Select.Content>
					</Select.Root>
				</div>
				<div class="space-y-2">
					<Label>Precision</Label>
					<Input type="number" bind:value={roundingPrecision} min={0} max={4} placeholder="0" />
				</div>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={onClose}>Cancel</Button>
				<Button type="submit">
					{isEditing ? 'Update Item' : 'Add Item'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
