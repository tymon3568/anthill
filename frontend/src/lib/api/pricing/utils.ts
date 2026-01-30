// =============================================================================
// Pricing API Utilities
// Shared helper functions for pricing API clients
// =============================================================================

/**
 * Build query string from params object
 * Filters out undefined and null values
 */
export function buildQueryString(params: Record<string, unknown>): string {
	const searchParams = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value !== undefined && value !== null) {
			searchParams.append(key, String(value));
		}
	}
	const query = searchParams.toString();
	return query ? `?${query}` : '';
}

/**
 * Convert typed object to Record<string, unknown> for API client
 * This is a type-safe way to pass typed objects to the generic API client
 */
export function toRecord<T extends object>(obj: T): Record<string, unknown> {
	return obj as unknown as Record<string, unknown>;
}
