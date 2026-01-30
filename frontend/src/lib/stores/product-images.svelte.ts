/**
 * Product Images Store
 *
 * Svelte 5 runes-based store for managing product images state.
 */

import { productImageApi } from '$lib/api/inventory/product-images';
import type { ProductImage } from '$lib/types/product-image';

/**
 * Create a product images store for a specific product
 */
export function createProductImagesStore(productId: string) {
	let images = $state<ProductImage[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let uploading = $state(false);

	/**
	 * Load images for the product
	 */
	async function loadImages() {
		loading = true;
		error = null;

		const response = await productImageApi.list(productId);

		if (response.success && response.data) {
			images = response.data.images;
		} else {
			error = response.error || 'Failed to load images';
		}

		loading = false;
	}

	/**
	 * Upload a new image
	 */
	async function uploadImage(file: File): Promise<boolean> {
		uploading = true;
		error = null;

		const response = await productImageApi.upload(productId, file);

		if (response.success && response.data) {
			// Add the new image to the list
			images = [...images, response.data.image];
			uploading = false;
			return true;
		} else {
			error = response.error || 'Failed to upload image';
			uploading = false;
			return false;
		}
	}

	/**
	 * Delete an image
	 */
	async function deleteImage(imageId: string): Promise<boolean> {
		error = null;

		const response = await productImageApi.delete(productId, imageId);

		if (response.success) {
			// Remove the image from the list
			images = images.filter((img) => img.id !== imageId);
			return true;
		} else {
			error = response.error || 'Failed to delete image';
			return false;
		}
	}

	/**
	 * Update image metadata (alt text)
	 */
	async function updateImage(imageId: string, altText: string): Promise<boolean> {
		error = null;

		const response = await productImageApi.update(productId, imageId, { altText });

		if (response.success && response.data) {
			// Update the image in the list
			images = images.map((img) => (img.id === imageId ? response.data! : img));
			return true;
		} else {
			error = response.error || 'Failed to update image';
			return false;
		}
	}

	/**
	 * Set an image as primary
	 */
	async function setPrimaryImage(imageId: string): Promise<boolean> {
		error = null;

		const response = await productImageApi.setPrimary(productId, imageId);

		if (response.success) {
			// Update the primary flag in the list
			images = images.map((img) => ({
				...img,
				isPrimary: img.id === imageId
			}));
			return true;
		} else {
			error = response.error || 'Failed to set primary image';
			return false;
		}
	}

	/**
	 * Reorder images
	 */
	async function reorderImages(imageIds: string[]): Promise<boolean> {
		error = null;

		const response = await productImageApi.reorder(productId, imageIds);

		if (response.success) {
			// Update positions in the list
			images = imageIds
				.map((id, index) => {
					const img = images.find((i) => i.id === id);
					if (img) {
						return { ...img, position: index };
					}
					return null;
				})
				.filter((img): img is ProductImage => img !== null);
			return true;
		} else {
			error = response.error || 'Failed to reorder images';
			return false;
		}
	}

	return {
		get images() {
			return images;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get uploading() {
			return uploading;
		},
		get primaryImage() {
			return images.find((img) => img.isPrimary) || images[0] || null;
		},
		loadImages,
		uploadImage,
		deleteImage,
		updateImage,
		setPrimaryImage,
		reorderImages
	};
}

export type ProductImagesStore = ReturnType<typeof createProductImagesStore>;
