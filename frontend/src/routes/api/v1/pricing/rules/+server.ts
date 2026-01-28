import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { PricingRule, CreatePricingRuleInput, RuleType, ApplyOn } from '$lib/types/pricing';

// Mock data for development
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
	},
	{
		ruleId: 'rule-004',
		tenantId: 'tenant-001',
		name: 'First Order Discount',
		code: 'FIRST10',
		description: 'Welcome discount for new customers',
		ruleType: 'discount_percentage',
		conditions: {
			firstOrderOnly: true
		},
		discountPercentage: 10,
		maxDiscountAmount: 100000,
		priority: 50,
		isCombinable: true,
		applyOn: 'order',
		isActive: true,
		createdAt: new Date('2025-01-01'),
		updatedAt: new Date('2025-01-01')
	},
	{
		ruleId: 'rule-005',
		tenantId: 'tenant-001',
		name: 'Summer Sale 2026',
		code: 'SUMMER26',
		ruleType: 'discount_percentage',
		conditions: {},
		discountPercentage: 20,
		validFrom: new Date('2026-06-01'),
		validTo: new Date('2026-08-31'),
		priority: 30,
		isCombinable: false,
		exclusiveGroup: 'SEASONAL',
		applyOn: 'order',
		isActive: true,
		createdAt: new Date('2026-01-10'),
		updatedAt: new Date('2026-01-10')
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

// GET /api/v1/pricing/rules - List pricing rules with filtering
export const GET: RequestHandler = async ({ url }) => {
	const search = url.searchParams.get('search') || '';
	const ruleType = url.searchParams.get('ruleType') as RuleType | null;
	const status = url.searchParams.get('status'); // active, scheduled, expired, inactive
	const isActive = url.searchParams.get('isActive');
	const page = parseInt(url.searchParams.get('page') || '1');
	const limit = parseInt(url.searchParams.get('limit') || '20');

	let filtered = [...mockPricingRules];

	// Search filter
	if (search) {
		const searchLower = search.toLowerCase();
		filtered = filtered.filter(
			(r) =>
				r.name.toLowerCase().includes(searchLower) ||
				r.code?.toLowerCase().includes(searchLower) ||
				r.description?.toLowerCase().includes(searchLower)
		);
	}

	// Rule type filter
	if (ruleType) {
		filtered = filtered.filter((r) => r.ruleType === ruleType);
	}

	// Active filter
	if (isActive !== null) {
		filtered = filtered.filter((r) => r.isActive === (isActive === 'true'));
	}

	// Status filter (active, scheduled, expired)
	if (status) {
		const now = new Date();
		filtered = filtered.filter((r) => {
			if (!r.isActive && status !== 'inactive') return false;
			if (status === 'inactive') return !r.isActive;

			const hasStarted = !r.validFrom || r.validFrom <= now;
			const hasEnded = r.validTo && r.validTo < now;

			if (status === 'active') return r.isActive && hasStarted && !hasEnded;
			if (status === 'scheduled') return r.isActive && !hasStarted;
			if (status === 'expired') return hasEnded;
			return true;
		});
	}

	// Pagination
	const total = filtered.length;
	const offset = (page - 1) * limit;
	const paginatedRules = filtered.slice(offset, offset + limit);

	return json({
		data: paginatedRules.map(serializeRule),
		pagination: {
			page,
			limit,
			total,
			totalPages: Math.ceil(total / limit)
		}
	});
};

// POST /api/v1/pricing/rules - Create a new pricing rule
export const POST: RequestHandler = async ({ request }) => {
	const data: CreatePricingRuleInput = await request.json();

	// Validate required fields
	if (!data.name) {
		return json({ error: 'Name is required' }, { status: 400 });
	}

	if (!data.ruleType) {
		return json({ error: 'Rule type is required' }, { status: 400 });
	}

	// Create new rule
	const newRule: PricingRule = {
		ruleId: `rule-${Date.now()}`,
		tenantId: 'tenant-001',
		name: data.name,
		code: data.code || generateCode(data.name),
		description: data.description,
		ruleType: data.ruleType,
		conditions: data.conditions || {},
		discountPercentage: data.discountPercentage,
		discountAmount: data.discountAmount,
		fixedPrice: data.fixedPrice,
		maxDiscountAmount: data.maxDiscountAmount,
		buyQuantity: data.buyQuantity,
		getQuantity: data.getQuantity,
		freeQuantity: data.freeQuantity,
		validFrom: data.validFrom ? new Date(data.validFrom) : undefined,
		validTo: data.validTo ? new Date(data.validTo) : undefined,
		usageLimit: data.usageLimit,
		usageCount: 0,
		perCustomerLimit: data.perCustomerLimit,
		priority: data.priority ?? 100,
		isCombinable: data.isCombinable ?? true,
		exclusiveGroup: data.exclusiveGroup,
		applyOn: data.applyOn ?? 'line',
		isActive: data.isActive ?? true,
		createdAt: new Date(),
		updatedAt: new Date()
	};

	// In a real implementation, save to database
	mockPricingRules.push(newRule);

	return json(serializeRule(newRule), { status: 201 });
};

// Helper function to generate code from name
function generateCode(name: string): string {
	return name
		.toUpperCase()
		.replace(/[^A-Z0-9]+/g, '_')
		.replace(/^_+|_+$/g, '')
		.slice(0, 20);
}
