export type ActivityType = 'receipt' | 'shipment' | 'transfer' | 'adjustment' | 'count';

export interface ActivityItem {
	id: string;
	type: ActivityType;
	description: string;
	reference: string;
	timestamp: string;
	user?: string;
}

export interface LowStockItem {
	productId: string;
	sku: string;
	name: string;
	currentStock: number;
	minStock: number;
	warehouseName: string;
	severity: 'critical' | 'warning' | 'low';
}
