<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { toast } from 'svelte-sonner';
	import type { PricingRule, RuleType } from '$lib/types/pricing';
	import {
		Play,
		ShoppingCart,
		Package,
		DollarSign,
		CheckCircle,
		XCircle,
		AlertTriangle,
		Calculator,
		Loader2
	} from 'lucide-svelte';

	interface Props {
		rule: PricingRule;
		open?: boolean;
		onOpenChange?: (open: boolean) => void;
	}

	let { rule, open = $bindable(false), onOpenChange }: Props = $props();

	// Test order state
	let quantity = $state(1);
	let unitPrice = $state(100000);
	let orderTotal = $state(500000);
	let categoryId = $state('');
	let isFirstOrder = $state(false);
	let orderDate = $state(new Date().toISOString().split('T')[0]);
	let orderTime = $state('12:00');

	// Test result state
	let isLoading = $state(false);
	let testResult = $state<{
		eligible: boolean;
		originalPrice: number;
		discountAmount: number;
		finalPrice: number;
		appliedDiscount: string;
		failureReasons: string[];
	} | null>(null);

	// Mock categories for testing
	const mockCategories = [
		{ value: 'cat-1', label: 'Electronics' },
		{ value: 'cat-2', label: 'Clothing' },
		{ value: 'cat-3', label: 'Food & Beverages' },
		{ value: 'cat-4', label: 'Home & Garden' }
	];

	const lineTotal = $derived(quantity * unitPrice);

	// Helper to get the discount value based on rule type
	function getDiscountValue(r: PricingRule): number {
		switch (r.ruleType) {
			case 'discount_percentage':
				return r.discountPercentage ?? 0;
			case 'discount_amount':
				return r.discountAmount ?? 0;
			case 'fixed_price':
			case 'bundle_price':
				return r.fixedPrice ?? 0;
			case 'free_item':
			case 'buy_x_get_y':
				return r.freeQuantity ?? 0;
			default:
				return 0;
		}
	}

	function getDiscountDescription(ruleType: RuleType, value: number): string {
		switch (ruleType) {
			case 'discount_percentage':
				return `${value}% off`;
			case 'discount_amount':
				return `₫${value.toLocaleString()} off`;
			case 'fixed_price':
				return `Fixed at ₫${value.toLocaleString()}`;
			case 'free_item':
				return `Free item included`;
			case 'buy_x_get_y':
				return `Buy X Get Y deal`;
			case 'bundle_price':
				return `Bundle price: ₫${value.toLocaleString()}`;
			default:
				return 'Discount applied';
		}
	}

	function calculateDiscount(
		ruleType: RuleType,
		value: number,
		originalPrice: number
	): { discountAmount: number; finalPrice: number } {
		switch (ruleType) {
			case 'discount_percentage':
				const percentDiscount = originalPrice * (value / 100);
				return {
					discountAmount: percentDiscount,
					finalPrice: originalPrice - percentDiscount
				};
			case 'discount_amount':
				return {
					discountAmount: value,
					finalPrice: Math.max(0, originalPrice - value)
				};
			case 'fixed_price':
				return {
					discountAmount: Math.max(0, originalPrice - value),
					finalPrice: value
				};
			case 'free_item':
			case 'buy_x_get_y':
				// Simplified - would need more complex logic in production
				const freeItemValue = unitPrice;
				return {
					discountAmount: freeItemValue,
					finalPrice: originalPrice - freeItemValue
				};
			case 'bundle_price':
				return {
					discountAmount: Math.max(0, originalPrice - value),
					finalPrice: value
				};
			default:
				return { discountAmount: 0, finalPrice: originalPrice };
		}
	}

	function checkConditions(): string[] {
		const failures: string[] = [];
		const conditions = rule.conditions;

		if (!conditions) return failures;

		// Check quantity conditions
		if (conditions.minQuantity && quantity < conditions.minQuantity) {
			failures.push(`Minimum quantity required: ${conditions.minQuantity} (current: ${quantity})`);
		}
		if (conditions.maxQuantity && quantity > conditions.maxQuantity) {
			failures.push(`Maximum quantity allowed: ${conditions.maxQuantity} (current: ${quantity})`);
		}

		// Check order amount conditions
		if (conditions.minOrderAmount && orderTotal < conditions.minOrderAmount) {
			failures.push(
				`Minimum order amount: ₫${conditions.minOrderAmount.toLocaleString()} (current: ₫${orderTotal.toLocaleString()})`
			);
		}
		if (conditions.maxOrderAmount && orderTotal > conditions.maxOrderAmount) {
			failures.push(
				`Maximum order amount: ₫${conditions.maxOrderAmount.toLocaleString()} (current: ₫${orderTotal.toLocaleString()})`
			);
		}

		// Check category conditions
		if (conditions.categoryIds && conditions.categoryIds.length > 0 && categoryId) {
			if (!conditions.categoryIds.includes(categoryId)) {
				failures.push('Product category not eligible for this rule');
			}
		}

		// Check first order condition
		if (conditions.firstOrderOnly && !isFirstOrder) {
			failures.push('This rule applies only to first orders');
		}

		// Check date validity
		const now = new Date();
		if (rule.validFrom && new Date(rule.validFrom) > now) {
			failures.push(
				`Rule not yet active (starts: ${new Date(rule.validFrom).toLocaleDateString()})`
			);
		}
		if (rule.validTo && new Date(rule.validTo) < now) {
			failures.push(`Rule has expired (ended: ${new Date(rule.validTo).toLocaleDateString()})`);
		}

		// Check usage limit
		if (rule.usageLimit && (rule.usageCount ?? 0) >= rule.usageLimit) {
			failures.push(`Usage limit reached (${rule.usageCount}/${rule.usageLimit})`);
		}

		// Check day of week
		if (conditions.validDays && conditions.validDays.length > 0) {
			const testDate = new Date(orderDate);
			const dayOfWeek = testDate.getDay();
			if (!conditions.validDays.includes(dayOfWeek)) {
				const dayNames = [
					'Sunday',
					'Monday',
					'Tuesday',
					'Wednesday',
					'Thursday',
					'Friday',
					'Saturday'
				];
				failures.push(`Not valid on ${dayNames[dayOfWeek]}`);
			}
		}

		// Check time of day
		if (conditions.validHoursStart !== undefined && conditions.validHoursEnd !== undefined) {
			const [hours] = orderTime.split(':').map(Number);
			const startHour = Number(conditions.validHoursStart);
			const endHour = Number(conditions.validHoursEnd);
			if (hours < startHour || hours >= endHour) {
				failures.push(`Only valid between ${startHour}:00 - ${endHour}:00 (current: ${orderTime})`);
			}
		}

		return failures;
	}

	async function runTest() {
		isLoading = true;
		testResult = null;

		// Simulate API delay
		await new Promise((resolve) => setTimeout(resolve, 500));

		const failures = checkConditions();
		const eligible = failures.length === 0;
		const originalPrice = lineTotal;

		if (eligible) {
			const discountValue = getDiscountValue(rule);
			const { discountAmount, finalPrice } = calculateDiscount(
				rule.ruleType,
				discountValue,
				originalPrice
			);
			testResult = {
				eligible: true,
				originalPrice,
				discountAmount,
				finalPrice,
				appliedDiscount: getDiscountDescription(rule.ruleType, discountValue),
				failureReasons: []
			};
		} else {
			testResult = {
				eligible: false,
				originalPrice,
				discountAmount: 0,
				finalPrice: originalPrice,
				appliedDiscount: '',
				failureReasons: failures
			};
		}

		isLoading = false;
	}

	function resetTest() {
		quantity = 1;
		unitPrice = 100000;
		orderTotal = 500000;
		categoryId = '';
		isFirstOrder = false;
		orderDate = new Date().toISOString().split('T')[0];
		orderTime = '12:00';
		testResult = null;
	}

	function handleOpenChange(value: boolean) {
		open = value;
		if (!value) {
			resetTest();
		}
		onOpenChange?.(value);
	}
