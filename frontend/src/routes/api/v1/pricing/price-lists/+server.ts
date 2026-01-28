/**
 * Pricing API - Price Lists CRUD (Mock)
 *
 * This mock endpoint provides CRUD operations for price lists.
 * In production, these would be proxied to the backend inventory service.
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { mockPriceLists, type PriceList, type CreatePriceListInput } from '$lib/types/pricing';

// In-memory store for mock data
let priceLists = [...mockPriceLists];

export const GET: RequestHandler = async ({ url }) => {
	// Parse query parameters
	const search = url.searchParams.get('search')?.toLowerCase();
	const priceListType = url.searchParams.get('type');
	const isActive = url.searchParams.get('isActive');
	const page = parseInt(url.searchParams.get('page') || '1');
	const limit = parseInt(url.searchParams.get('limit') || '10');

	// Filter price lists
	let filtered = priceLists;

	if (search) {
		filtered = filtered.filter(
			(pl) =>
				pl.name.toLowerCase().includes(search) ||
				pl.code.toLowerCase().includes(search) ||
				pl.description?.toLowerCase().includes(search)
		);
	}

	if (priceListType && priceListType !== 'all') {
		filtered = filtered.filter((pl) => pl.priceListType === priceListType);
	}

	if (isActive !== null && isActive !== undefined && isActive !== 'all') {
		filtered = filtered.filter((pl) => pl.isActive === (isActive === 'true'));
	}

	// Paginate
	const total = filtered.length;
	const offset = (page - 1) * limit;
	const paginated = filtered.slice(offset, offset + limit);

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

export const POST: RequestHandler = async ({ request }) => {
	try {
		const body: CreatePriceListInput = await request.json();

		// Validate required fields
		if (!body.name || !body.code) {
			return json({ error: 'Name and code are required' }, { status: 400 });
		}

		// Check for duplicate code
		if (priceLists.some((pl) => pl.code === body.code)) {
			return json({ error: 'Price list with this code already exists' }, { status: 409 });
		}

		// Create new price list
		const newPriceList: PriceList = {
			priceListId: `pl-${Date.now()}`,
			tenantId: 'tenant-001',
			name: body.name,
			code: body.code,
			description: body.description,
			currencyCode: body.currencyCode || 'VND',
			priceListType: body.priceListType || 'sale',
			basedOn: body.basedOn || 'base_price',
			parentPriceListId: body.parentPriceListId,
			defaultPercentage: body.defaultPercentage || 0,
			validFrom: body.validFrom ? new Date(body.validFrom) : undefined,
			validTo: body.validTo ? new Date(body.validTo) : undefined,
			priority: body.priority || 100,
			isDefault: body.isDefault || false,
			isActive: body.isActive ?? true,
			createdAt: new Date(),
			updatedAt: new Date(),
			itemCount: 0,
			customerCount: 0
		};

		// If setting as default, unset other defaults
		if (newPriceList.isDefault && newPriceList.priceListType === 'sale') {
			priceLists = priceLists.map((pl) => ({
				...pl,
				isDefault: pl.priceListType === 'sale' ? false : pl.isDefault
			}));
		}

		priceLists.push(newPriceList);

		return json(newPriceList, { status: 201 });
	} catch (error) {
		console.error('Error creating price list:', error);
		return json({ error: 'Invalid request body' }, { status: 400 });
	}
};
