/**
 * Pricing API - Customer Price List Assignments (Mock)
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import type { CustomerPriceList, AssignCustomerInput } from '$lib/types/pricing';

// Mock customer assignments
let customerPriceLists: CustomerPriceList[] = [
	{
		id: 'cpl-001',
		tenantId: 'tenant-001',
		customerId: 'cust-001',
		priceListId: 'pl-002',
		priority: 0,
		createdAt: new Date(),
		customer: { name: 'ABC Corporation', email: 'abc@example.com' }
	},
	{
		id: 'cpl-002',
		tenantId: 'tenant-001',
		customerId: 'cust-002',
		priceListId: 'pl-002',
		priority: 0,
		validFrom: new Date('2026-01-01'),
		validTo: new Date('2026-12-31'),
		createdAt: new Date(),
		customer: { name: 'XYZ Trading', email: 'xyz@example.com' }
	},
	{
		id: 'cpl-003',
		tenantId: 'tenant-001',
		customerId: 'cust-003',
		priceListId: 'pl-003',
		priority: 0,
		createdAt: new Date(),
		customer: { name: 'Tech Solutions Ltd', email: 'tech@example.com' }
	}
];

export const GET: RequestHandler = async ({ params }) => {
	const { id } = params;

	// Filter assignments for this price list
	const assignments = customerPriceLists.filter((cpl) => cpl.priceListId === id);

	return json({
		data: assignments,
		meta: {
			total: assignments.length
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
		const { customerId, ...data }: { customerId: string } & AssignCustomerInput = body;

		if (!customerId) {
			return json({ error: 'Customer ID is required' }, { status: 400 });
		}

		// Check if already assigned
		const existing = customerPriceLists.find(
			(cpl) => cpl.priceListId === id && cpl.customerId === customerId
		);

		if (existing) {
			return json({ error: 'Customer is already assigned to this price list' }, { status: 409 });
		}

		// Create new assignment
		const assignment: CustomerPriceList = {
			id: `cpl-${Date.now()}`,
			tenantId: 'tenant-001',
			customerId,
			priceListId: id,
			priority: data.priority || 0,
			validFrom: data.validFrom ? new Date(data.validFrom) : undefined,
			validTo: data.validTo ? new Date(data.validTo) : undefined,
			createdAt: new Date()
		};

		customerPriceLists.push(assignment);

		return json(assignment, { status: 201 });
	} catch (error) {
		console.error('Error assigning customer:', error);
		return json({ error: 'Invalid request body' }, { status: 400 });
	}
};

export const DELETE: RequestHandler = async ({ params, url }) => {
	const { id } = params;
	const customerId = url.searchParams.get('customerId');

	if (!customerId) {
		return json({ error: 'Customer ID is required' }, { status: 400 });
	}

	const index = customerPriceLists.findIndex(
		(cpl) => cpl.priceListId === id && cpl.customerId === customerId
	);

	if (index === -1) {
		return json({ error: 'Assignment not found' }, { status: 404 });
	}

	customerPriceLists.splice(index, 1);

	return new Response(null, { status: 204 });
};
