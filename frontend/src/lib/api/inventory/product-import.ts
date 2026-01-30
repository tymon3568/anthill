// =============================================================================
// Product Import/Export API Client
// =============================================================================

import { getCurrentTenantSlug } from '$lib/tenant';
import { transformKeysToCamelCase } from '$lib/api/client';
import type {
	ImportValidationResult,
	ImportResult,
	ExportProductsQuery
} from '$lib/types/product-import';

const BASE_PATH = '/api/v1/inventory/products/import';

/**
 * Build headers with tenant context for import/export requests
 */
function buildHeaders(contentType?: string): Record<string, string> {
	const headers: Record<string, string> = {};

	if (contentType) {
		headers['Content-Type'] = contentType;
	}

	const tenantSlug = getCurrentTenantSlug();
	if (tenantSlug) {
		headers['X-Tenant-ID'] = tenantSlug;
	}

	return headers;
}

/**
 * Product Import/Export API
 */
export const productImportApi = {
	/**
	 * Download CSV template
	 */
	async getTemplate(): Promise<Blob> {
		const response = await fetch(`${BASE_PATH}/template`, {
			method: 'GET',
			headers: buildHeaders(),
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error('Failed to download template');
		}

		return response.blob();
	},

	/**
	 * Validate CSV file before import
	 */
	async validateCsv(file: File): Promise<ImportValidationResult> {
		const response = await fetch(`${BASE_PATH}/validate`, {
			method: 'POST',
			headers: buildHeaders('text/csv'),
			credentials: 'include',
			body: await file.text()
		});

		if (!response.ok) {
			const error = await response.json().catch(() => ({ message: 'Failed to validate CSV' }));
			throw new Error(error.message || error.error || 'Failed to validate CSV');
		}

		const rawData = await response.json();
		return transformKeysToCamelCase<ImportValidationResult>(rawData);
	},

	/**
	 * Import products from CSV
	 */
	async importCsv(file: File, upsert: boolean = false): Promise<ImportResult> {
		const params = new URLSearchParams();
		if (upsert) {
			params.set('upsert', 'true');
		}

		const url = `${BASE_PATH}/import${params.toString() ? '?' + params.toString() : ''}`;

		const response = await fetch(url, {
			method: 'POST',
			headers: buildHeaders('text/csv'),
			credentials: 'include',
			body: await file.text()
		});

		if (!response.ok) {
			const error = await response.json().catch(() => ({ message: 'Failed to import products' }));
			throw new Error(error.message || error.error || 'Failed to import products');
		}

		const rawData = await response.json();
		return transformKeysToCamelCase<ImportResult>(rawData);
	},

	/**
	 * Export products to CSV
	 */
	async exportCsv(query?: ExportProductsQuery): Promise<Blob> {
		const params = new URLSearchParams();

		if (query?.categoryId) {
			params.set('categoryId', query.categoryId);
		}
		if (query?.productType) {
			params.set('productType', query.productType);
		}
		if (query?.isActive !== undefined) {
			params.set('isActive', String(query.isActive));
		}
		if (query?.search) {
			params.set('search', query.search);
		}

		const url = `${BASE_PATH}/export${params.toString() ? '?' + params.toString() : ''}`;

		const response = await fetch(url, {
			method: 'GET',
			headers: buildHeaders(),
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error('Failed to export products');
		}

		return response.blob();
	},

	/**
	 * Download a blob as a file
	 */
	downloadBlob(blob: Blob, filename: string): void {
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
};
