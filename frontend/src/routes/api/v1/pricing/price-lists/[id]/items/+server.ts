/**
 * Pricing API - Price List Items (Mock)
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import type { PriceListItem, CreatePriceListItemInput } from '$lib/types/pricing';

// Mock price list items
let priceListItems: PriceListItem[] = [
	{
		itemId: 'item-001',
		tenantId: 'tenant-001',
		priceListId: 'pl-002',
		applyTo: 'product',
		productId: 'prod-001',
		minQuantity: 1,
		computeMethod: 'percentage',
		percentage: -15,
		roundingMethod: 'none',
		roundingPrecision: 0,
		createdAt: new Date(),
		updatedAt: new Date(),
		product: { name: 'Laptop Pro 15"', sku: 'LP-15', salePrice: 25000000 }
	},
	{
		itemId: 'item-002',
		tenantId: 'tenant-001',
		priceListId: 'pl-002',
		applyTo: 'product',
		productId: 'prod-001',
		minQuantity: 10,
		computeMethod: 'percentage',
		percentage: -20,
		roundingMethod: 'none',
		roundingPrecision: 0,
		createdAt: new Date(),
		updatedAt: new Date(),
		product: { name: 'Laptop Pro 15"', sku: 'LP-15', salePrice: 25000000 }
	},
	{
		itemId: 'item-003',
		tenantId: 'tenant-001',
		priceListId: 'pl-002',
		applyTo: 'product',
		productId: 'prod-002',
		minQuantity: 1,
		computeMethod: 'fixed',
		fixedPrice: 400000,
		roundingMethod: 'none',
		roundingPrecision: 0,
		createdAt: new Date(),
		updatedAt: new Date(),
		product: { name: 'Wireless Mouse', sku: 'WM-01', salePrice: 500000 }
	},
	{
		itemId: 'item-004',
		tenantId: 'tenant-001',
		priceListId: 'pl-002',
		applyTo: 'category',
		categoryId: 'cat-001',
		minQuantity: 1,
		computeMethod: 'percentage',
		percentage: -10,
		roundingMethod: 'none',
		roundingPrecision: 0,
		createdAt: new Date(),
		updatedAt: new Date(),
		category: { name: 'Electronics', code: 'ELEC' }
	}
];

export const GET: RequestHandler = async ({ params, url }) => {
	const { id } = params;

	// Filter items for this price list
	const items = priceListItems.filter((item) => item.priceListId === id);

	// Parse query parameters
	const page = parseInt(url.searchParams.get('page') || '1');
	const limit = parseInt(url.searchParams.get('limit') || '20');

	// Paginate
	const total = items.length;
	const offset = (page - 1) * limit;
	const paginated = items.slice(offset, offset + limit);

	return json({
		data: paginated,
		meta: {
			total,
			page,
			limit,
			totalPages: Math.ceil(total / limit)
		}
	});
};

export const POST: RequestHandler = async ({ params, request }) => {
	const { id } = params;

	if (!id) {
		return json({ error: 'Price list ID is required' }, { status: 400 });
	}

	try {
		const body = await request.json();

		// Handle both single item and array of items
		const items: CreatePriceListItemInput[] = Array.isArray(body) ? body : [body];

		const createdItems: PriceListItem[] = items.map((input) => ({
			itemId: `item-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
			tenantId: 'tenant-001',
			priceListId: id,
			applyTo: input.applyTo,
			productId: input.productId,
			variantId: input.variantId,
			categoryId: input.categoryId,
			minQuantity: input.minQuantity || 1,
			maxQuantity: input.maxQuantity,
			computeMethod: input.computeMethod,
			fixedPrice: input.fixedPrice,
			percentage: input.percentage,
			marginPercentage: input.marginPercentage,
			formula: input.formula,
			roundingMethod: input.roundingMethod || 'none',
			roundingPrecision: input.roundingPrecision || 0,
			validFrom: input.validFrom ? new Date(input.validFrom) : undefined,
			validTo: input.validTo ? new Date(input.validTo) : undefined,
			createdAt: new Date(),
			updatedAt: new Date()
		}));

		priceListItems.push(...createdItems);

		return json(Array.isArray(body) ? createdItems : createdItems[0], { status: 201 });
	} catch (error) {
		console.error('Error creating price list item:', error);
		return json({ error: 'Invalid request body' }, { status: 400 });
	}
};

export const DELETE: RequestHandler = async ({ params, url }) => {
	const { id } = params;
	const itemId = url.searchParams.get('itemId');

	if (itemId) {
		// Delete specific item
		const index = priceListItems.findIndex(
			(item) => item.priceListId === id && item.itemId === itemId
		);

		if (index === -1) {
			return json({ error: 'Item not found' }, { status: 404 });
		}

		priceListItems.splice(index, 1);
	} else {
		// Delete all items for this price list
		priceListItems = priceListItems.filter((item) => item.priceListId !== id);
	}

	return new Response(null, { status: 204 });
};
