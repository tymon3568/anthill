// =============================================================================
// Product Import/Export Types
// =============================================================================

/**
 * CSV row representation for product import/export
 */
export interface ProductCsvRow {
	sku: string;
	name: string;
	description?: string;
	productType?: string;
	categoryId?: string;
	salePrice?: number;
	costPrice?: number;
	currency?: string;
	weight?: number;
	length?: number;
	width?: number;
	height?: number;
	barcode?: string;
	barcodeType?: string;
	isActive?: boolean;
}

/**
 * Error for a specific row in the import file
 */
export interface ImportRowError {
	rowNumber: number;
	field: string;
	error: string;
}

/**
 * Result of validating a CSV file before import
 */
export interface ImportValidationResult {
	isValid: boolean;
	totalRows: number;
	validRows: number;
	errors: ImportRowError[];
	preview?: ProductCsvRow[];
}

/**
 * Result of importing products from CSV
 */
export interface ImportResult {
	created: number;
	updated: number;
	failed: number;
	errors: ImportRowError[];
}

/**
 * Query parameters for exporting products
 */
export interface ExportProductsQuery {
	categoryId?: string;
	productType?: string;
	isActive?: boolean;
	search?: string;
}

/**
 * Import wizard step
 */
export type ImportStep = 'upload' | 'validate' | 'import' | 'complete';

/**
 * Import wizard state
 */
export interface ImportWizardState {
	step: ImportStep;
	file: File | null;
	validationResult: ImportValidationResult | null;
	importResult: ImportResult | null;
	upsertMode: boolean;
	isLoading: boolean;
	error: string | null;
}
