<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import {
		createProductImagesStore,
		type ProductImagesStore
	} from '$lib/stores/product-images.svelte';
	import type { ProductImage } from '$lib/types/product-image';

	interface Props {
		productId: string;
		readonly?: boolean;
	}

	let { productId, readonly = false }: Props = $props();

	// Create store reactively based on productId
	let store = $state<ProductImagesStore | null>(null);

	$effect(() => {
		const newStore = createProductImagesStore(productId);
		store = newStore;
		newStore.loadImages();
	});

	// Derived values for safe access
	const images = $derived(store?.images ?? []);
	const loading = $derived(store?.loading ?? true);
	const uploading = $derived(store?.uploading ?? false);
	const error = $derived(store?.error ?? null);

	// UI state
	let fileInputRef = $state<HTMLInputElement | null>(null);
	let dragOver = $state(false);
	let editingImage = $state<ProductImage | null>(null);
	let editAltText = $state('');
	let deleteConfirmImage = $state<ProductImage | null>(null);
	let lightboxImage = $state<ProductImage | null>(null);

	// Handle file selection
	async function handleFileSelect(event: Event) {
		if (!store) return;

		const input = event.target as HTMLInputElement;
		const files = input.files;

		if (!files || files.length === 0) return;

		for (const file of files) {
			if (!file.type.startsWith('image/')) {
				continue;
			}

			// Max 5MB
			if (file.size > 5 * 1024 * 1024) {
				continue;
			}

			await store.uploadImage(file);
		}

		// Reset input
		if (fileInputRef) {
			fileInputRef.value = '';
		}
	}

	// Handle drag and drop
	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		dragOver = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		dragOver = false;
	}

	async function handleDrop(event: DragEvent) {
		if (!store) return;

		event.preventDefault();
		dragOver = false;

		const files = event.dataTransfer?.files;
		if (!files || files.length === 0) return;

		for (const file of files) {
			if (!file.type.startsWith('image/')) {
				continue;
			}

			if (file.size > 5 * 1024 * 1024) {
				continue;
			}

			await store.uploadImage(file);
		}
	}

	// Open edit dialog
	function openEditDialog(image: ProductImage) {
		editingImage = image;
		editAltText = image.altText || '';
	}

	// Save alt text
	async function saveAltText() {
		if (!editingImage || !store) return;

		const success = await store.updateImage(editingImage.id, editAltText);
		if (success) {
			editingImage = null;
			editAltText = '';
		}
	}

	// Delete image
	async function confirmDelete() {
		if (!deleteConfirmImage || !store) return;

		const success = await store.deleteImage(deleteConfirmImage.id);
		if (success) {
			deleteConfirmImage = null;
		}
	}

	// Set primary image
	async function handleSetPrimary(image: ProductImage) {
		if (!store) return;
		await store.setPrimaryImage(image.id);
	}

	// Format file size
	function formatFileSize(bytes?: number): string {
		if (!bytes) return '';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between">
			<span>Product Images</span>
			{#if !readonly && images.length > 0}
				<span class="text-sm font-normal text-muted-foreground">
					{images.length} / 10 images
				</span>
			{/if}
		</CardTitle>
	</CardHeader>
	<CardContent class="space-y-4">
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<div
					class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
				></div>
			</div>
		{:else}
			<!-- Upload area -->
			{#if !readonly && images.length < 10}
				<div
					class="relative rounded-lg border-2 border-dashed p-6 transition-colors {dragOver
						? 'border-primary bg-primary/5'
						: 'border-muted-foreground/25 hover:border-muted-foreground/50'}"
					role="button"
					tabindex="0"
					ondragover={handleDragOver}
					ondragleave={handleDragLeave}
					ondrop={handleDrop}
					onclick={() => fileInputRef?.click()}
					onkeydown={(e) => e.key === 'Enter' && fileInputRef?.click()}
				>
					<input
						type="file"
						accept="image/jpeg,image/png,image/gif,image/webp"
						multiple
						class="hidden"
						bind:this={fileInputRef}
						onchange={handleFileSelect}
					/>
					<div class="flex flex-col items-center gap-2 text-center">
						{#if uploading}
							<div
								class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
							></div>
							<p class="text-sm text-muted-foreground">Uploading...</p>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="32"
								height="32"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								class="text-muted-foreground"
							>
								<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
								<polyline points="17 8 12 3 7 8" />
								<line x1="12" y1="3" x2="12" y2="15" />
							</svg>
							<p class="text-sm font-medium">Drag and drop images here, or click to select</p>
							<p class="text-xs text-muted-foreground">
								JPEG, PNG, GIF, WebP. Max 5MB per image. Up to 10 images total.
							</p>
						{/if}
					</div>
				</div>
			{/if}

			<!-- Error message -->
			{#if error}
				<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
					{error}
				</div>
			{/if}

			<!-- Image grid -->
			{#if images.length > 0}
				<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4">
					{#each images as image (image.id)}
						<div class="group relative aspect-square overflow-hidden rounded-lg border bg-muted">
							<!-- Image -->
							<button
								type="button"
								class="h-full w-full cursor-pointer"
								onclick={() => (lightboxImage = image)}
							>
								<img
									src={image.url}
									alt={image.altText || 'Product image'}
									class="h-full w-full object-cover transition-transform group-hover:scale-105"
								/>
							</button>

							<!-- Primary badge -->
							{#if image.isPrimary}
								<div
									class="absolute top-2 left-2 rounded bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground"
								>
									Primary
								</div>
							{/if}

							<!-- Actions overlay -->
							{#if !readonly}
								<div
									class="absolute inset-0 flex items-center justify-center gap-2 bg-black/50 opacity-0 transition-opacity group-hover:opacity-100"
								>
									<!-- Set as primary -->
									{#if !image.isPrimary}
										<button
											type="button"
											class="rounded-full bg-white p-2 text-gray-800 shadow-lg transition-colors hover:bg-gray-100"
											title="Set as primary"
											onclick={() => handleSetPrimary(image)}
										>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												width="16"
												height="16"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
											>
												<polygon
													points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
												/>
											</svg>
										</button>
									{/if}

									<!-- Edit alt text -->
									<button
										type="button"
										class="rounded-full bg-white p-2 text-gray-800 shadow-lg transition-colors hover:bg-gray-100"
										title="Edit alt text"
										onclick={() => openEditDialog(image)}
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="16"
											height="16"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										>
											<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
											<path d="m15 5 4 4" />
										</svg>
									</button>

									<!-- Delete -->
									<button
										type="button"
										class="rounded-full bg-white p-2 text-red-600 shadow-lg transition-colors hover:bg-red-50"
										title="Delete image"
										onclick={() => (deleteConfirmImage = image)}
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="16"
											height="16"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										>
											<path d="M3 6h18" />
											<path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
											<path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
										</svg>
									</button>
								</div>
							{/if}

							<!-- File info -->
							{#if image.fileSize}
								<div
									class="absolute right-2 bottom-2 rounded bg-black/60 px-1.5 py-0.5 text-xs text-white"
								>
									{formatFileSize(image.fileSize)}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{:else if !loading}
				<div
					class="flex flex-col items-center justify-center py-8 text-center text-muted-foreground"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="48"
						height="48"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="mb-2 opacity-50"
					>
						<rect width="18" height="18" x="3" y="3" rx="2" ry="2" />
						<circle cx="9" cy="9" r="2" />
						<path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
					</svg>
					<p>No images uploaded yet</p>
				</div>
			{/if}
		{/if}
	</CardContent>
</Card>

<!-- Edit alt text dialog -->
<Dialog.Root open={!!editingImage} onOpenChange={(open) => !open && (editingImage = null)}>
	<Dialog.Content class="sm:max-w-[425px]">
		<Dialog.Header>
			<Dialog.Title>Edit Image Details</Dialog.Title>
			<Dialog.Description>Add alt text to improve accessibility and SEO.</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4 py-4">
			{#if editingImage}
				<div class="flex justify-center">
					<img
						src={editingImage.url}
						alt={editingImage.altText || 'Product image'}
						class="h-32 w-32 rounded-lg object-cover"
					/>
				</div>
			{/if}
			<div class="space-y-2">
				<Label for="altText">Alt Text</Label>
				<Input id="altText" bind:value={editAltText} placeholder="Describe the image..." />
				<p class="text-xs text-muted-foreground">
					Describe what's shown in the image for screen readers.
				</p>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (editingImage = null)}>Cancel</Button>
			<Button onclick={saveAltText}>Save</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete confirmation dialog -->
<Dialog.Root
	open={!!deleteConfirmImage}
	onOpenChange={(open) => !open && (deleteConfirmImage = null)}
>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete Image</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete this image? This action cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		{#if deleteConfirmImage}
			<div class="flex justify-center py-4">
				<img
					src={deleteConfirmImage.url}
					alt={deleteConfirmImage.altText || 'Product image'}
					class="h-32 w-32 rounded-lg object-cover"
				/>
			</div>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteConfirmImage = null)}>Cancel</Button>
			<Button variant="destructive" onclick={confirmDelete}>Delete</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Lightbox dialog -->
<Dialog.Root open={!!lightboxImage} onOpenChange={(open) => !open && (lightboxImage = null)}>
	<Dialog.Content class="max-w-4xl">
		{#if lightboxImage}
			<div class="relative">
				<img
					src={lightboxImage.url}
					alt={lightboxImage.altText || 'Product image'}
					class="w-full rounded-lg"
				/>
				{#if lightboxImage.width && lightboxImage.height}
					<div class="mt-2 text-center text-sm text-muted-foreground">
						{lightboxImage.width} x {lightboxImage.height}
						{#if lightboxImage.fileSize}
							&bull; {formatFileSize(lightboxImage.fileSize)}
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
