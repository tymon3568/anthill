/**
 * Pricing API - Single Price List Operations (Mock)
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { mockPriceLists, type PriceList, type UpdatePriceListInput } from '$lib/types/pricing';

// Reference the same in-memory store
let priceLists = [...mockPriceLists];

export const GET: RequestHandler = async ({ params }) => {
	const { id } = params;

	const priceList = priceLists.find((pl) => pl.priceListId === id);

	if (!priceList) {
		return json({ error: 'Price list not found' }, { status: 404 });
	}

	return json(priceList);
};

export const PUT: RequestHandler = async ({ params, request }) => {
	const { id } = params;

	const index = priceLists.findIndex((pl) => pl.priceListId === id);

	if (index === -1) {
		return json({ error: 'Price list not found' }, { status: 404 });
	}

	try {
		const body: UpdatePriceListInput = await request.json();
		const existing = priceLists[index];

		// Check for duplicate code if changing
		if (body.code && body.code !== existing.code) {
			if (priceLists.some((pl) => pl.code === body.code && pl.priceListId !== id)) {
				return json({ error: 'Price list with this code already exists' }, { status: 409 });
			}
		}

		// Update price list
		const updated: PriceList = {
			...existing,
			name: body.name ?? existing.name,
			code: body.code ?? existing.code,
			description: body.description ?? existing.description,
			currencyCode: body.currencyCode ?? existing.currencyCode,
			priceListType: body.priceListType ?? existing.priceListType,
			basedOn: body.basedOn ?? existing.basedOn,
			parentPriceListId: body.parentPriceListId ?? existing.parentPriceListId,
			defaultPercentage: body.defaultPercentage ?? existing.defaultPercentage,
			validFrom: body.validFrom ? new Date(body.validFrom) : existing.validFrom,
			validTo: body.validTo ? new Date(body.validTo) : existing.validTo,
			priority: body.priority ?? existing.priority,
			isDefault: body.isDefault ?? existing.isDefault,
			isActive: body.isActive ?? existing.isActive,
			updatedAt: new Date()
		};

		// If setting as default, unset other defaults
		if (updated.isDefault && !existing.isDefault && updated.priceListType === 'sale') {
			priceLists = priceLists.map((pl) => ({
				...pl,
				isDefault: pl.priceListId === id ? true : pl.priceListType === 'sale' ? false : pl.isDefault
			}));
		}

		priceLists[index] = updated;

		return json(updated);
	} catch (error) {
		console.error('Error updating price list:', error);
		return json({ error: 'Invalid request body' }, { status: 400 });
	}
};

export const PATCH: RequestHandler = async ({ params, request }) => {
	// PATCH is the same as PUT for partial updates
	const { id } = params;

	const index = priceLists.findIndex((pl) => pl.priceListId === id);

	if (index === -1) {
		return json({ error: 'Price list not found' }, { status: 404 });
	}

	try {
		const body: Partial<UpdatePriceListInput> = await request.json();
		const existing = priceLists[index];

		const updated: PriceList = {
			...existing,
			...body,
			updatedAt: new Date()
		};

		priceLists[index] = updated;

		return json(updated);
	} catch (error) {
		console.error('Error updating price list:', error);
		return json({ error: 'Invalid request body' }, { status: 400 });
	}
};

export const DELETE: RequestHandler = async ({ params }) => {
	const { id } = params;

	const index = priceLists.findIndex((pl) => pl.priceListId === id);

	if (index === -1) {
		return json({ error: 'Price list not found' }, { status: 404 });
	}

	// Cannot delete default price list
	if (priceLists[index].isDefault) {
		return json({ error: 'Cannot delete default price list' }, { status: 400 });
	}

	priceLists.splice(index, 1);

	return new Response(null, { status: 204 });
};