</script>

<Dialog.Root bind:open onOpenChange={handleOpenChange}>
	<Dialog.Content class="max-w-2xl">
		<Dialog.Header>
			<Dialog.Title class="flex items-center gap-2">
				<Play class="h-5 w-5" />
				Test Pricing Rule
			</Dialog.Title>
			<Dialog.Description>
				Simulate an order to test if the rule "{rule.name}" would apply
			</Dialog.Description>
		</Dialog.Header>

		<div class="grid gap-6 py-4">
			<!-- Test Input Section -->
			<div class="grid gap-4">
				<h4 class="flex items-center gap-2 font-medium">
					<ShoppingCart class="h-4 w-4" />
					Sample Order Details
				</h4>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="quantity">Quantity</Label>
						<Input id="quantity" type="number" min={1} bind:value={quantity} />
					</div>

					<div class="space-y-2">
						<Label for="unitPrice">Unit Price (₫)</Label>
						<Input id="unitPrice" type="number" min={0} bind:value={unitPrice} />
					</div>

					<div class="space-y-2">
						<Label for="orderTotal">Order Total (₫)</Label>
						<Input id="orderTotal" type="number" min={0} bind:value={orderTotal} />
					</div>

					<div class="space-y-2">
						<Label for="category">Product Category</Label>
						<Select.Root type="single" bind:value={categoryId}>
							<Select.Trigger id="category" class="w-full">
								{#if categoryId}
									{mockCategories.find((c) => c.value === categoryId)?.label}
								{:else}
									<span class="text-muted-foreground">Select category</span>
								{/if}
							</Select.Trigger>
							<Select.Content>
								{#each mockCategories as category (category.value)}
									<Select.Item value={category.value}>{category.label}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<div class="space-y-2">
						<Label for="orderDate">Order Date</Label>
						<Input id="orderDate" type="date" bind:value={orderDate} />
					</div>

					<div class="space-y-2">
						<Label for="orderTime">Order Time</Label>
						<Input id="orderTime" type="time" bind:value={orderTime} />
					</div>
				</div>

				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="isFirstOrder"
						bind:checked={isFirstOrder}
						class="h-4 w-4 rounded border-gray-300"
					/>
					<Label for="isFirstOrder" class="cursor-pointer">This is a first order</Label>
				</div>

				<div class="flex items-center gap-4 rounded-lg bg-muted p-3">
					<Package class="h-5 w-5 text-muted-foreground" />
					<div>
						<p class="text-sm font-medium">Line Total</p>
						<p class="text-lg font-bold">₫{lineTotal.toLocaleString()}</p>
					</div>
				</div>
			</div>

			<Separator />

			<!-- Test Results Section -->
			{#if testResult}
				<div class="grid gap-4">
					<h4 class="flex items-center gap-2 font-medium">
						<Calculator class="h-4 w-4" />
						Test Results
					</h4>

					{#if testResult.eligible}
						<div
							class="rounded-lg border border-green-200 bg-green-50 p-4 dark:border-green-800 dark:bg-green-950"
						>
							<div class="mb-3 flex items-center gap-2">
								<CheckCircle class="h-5 w-5 text-green-600" />
								<span class="font-medium text-green-700 dark:text-green-400">Rule Applies!</span>
								<Badge variant="secondary">{testResult.appliedDiscount}</Badge>
							</div>

							<div class="grid grid-cols-3 gap-4 text-sm">
								<div>
									<p class="text-muted-foreground">Original Price</p>
									<p class="font-medium">₫{testResult.originalPrice.toLocaleString()}</p>
								</div>
								<div>
									<p class="text-muted-foreground">Discount</p>
									<p class="font-medium text-green-600">
										-₫{testResult.discountAmount.toLocaleString()}
									</p>
								</div>
								<div>
									<p class="text-muted-foreground">Final Price</p>
									<p class="text-lg font-bold">₫{testResult.finalPrice.toLocaleString()}</p>
								</div>
							</div>
						</div>
					{:else}
						<div
							class="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-950"
						>
							<div class="mb-3 flex items-center gap-2">
								<XCircle class="h-5 w-5 text-red-600" />
								<span class="font-medium text-red-700 dark:text-red-400">Rule Does Not Apply</span>
							</div>

							<div class="space-y-2">
								<p class="text-sm text-muted-foreground">Conditions not met:</p>
								<ul class="space-y-1">
									{#each testResult.failureReasons as reason, i (i)}
										<li class="flex items-start gap-2 text-sm">
											<AlertTriangle class="mt-0.5 h-4 w-4 shrink-0 text-amber-500" />
											<span>{reason}</span>
										</li>
									{/each}
								</ul>
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => handleOpenChange(false)}>Close</Button>
			<Button variant="outline" onclick={resetTest}>Reset</Button>
			<Button onclick={runTest} disabled={isLoading}>
				{#if isLoading}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Testing...
				{:else}
					<Play class="mr-2 h-4 w-4" />
					Run Test
				{/if}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
