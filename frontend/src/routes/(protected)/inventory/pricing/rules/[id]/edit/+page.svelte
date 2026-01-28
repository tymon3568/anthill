<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Card from '$lib/components/ui/card';
	import * as RadioGroup from '$lib/components/ui/radio-group';
	import type { PricingRule, UpdatePricingRuleInput, RuleType, ApplyOn } from '$lib/types/pricing';
	import {
		ArrowLeft,
		Percent,
		DollarSign,
		Tag,
		Gift,
		ShoppingCart,
		Package,
		Loader2
	} from 'lucide-svelte';

	// Get rule ID from params
	const ruleId = $derived(page.params.id);

	// Mock data - same as detail page
	const mockPricingRules: PricingRule[] = [
		{
			ruleId: 'rule-001',
			tenantId: 'tenant-001',
			name: 'Lunar New Year 2026',
			code: 'TET2026',
			description: 'Special discount for Lunar New Year celebration',
			ruleType: 'discount_percentage',
			conditions: {
				minOrderAmount: 500000,
				categories: ['cat-electronics']
			},
			discountPercentage: 15,
			maxDiscountAmount: 200000,
			validFrom: new Date('2026-01-15'),
			validTo: new Date('2026-02-15'),
			usageLimit: 1000,
			usageCount: 245,
			perCustomerLimit: 2,
			priority: 10,
			isCombinable: true,
			applyOn: 'order',
			isActive: true,
			createdAt: new Date('2025-12-20'),
			updatedAt: new Date('2026-01-10')
		},
		{
			ruleId: 'rule-002',
			tenantId: 'tenant-001',
			name: 'Buy 2 Get 1 Free - Accessories',
			code: 'B2G1FREE',
			description: 'Buy 2 accessories, get 1 free (lowest priced)',
			ruleType: 'buy_x_get_y',
			conditions: {
				categories: ['cat-accessories'],
				minQuantity: 3
			},
			buyQuantity: 2,
			getQuantity: 1,
			validFrom: new Date('2026-01-01'),
			validTo: new Date('2026-03-31'),
			priority: 20,
			isCombinable: false,
			exclusiveGroup: 'PROMO_ACCESSORIES',
			applyOn: 'line',
			isActive: true,
			createdAt: new Date('2025-12-15'),
			updatedAt: new Date('2025-12-15')
		},
		{
			ruleId: 'rule-003',
			tenantId: 'tenant-001',
			name: 'Flash Sale - 50k Off',
			code: 'FLASH50K',
			ruleType: 'discount_amount',
			conditions: {
				minOrderAmount: 300000
			},
			discountAmount: 50000,
			validFrom: new Date('2026-01-20'),
			validTo: new Date('2026-01-21'),
			usageLimit: 100,
			usageCount: 100,
			priority: 5,
			isCombinable: false,
			applyOn: 'order',
			isActive: false,
			createdAt: new Date('2026-01-19'),
			updatedAt: new Date('2026-01-21')
		}
	];

	// Find the rule
	const rule = $derived(mockPricingRules.find((r) => r.ruleId === ruleId));

	// Form state
	let name = $state('');
	let code = $state('');
	let description = $state('');
	let ruleType = $state<RuleType>('discount_percentage');

	// Discount values
	let discountPercentage = $state<number>(10);
	let discountAmount = $state<number>(0);
	let fixedPrice = $state<number>(0);
	let maxDiscountAmount = $state<number | undefined>(undefined);

	// Buy X Get Y
	let buyQuantity = $state<number>(2);
	let getQuantity = $state<number>(1);
	let freeQuantity = $state<number>(1);

	// Conditions
	let minQuantity = $state<number | undefined>(undefined);
	let minOrderAmount = $state<number | undefined>(undefined);
	let selectedCategories = $state<string[]>([]);
	let firstOrderOnly = $state(false);

	// Validity
	let validFrom = $state('');
	let validTo = $state('');
	let usageLimit = $state<number | undefined>(undefined);
	let perCustomerLimit = $state<number | undefined>(undefined);

	// Priority & combination
	let priority = $state(100);
	let isCombinable = $state(true);
	let exclusiveGroup = $state('');
	let applyOn = $state<ApplyOn>('line');

	// Status
	let isActive = $state(true);

	// UI state
	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	let isInitialized = $state(false);

	// Mock categories
	const mockCategories = [
		{ id: 'cat-electronics', name: 'Electronics' },
		{ id: 'cat-accessories', name: 'Accessories' },
		{ id: 'cat-office', name: 'Office Supplies' }
	];

	const ruleTypes: { value: RuleType; label: string; icon: typeof Percent; description: string }[] =
		[
			{
				value: 'discount_percentage',
				label: '% Off',
				icon: Percent,
				description: 'Percentage discount'
			},
			{
				value: 'discount_amount',
				label: '$ Off',
				icon: DollarSign,
				description: 'Fixed amount off'
			},
			{ value: 'fixed_price', label: 'Fixed', icon: Tag, description: 'Override to fixed price' },
			{
				value: 'free_item',
				label: 'Free Item',
				icon: Gift,
				description: 'Free item with purchase'
			},
			{
				value: 'buy_x_get_y',
				label: 'Buy X Get Y',
				icon: ShoppingCart,
				description: 'Buy X get Y free'
			},
			{
				value: 'bundle_price',
				label: 'Bundle',
				icon: Package,
				description: 'Special bundle pricing'
			}
		];

	// Initialize form from rule data
	$effect(() => {
		if (rule && !isInitialized) {
			name = rule.name;
			code = rule.code || '';
			description = rule.description || '';
			ruleType = rule.ruleType;

			// Discount values
			discountPercentage = rule.discountPercentage ?? 10;
			discountAmount = rule.discountAmount ?? 0;
			fixedPrice = rule.fixedPrice ?? 0;
			maxDiscountAmount = rule.maxDiscountAmount;

			// Buy X Get Y
			buyQuantity = rule.buyQuantity ?? 2;
			getQuantity = rule.getQuantity ?? 1;
			freeQuantity = rule.freeQuantity ?? 1;

			// Conditions
			minQuantity = rule.conditions?.minQuantity;
			minOrderAmount = rule.conditions?.minOrderAmount;
			selectedCategories = rule.conditions?.categories || [];
			firstOrderOnly = rule.conditions?.firstOrderOnly || false;

			// Validity
			validFrom = rule.validFrom ? formatDateForInput(rule.validFrom) : '';
			validTo = rule.validTo ? formatDateForInput(rule.validTo) : '';
			usageLimit = rule.usageLimit;
			perCustomerLimit = rule.perCustomerLimit;

			// Priority & combination
			priority = rule.priority;
			isCombinable = rule.isCombinable;
			exclusiveGroup = rule.exclusiveGroup || '';
			applyOn = rule.applyOn;

			// Status
			isActive = rule.isActive;

			isInitialized = true;
		}
	});

	function formatDateForInput(date: Date): string {
		return date.toISOString().split('T')[0];
	}

	function validate(): boolean {
		const newErrors: Record<string, string> = {};

		if (!name.trim()) {
			newErrors.name = 'Name is required';
		}

		if (ruleType === 'discount_percentage' && (!discountPercentage || discountPercentage <= 0)) {
			newErrors.discountPercentage = 'Discount percentage is required';
		}

		if (ruleType === 'discount_amount' && (!discountAmount || discountAmount <= 0)) {
			newErrors.discountAmount = 'Discount amount is required';
		}

		if (ruleType === 'fixed_price' && (!fixedPrice || fixedPrice <= 0)) {
			newErrors.fixedPrice = 'Fixed price is required';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	async function handleSubmit() {
		if (!validate()) return;

		isSubmitting = true;

		try {
			const data: UpdatePricingRuleInput = {
				name: name.trim(),
				code: code.trim() || undefined,
				description: description.trim() || undefined,
				ruleType,
				conditions: {
					minQuantity: minQuantity || undefined,
					minOrderAmount: minOrderAmount || undefined,
					categories: selectedCategories.length > 0 ? selectedCategories : undefined,
					firstOrderOnly: firstOrderOnly || undefined
				},
				discountPercentage: ruleType === 'discount_percentage' ? discountPercentage : undefined,
				discountAmount: ruleType === 'discount_amount' ? discountAmount : undefined,
				fixedPrice: ruleType === 'fixed_price' ? fixedPrice : undefined,
				maxDiscountAmount: maxDiscountAmount || undefined,
				buyQuantity: ruleType === 'buy_x_get_y' ? buyQuantity : undefined,
				getQuantity: ruleType === 'buy_x_get_y' ? getQuantity : undefined,
				freeQuantity:
					ruleType === 'free_item' || ruleType === 'buy_x_get_y' ? freeQuantity : undefined,
				validFrom: validFrom ? new Date(validFrom) : undefined,
				validTo: validTo ? new Date(validTo) : undefined,
				usageLimit: usageLimit || undefined,
				perCustomerLimit: perCustomerLimit || undefined,
				priority,
				isCombinable,
				exclusiveGroup: exclusiveGroup.trim() || undefined,
				applyOn,
				isActive
			};

			// Mock API call
			console.log('Updating pricing rule:', ruleId, data);
			await new Promise((resolve) => setTimeout(resolve, 500));

			goto(`/inventory/pricing/rules/${ruleId}`);
		} catch (error) {
			console.error('Error updating rule:', error);
		} finally {
			isSubmitting = false;
		}
	}

	function handleApplyOnChange(value: string) {
		applyOn = value as ApplyOn;
	}

	function toggleCategory(categoryId: string) {
		if (selectedCategories.includes(categoryId)) {
			selectedCategories = selectedCategories.filter((id) => id !== categoryId);
		} else {
			selectedCategories = [...selectedCategories, categoryId];
		}
	}
</script>

{#if !rule}
	<div class="container mx-auto flex items-center justify-center p-12">
		<div class="text-center">
			<h2 class="mb-2 text-xl font-semibold">Rule Not Found</h2>
			<p class="mb-4 text-muted-foreground">The pricing rule you're looking for doesn't exist.</p>
			<Button onclick={() => goto('/inventory/pricing/rules')}>
				<ArrowLeft class="mr-2 h-4 w-4" />
				Back to Rules
			</Button>
		</div>
	</div>
{:else}
	<div class="container mx-auto max-w-3xl space-y-6 p-6">
		<!-- Header -->
		<div class="flex items-center gap-4">
			<Button
				variant="ghost"
				size="icon"
				onclick={() => goto(`/inventory/pricing/rules/${ruleId}`)}
			>
				<ArrowLeft class="h-4 w-4" />
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Edit Pricing Rule</h1>
				<p class="text-muted-foreground">Update "{rule.name}"</p>
			</div>
		</div>

		<form
			onsubmit={(e) => {
				e.preventDefault();
				handleSubmit();
			}}
			class="space-y-6"
		>
			<!-- Basic Information -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Basic Information</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="name">Name *</Label>
							<Input
								id="name"
								bind:value={name}
								placeholder="e.g., Lunar New Year 2026"
								class={errors.name ? 'border-destructive' : ''}
							/>
							{#if errors.name}
								<p class="text-sm text-destructive">{errors.name}</p>
							{/if}
						</div>
						<div class="space-y-2">
							<Label for="code">Code</Label>
							<Input id="code" bind:value={code} placeholder="AUTO_GENERATED" />
						</div>
					</div>
					<div class="space-y-2">
						<Label for="description">Description</Label>
						<Textarea
							id="description"
							bind:value={description}
							placeholder="Optional description for this rule"
							rows={2}
						/>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Rule Type -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Rule Type</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
						{#each ruleTypes as rt (rt.value)}
							{@const Icon = rt.icon}
							<button
								type="button"
								class="flex flex-col items-center gap-2 rounded-lg border p-4 text-center transition-colors hover:bg-muted/50 {ruleType ===
								rt.value
									? 'border-primary bg-primary/5'
									: ''}"
								onclick={() => (ruleType = rt.value)}
							>
								<Icon
									class="h-6 w-6 {ruleType === rt.value ? 'text-primary' : 'text-muted-foreground'}"
								/>
								<span class="text-sm font-medium">{rt.label}</span>
								<span class="text-xs text-muted-foreground">{rt.description}</span>
							</button>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Discount Value -->
			<Card.Root>
				<Card.Header>
					<Card.Title>
						{#if ruleType === 'discount_percentage'}
							Percentage Discount
						{:else if ruleType === 'discount_amount'}
							Amount Discount
						{:else if ruleType === 'fixed_price'}
							Fixed Price
						{:else if ruleType === 'buy_x_get_y'}
							Buy X Get Y Configuration
						{:else if ruleType === 'free_item'}
							Free Item Configuration
						{:else}
							Bundle Configuration
						{/if}
					</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					{#if ruleType === 'discount_percentage'}
						<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
							<div class="space-y-2">
								<Label for="discountPercentage">Discount Percentage *</Label>
								<div class="flex items-center gap-2">
									<Input
										id="discountPercentage"
										type="number"
										bind:value={discountPercentage}
										min={0}
										max={100}
										class={errors.discountPercentage ? 'border-destructive' : ''}
									/>
									<span>%</span>
								</div>
								{#if errors.discountPercentage}
									<p class="text-sm text-destructive">{errors.discountPercentage}</p>
								{/if}
							</div>
							<div class="space-y-2">
								<Label for="maxDiscountAmount">Maximum Discount (optional)</Label>
								<Input
									id="maxDiscountAmount"
									type="number"
									bind:value={maxDiscountAmount}
									placeholder="No cap"
								/>
								<p class="text-xs text-muted-foreground">Leave empty for no cap</p>
							</div>
						</div>
					{:else if ruleType === 'discount_amount'}
						<div class="space-y-2">
							<Label for="discountAmount">Discount Amount *</Label>
							<div class="flex items-center gap-2">
								<Input
									id="discountAmount"
									type="number"
									bind:value={discountAmount}
									min={0}
									class={errors.discountAmount ? 'border-destructive' : ''}
								/>
								<span>VND</span>
							</div>
							{#if errors.discountAmount}
								<p class="text-sm text-destructive">{errors.discountAmount}</p>
							{/if}
						</div>
					{:else if ruleType === 'fixed_price'}
						<div class="space-y-2">
							<Label for="fixedPrice">Fixed Price *</Label>
							<div class="flex items-center gap-2">
								<Input
									id="fixedPrice"
									type="number"
									bind:value={fixedPrice}
									min={0}
									class={errors.fixedPrice ? 'border-destructive' : ''}
								/>
								<span>VND</span>
							</div>
							{#if errors.fixedPrice}
								<p class="text-sm text-destructive">{errors.fixedPrice}</p>
							{/if}
						</div>
					{:else if ruleType === 'buy_x_get_y'}
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="buyQuantity">Buy Quantity</Label>
								<Input id="buyQuantity" type="number" bind:value={buyQuantity} min={1} />
							</div>
							<div class="space-y-2">
								<Label for="getQuantity">Get Quantity (Free)</Label>
								<Input id="getQuantity" type="number" bind:value={getQuantity} min={1} />
							</div>
						</div>
						<p class="text-sm text-muted-foreground">
							Buy {buyQuantity} items, get {getQuantity} free (lowest priced)
						</p>
					{:else if ruleType === 'free_item'}
						<div class="space-y-2">
							<Label for="freeQuantity">Free Quantity</Label>
							<Input id="freeQuantity" type="number" bind:value={freeQuantity} min={1} />
						</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<!-- Conditions -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Conditions</Card.Title>
					<p class="text-sm text-muted-foreground">When should this rule apply?</p>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="minQuantity">Minimum Quantity</Label>
							<Input
								id="minQuantity"
								type="number"
								bind:value={minQuantity}
								min={1}
								placeholder="Any"
							/>
						</div>
						<div class="space-y-2">
							<Label for="minOrderAmount">Minimum Order Amount</Label>
							<div class="flex items-center gap-2">
								<Input
									id="minOrderAmount"
									type="number"
									bind:value={minOrderAmount}
									min={0}
									placeholder="Any"
								/>
								<span class="text-sm">VND</span>
							</div>
						</div>
					</div>

					<div class="space-y-2">
						<Label>Categories (optional)</Label>
						<div class="flex flex-wrap gap-2">
							{#each mockCategories as category (category.id)}
								<button
									type="button"
									class="rounded-full border px-3 py-1 text-sm transition-colors {selectedCategories.includes(
										category.id
									)
										? 'border-primary bg-primary text-primary-foreground'
										: 'hover:bg-muted'}"
									onclick={() => toggleCategory(category.id)}
								>
									{category.name}
								</button>
							{/each}
						</div>
					</div>

					<div class="flex items-center gap-2">
						<Checkbox id="firstOrderOnly" bind:checked={firstOrderOnly} />
						<Label for="firstOrderOnly" class="font-normal">First order only</Label>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Validity & Limits -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Validity & Limits</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="validFrom">Valid From</Label>
							<Input id="validFrom" type="date" bind:value={validFrom} />
						</div>
						<div class="space-y-2">
							<Label for="validTo">Valid To</Label>
							<Input id="validTo" type="date" bind:value={validTo} />
						</div>
					</div>

					<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="usageLimit">Total Usage Limit</Label>
							<Input
								id="usageLimit"
								type="number"
								bind:value={usageLimit}
								min={1}
								placeholder="Unlimited"
							/>
							{#if rule.usageCount}
								<p class="text-xs text-muted-foreground">
									Currently used: {rule.usageCount} times
								</p>
							{/if}
						</div>
						<div class="space-y-2">
							<Label for="perCustomerLimit">Per Customer Limit</Label>
							<Input
								id="perCustomerLimit"
								type="number"
								bind:value={perCustomerLimit}
								min={1}
								placeholder="Unlimited"
							/>
						</div>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Priority & Combination -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Priority & Combination</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
						<div class="space-y-2">
							<Label for="priority">Priority</Label>
							<Input id="priority" type="number" bind:value={priority} min={0} />
							<p class="text-xs text-muted-foreground">Lower = applied first</p>
						</div>
						<div class="space-y-2">
							<Label for="exclusiveGroup">Exclusive Group (optional)</Label>
							<Input id="exclusiveGroup" bind:value={exclusiveGroup} placeholder="e.g., SEASONAL" />
							<p class="text-xs text-muted-foreground">Only one rule per group can apply</p>
						</div>
					</div>

					<div class="space-y-3">
						<Label>Combination</Label>
						<RadioGroup.Root
							value={isCombinable ? 'combinable' : 'exclusive'}
							onValueChange={(v) => (isCombinable = v === 'combinable')}
						>
							<div class="flex items-center gap-2">
								<RadioGroup.Item value="combinable" id="combinable" />
								<Label for="combinable" class="font-normal"
									>Combinable - Can stack with other discounts</Label
								>
							</div>
							<div class="flex items-center gap-2">
								<RadioGroup.Item value="exclusive" id="exclusive" />
								<Label for="exclusive" class="font-normal"
									>Exclusive - Cannot combine with other discounts</Label
								>
							</div>
						</RadioGroup.Root>
					</div>

					<div class="space-y-3">
						<Label>Apply On</Label>
						<RadioGroup.Root value={applyOn} onValueChange={handleApplyOnChange}>
							<div class="flex items-center gap-2">
								<RadioGroup.Item value="line" id="apply-line" />
								<Label for="apply-line" class="font-normal">Per line item</Label>
							</div>
							<div class="flex items-center gap-2">
								<RadioGroup.Item value="order" id="apply-order" />
								<Label for="apply-order" class="font-normal">Whole order</Label>
							</div>
						</RadioGroup.Root>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Status -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Status</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="flex items-center gap-2">
						<Checkbox id="isActive" bind:checked={isActive} />
						<Label for="isActive" class="font-normal">Active</Label>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Actions -->
			<div class="flex justify-end gap-3">
				<Button
					type="button"
					variant="outline"
					onclick={() => goto(`/inventory/pricing/rules/${ruleId}`)}
					disabled={isSubmitting}
				>
					Cancel
				</Button>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Saving...
					{:else}
						Save Changes
					{/if}
				</Button>
			</div>
		</form>
	</div>
{/if}
