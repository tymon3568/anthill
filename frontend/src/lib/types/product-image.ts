/**
 * Product Image Types
 *
 * Types for product image management API operations.
 */

// ============================================================
// PRODUCT IMAGE TYPES
// ============================================================

/**
 * Product image entity
 */
export interface ProductImage {
	id: string;
	productId: string;
	url: string;
	altText?: string;
	position: number;
	isPrimary: boolean;
	fileSize?: number;
	mimeType?: string;
	width?: number;
	height?: number;
	createdAt: string;
}

/**
 * Response for listing product images
 */
export interface ProductImagesListResponse {
	images: ProductImage[];
	total: number;
}

/**
 * Response for image upload
 */
export interface UploadImageResponse {
	image: ProductImage;
	message: string;
}

/**
 * Response for image deletion
 */
export interface DeleteImageResponse {
	success: boolean;
	message: string;
}

/**
 * Request to update image metadata
 */
export interface UpdateProductImageRequest {
	altText?: string;
}

/**
 * Request to reorder images
 */
export interface ReorderImagesRequest {
	imageIds: string[];
}
