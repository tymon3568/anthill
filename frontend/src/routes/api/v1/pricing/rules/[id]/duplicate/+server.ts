import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { PricingRule } from '$lib/types/pricing';

// Mock data for development - shared state with parent route
// In production, this would be a database call
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

// POST /api/v1/pricing/rules/[id]/duplicate - Duplicate a pricing rule
export const POST: RequestHandler = async ({ params, request }) => {
	const { id } = params;

	// Find the source rule
	const sourceRule = mockPricingRules.find((r) => r.ruleId === id);

	if (!sourceRule) {
		return json({ error: 'Pricing rule not found' }, { status: 404 });
	}

	// Parse optional customizations from request body
	let customizations: Partial<PricingRule> = {};
	try {
		const body = await request.text();
		if (body) {
			customizations = JSON.parse(body);
		}
	} catch {
		// No customizations provided, that's fine
	}

	// Generate new ID and code
	const newId = `rule-${Date.now()}`;
	const newCode = customizations.code || `${sourceRule.code}_COPY`;
	const newName = customizations.name || `${sourceRule.name} (Copy)`;

	// Create duplicate rule
	const duplicatedRule: PricingRule = {
		...sourceRule,
		ruleId: newId,
		name: newName,
		code: newCode,
		description: customizations.description ?? sourceRule.description,
		// Reset usage stats
		usageCount: 0,
		// Set as inactive by default for safety
		isActive: false,
		// Update timestamps
		createdAt: new Date(),
		updatedAt: new Date(),
		// Keep validity dates if still valid, otherwise clear them
		validFrom:
			sourceRule.validTo && new Date(sourceRule.validTo) > new Date()
				? sourceRule.validFrom
				: undefined,
		validTo:
			sourceRule.validTo && new Date(sourceRule.validTo) > new Date()
				? sourceRule.validTo
				: undefined
	};

	// Add to mock database
	mockPricingRules.push(duplicatedRule);

	return json(serializeRule(duplicatedRule), { status: 201 });
};
