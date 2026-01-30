import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { PricingRule, UpdatePricingRuleInput } from '$lib/types/pricing';

// Mock data for development - shared state
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

// Helper function to serialize dates
function serializeRule(rule: PricingRule) {
	return {
		...rule,
		validFrom: rule.validFrom?.toISOString(),
		validTo: rule.validTo?.toISOString(),
		createdAt: rule.createdAt.toISOString(),
		updatedAt: rule.updatedAt.toISOString()
	};
}

// GET /api/v1/pricing/rules/[id] - Get a specific pricing rule
export const GET: RequestHandler = async ({ params }) => {
	const { id } = params;

	const rule = mockPricingRules.find((r) => r.ruleId === id);

	if (!rule) {
		return json({ error: 'Pricing rule not found' }, { status: 404 });
	}

	return json(serializeRule(rule));
};

// PUT /api/v1/pricing/rules/[id] - Full update of a pricing rule
export const PUT: RequestHandler = async ({ params, request }) => {
	const { id } = params;
	const data: UpdatePricingRuleInput = await request.json();

	const index = mockPricingRules.findIndex((r) => r.ruleId === id);

	if (index === -1) {
		return json({ error: 'Pricing rule not found' }, { status: 404 });
	}

	const existingRule = mockPricingRules[index];

	// Update the rule
	const updatedRule: PricingRule = {
		...existingRule,
		name: data.name ?? existingRule.name,
		code: data.code ?? existingRule.code,
		description: data.description,
		ruleType: data.ruleType ?? existingRule.ruleType,
		conditions: data.conditions ?? existingRule.conditions,
		discountPercentage: data.discountPercentage,
		discountAmount: data.discountAmount,
		fixedPrice: data.fixedPrice,
		maxDiscountAmount: data.maxDiscountAmount,
		buyQuantity: data.buyQuantity,
		getQuantity: data.getQuantity,
		freeQuantity: data.freeQuantity,
		validFrom: data.validFrom ? new Date(data.validFrom) : existingRule.validFrom,
		validTo: data.validTo ? new Date(data.validTo) : existingRule.validTo,
		usageLimit: data.usageLimit,
		perCustomerLimit: data.perCustomerLimit,
		priority: data.priority ?? existingRule.priority,
		isCombinable: data.isCombinable ?? existingRule.isCombinable,
		exclusiveGroup: data.exclusiveGroup,
		applyOn: data.applyOn ?? existingRule.applyOn,
		isActive: data.isActive ?? existingRule.isActive,
		updatedAt: new Date()
	};

	mockPricingRules[index] = updatedRule;

	return json(serializeRule(updatedRule));
};

// PATCH /api/v1/pricing/rules/[id] - Partial update (e.g., toggle active)
export const PATCH: RequestHandler = async ({ params, request }) => {
	const { id } = params;
	const data: Partial<UpdatePricingRuleInput> = await request.json();

	const index = mockPricingRules.findIndex((r) => r.ruleId === id);

	if (index === -1) {
		return json({ error: 'Pricing rule not found' }, { status: 404 });
	}

	const existingRule = mockPricingRules[index];

	// Partial update
	const updatedRule: PricingRule = {
		...existingRule,
		...data,
		validFrom: data.validFrom ? new Date(data.validFrom) : existingRule.validFrom,
		validTo: data.validTo ? new Date(data.validTo) : existingRule.validTo,
		updatedAt: new Date()
	};

	mockPricingRules[index] = updatedRule;

	return json(serializeRule(updatedRule));
};

// DELETE /api/v1/pricing/rules/[id] - Delete a pricing rule
export const DELETE: RequestHandler = async ({ params }) => {
	const { id } = params;

	const index = mockPricingRules.findIndex((r) => r.ruleId === id);

	if (index === -1) {
		return json({ error: 'Pricing rule not found' }, { status: 404 });
	}

	// Check if rule has been used
	const rule = mockPricingRules[index];
	if (rule.usageCount && rule.usageCount > 0) {
		return json(
			{ error: 'Cannot delete a rule that has been used. Consider deactivating it instead.' },
			{ status: 400 }
		);
	}

	mockPricingRules.splice(index, 1);

	return json({ success: true, message: 'Pricing rule deleted successfully' });
};
