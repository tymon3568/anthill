// =============================================================================
// Stock Adjustment Reason Codes
// Static data until backend provides /adjustment-reasons endpoint
// =============================================================================

import type {
	AdjustmentReasonCode,
	ReasonCodeOption
} from '$lib/types/inventory';

/**
 * Predefined reason codes for stock adjustments
 * Grouped by category for UI organization
 */
export const REASON_CODES: ReasonCodeOption[] = [
	// Inventory Count Category
	{
		code: 'COUNT_ERROR',
		label: 'Count Discrepancy',
		direction: 'both',
		category: 'inventory_count',
		requiresApproval: false
	},
	{
		code: 'STOCK_TAKE_VARIANCE',
		label: 'Stock Take Variance',
		direction: 'both',
		category: 'inventory_count',
		requiresApproval: true
	},
	{
		code: 'FOUND',
		label: 'Found Inventory',
		direction: 'increase',
		category: 'inventory_count',
		requiresApproval: true
	},

	// Quality Category
	{
		code: 'DAMAGE',
		label: 'Damaged Goods',
		direction: 'decrease',
		category: 'quality',
		requiresApproval: true
	},
	{
		code: 'EXPIRED',
		label: 'Expired Products',
		direction: 'decrease',
		category: 'quality',
		requiresApproval: true
	},
	{
		code: 'SCRAP',
		label: 'Scrap/Waste',
		direction: 'decrease',
		category: 'quality',
		requiresApproval: false
	},

	// Loss Category
	{
		code: 'THEFT',
		label: 'Theft/Shrinkage',
		direction: 'decrease',
		category: 'loss',
		requiresApproval: true
	},
	{
		code: 'LOST',
		label: 'Lost Inventory',
		direction: 'decrease',
		category: 'loss',
		requiresApproval: true
	},
	{
		code: 'WRITE_OFF',
		label: 'Write-off',
		direction: 'decrease',
		category: 'loss',
		requiresApproval: true
	},

	// Other Category
	{
		code: 'SAMPLE',
		label: 'Sample/Demo Use',
		direction: 'decrease',
		category: 'other',
		requiresApproval: false
	},
	{
		code: 'PROMOTION',
		label: 'Promotional Giveaway',
		direction: 'decrease',
		category: 'other',
		requiresApproval: false
	},
	{
		code: 'INTERNAL_USE',
		label: 'Internal Consumption',
		direction: 'decrease',
		category: 'other',
		requiresApproval: false
	},
	{
		code: 'RETURN_TO_STOCK',
		label: 'Return to Stock',
		direction: 'increase',
		category: 'other',
		requiresApproval: false
	},
	{
		code: 'CORRECTION',
		label: 'Data Correction',
		direction: 'both',
		category: 'other',
		requiresApproval: true
	}
];

/**
 * Get reason codes filtered by direction
 */
export function getReasonCodesByDirection(
	direction: 'increase' | 'decrease'
): ReasonCodeOption[] {
	return REASON_CODES.filter(
		(r) => r.direction === direction || r.direction === 'both'
	);
}

/**
 * Get reason codes grouped by category
 */
export function getReasonCodesByCategory(): Record<string, ReasonCodeOption[]> {
	return REASON_CODES.reduce(
		(acc, reason) => {
			if (!acc[reason.category]) {
				acc[reason.category] = [];
			}
			acc[reason.category].push(reason);
			return acc;
		},
		{} as Record<string, ReasonCodeOption[]>
	);
}

/**
 * Get a reason code option by its code
 */
export function getReasonCodeOption(
	code: AdjustmentReasonCode
): ReasonCodeOption | undefined {
	return REASON_CODES.find((r) => r.code === code);
}

/**
 * Get the label for a reason code
 */
export function getReasonCodeLabel(code: string): string {
	const option = REASON_CODES.find((r) => r.code === code);
	return option?.label ?? code;
}

/**
 * Category labels for display
 */
export const CATEGORY_LABELS: Record<string, string> = {
	inventory_count: 'Inventory Count',
	quality: 'Quality Issues',
	loss: 'Loss & Shrinkage',
	other: 'Other Adjustments'
};
