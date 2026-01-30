<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import * as Tabs from '$lib/components/ui/tabs';
	import type {
		CategoryResponse,
		CategoryCreateRequest,
		CategoryUpdateRequest
	} from '$lib/types/inventory';

	interface Props {
		open: boolean;
		category?: CategoryResponse | null;
		parentId?: string | null;
		categories: CategoryResponse[];
		isSubmitting?: boolean;
		onClose: () => void;
		onSubmit: (data: CategoryCreateRequest | CategoryUpdateRequest) => void;
	}

	let {
		open = false,
		category = null,
		parentId = null,
		categories = [],
		isSubmitting = false,
		onClose,
		onSubmit
	}: Props = $props();

	// Form state
	let name = $state('');
	let code = $state('');
	let description = $state('');
	let selectedParentId = $state<string | null>(null);
	let displayOrder = $state(0);
	let isActive = $state(true);
	let isVisible = $state(true);
	let icon = $state('');
	let color = $state('');
	let imageUrl = $state('');
	let slug = $state('');
	let metaTitle = $state('');
	let metaDescription = $state('');

	// Image upload state
	let imageMode = $state<'url' | 'upload'>('url');
	let imageFile = $state<File | null>(null);
	let isUploadingImage = $state(false);
	let imageError = $state('');
	let fileInputRef = $state<HTMLInputElement | null>(null);

	// Track if slug/code were manually edited
	let slugManuallyEdited = $state(false);
	let codeManuallyEdited = $state(false);

	// Validation
	let errors = $state<Record<string, string>>({});

	const isEditing = $derived(!!category);
	const title = $derived(isEditing ? 'Edit Category' : 'Add Category');

	// Filter out current category and its descendants for parent selection
	const availableParents = $derived.by(() => {
		if (!category) return categories.filter((c) => c.isActive);

		const descendantIds = new Set<string>();
		const addDescendants = (id: string) => {
			descendantIds.add(id);
			categories
				.filter((c) => c.parentCategoryId === id)
				.forEach((c) => addDescendants(c.categoryId));
		};
		addDescendants(category.categoryId);

		return categories.filter((c) => c.isActive && !descendantIds.has(c.categoryId));
	});

	// Vietnamese character mapping (shared between slug and code generation)
	const vietnameseMap: Record<string, string> = {
		à: 'a',
		á: 'a',
		ạ: 'a',
		ả: 'a',
		ã: 'a',
		â: 'a',
		ầ: 'a',
		ấ: 'a',
		ậ: 'a',
		ẩ: 'a',
		ẫ: 'a',
		ă: 'a',
		ằ: 'a',
		ắ: 'a',
		ặ: 'a',
		ẳ: 'a',
		ẵ: 'a',
		è: 'e',
		é: 'e',
		ẹ: 'e',
		ẻ: 'e',
		ẽ: 'e',
		ê: 'e',
		ề: 'e',
		ế: 'e',
		ệ: 'e',
		ể: 'e',
		ễ: 'e',
		ì: 'i',
		í: 'i',
		ị: 'i',
		ỉ: 'i',
		ĩ: 'i',
		ò: 'o',
		ó: 'o',
		ọ: 'o',
		ỏ: 'o',
		õ: 'o',
		ô: 'o',
		ồ: 'o',
		ố: 'o',
		ộ: 'o',
		ổ: 'o',
		ỗ: 'o',
		ơ: 'o',
		ờ: 'o',
		ớ: 'o',
		ợ: 'o',
		ở: 'o',
		ỡ: 'o',
		ù: 'u',
		ú: 'u',
		ụ: 'u',
		ủ: 'u',
		ũ: 'u',
		ư: 'u',
		ừ: 'u',
		ứ: 'u',
		ự: 'u',
		ử: 'u',
		ữ: 'u',
		ỳ: 'y',
		ý: 'y',
		ỵ: 'y',
		ỷ: 'y',
		ỹ: 'y',
		đ: 'd',
		// Uppercase versions
		À: 'A',
		Á: 'A',
		Ạ: 'A',
		Ả: 'A',
		Ã: 'A',
		Â: 'A',
		Ầ: 'A',
		Ấ: 'A',
		Ậ: 'A',
		Ẩ: 'A',
		Ẫ: 'A',
		Ă: 'A',
		Ằ: 'A',
		Ắ: 'A',
		Ặ: 'A',
		Ẳ: 'A',
		Ẵ: 'A',
		È: 'E',
		É: 'E',
		Ẹ: 'E',
		Ẻ: 'E',
		Ẽ: 'E',
		Ê: 'E',
		Ề: 'E',
		Ế: 'E',
		Ệ: 'E',
		Ể: 'E',
		Ễ: 'E',
		Ì: 'I',
		Í: 'I',
		Ị: 'I',
		Ỉ: 'I',
		Ĩ: 'I',
		Ò: 'O',
		Ó: 'O',
		Ọ: 'O',
		Ỏ: 'O',
		Õ: 'O',
		Ô: 'O',
		Ồ: 'O',
		Ố: 'O',
		Ộ: 'O',
		Ổ: 'O',
		Ỗ: 'O',
		Ơ: 'O',
		Ờ: 'O',
		Ớ: 'O',
		Ợ: 'O',
		Ở: 'O',
		Ỡ: 'O',
		Ù: 'U',
		Ú: 'U',
		Ụ: 'U',
		Ủ: 'U',
		Ũ: 'U',
		Ư: 'U',
		Ừ: 'U',
		Ứ: 'U',
		Ự: 'U',
		Ử: 'U',
		Ữ: 'U',
		Ỳ: 'Y',
		Ý: 'Y',
		Ỵ: 'Y',
		Ỷ: 'Y',
		Ỹ: 'Y',
		Đ: 'D'
	};

	// Helper to transliterate Vietnamese characters
	function transliterate(text: string): string {
		return text
			.split('')
			.map((char) => vietnameseMap[char] || char)
			.join('');
	}

	// Generate slug from name (with Vietnamese transliteration)
	function generateSlug(text: string): string {
		return transliterate(text)
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-+|-+$/g, '');
	}

	// Generate code from name (with Vietnamese transliteration)
	function generateCode(text: string): string {
		return transliterate(text)
			.toUpperCase()
			.replace(/[^A-Z0-9]+/g, '_')
			.replace(/^_+|_+$/g, '')
			.slice(0, 20);
	}

	// Handle file selection and convert to data URL
	function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];

		if (!file) return;

		// Validate file type
		if (!file.type.startsWith('image/')) {
			imageError = 'Please select an image file';
			return;
		}

		// Validate file size (max 5MB)
		if (file.size > 5 * 1024 * 1024) {
			imageError = 'Image must be less than 5MB';
			return;
		}

		imageError = '';
		imageFile = file;
		isUploadingImage = true;

		// Convert to data URL for preview and storage
		const reader = new FileReader();
		reader.onload = (e) => {
			imageUrl = e.target?.result as string;
			isUploadingImage = false;
		};
		reader.onerror = () => {
			imageError = 'Failed to read file';
			isUploadingImage = false;
		};
		reader.readAsDataURL(file);
	}

	// Clear uploaded image
	function clearImage() {
		imageUrl = '';
		imageFile = null;
		imageError = '';
		if (fileInputRef) {
			fileInputRef.value = '';
		}
	}

	// Reset form when dialog opens
	$effect(() => {
		if (open) {
			if (category) {
				name = category.name;
				code = category.code ?? '';
				description = category.description ?? '';
				selectedParentId = category.parentCategoryId ?? null;
				displayOrder = category.displayOrder;
				isActive = category.isActive;
				isVisible = category.isVisible;
				icon = category.icon ?? '';
				color = category.color ?? '';
				imageUrl = category.imageUrl ?? '';
				slug = category.slug ?? '';
				metaTitle = category.metaTitle ?? '';
				metaDescription = category.metaDescription ?? '';
				// Set image mode based on whether URL looks like a data URL
				imageMode = category.imageUrl?.startsWith('data:') ? 'upload' : 'url';
			} else {
				name = '';
				code = '';
				description = '';
				selectedParentId = parentId;
				displayOrder = 0;
				isActive = true;
				isVisible = true;
				icon = '';
				color = '';
				imageUrl = '';
				slug = '';
				metaTitle = '';
				metaDescription = '';
				imageMode = 'url';
			}
			// Reset image state
			imageFile = null;
			imageError = '';
			isUploadingImage = false;
			// Reset manual edit flags
			slugManuallyEdited = isEditing; // Don't auto-generate for existing categories
			codeManuallyEdited = isEditing;
			errors = {};
		}
	});

	// Auto-generate slug and code when name changes (only if not manually edited)
	$effect(() => {
		if (!isEditing && name && !slugManuallyEdited) {
			slug = generateSlug(name);
		}
	});

	$effect(() => {
		if (!isEditing && name && !codeManuallyEdited) {
			code = generateCode(name);
		}
	});

	function validate(): boolean {
		const newErrors: Record<string, string> = {};

		if (!name.trim()) {
			newErrors.name = 'Name is required';
		} else if (name.length > 100) {
			newErrors.name = 'Name must be 100 characters or less';
		}

		if (code && code.length > 20) {
			newErrors.code = 'Code must be 20 characters or less';
		}

		if (description && description.length > 500) {
			newErrors.description = 'Description must be 500 characters or less';
		}

		if (displayOrder < 0) {
			newErrors.displayOrder = 'Display order must be 0 or greater';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	function handleSubmit() {
		if (!validate()) return;

		const data: CategoryCreateRequest | CategoryUpdateRequest = {
			name: name.trim(),
			code: code.trim() || null,
			description: description.trim() || null,
			parentCategoryId: selectedParentId,
			displayOrder,
			isActive,
			isVisible,
			icon: icon.trim() || null,
			color: color.trim() || null,
			imageUrl: imageUrl.trim() || null,
			slug: slug.trim() || null,
			metaTitle: metaTitle.trim() || null,
			metaDescription: metaDescription.trim() || null
		};

		onSubmit(data);
	}

	function handleParentChange(value: string | undefined) {
		selectedParentId = value === 'none' ? null : (value ?? null);
	}
</script>

<Dialog.Root bind:open onOpenChange={(value) => !value && onClose()}>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-[600px]">
		<Dialog.Header>
			<Dialog.Title>{title}</Dialog.Title>
			<Dialog.Description>
				{isEditing ? 'Update category details' : 'Create a new product category'}
			</Dialog.Description>
		</Dialog.Header>

		<form
			onsubmit={(e) => {
				e.preventDefault();
				handleSubmit();
			}}
			class="space-y-6"
		>
			<!-- Basic Info -->
			<div class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="name">Name *</Label>
						<Input
							id="name"
							bind:value={name}
							placeholder="Category name"
							class={errors.name ? 'border-destructive' : ''}
						/>
						{#if errors.name}
							<p class="text-sm text-destructive">{errors.name}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="code">Code</Label>
						<Input
							id="code"
							bind:value={code}
							placeholder="AUTO_GENERATED"
							class={errors.code ? 'border-destructive' : ''}
							oninput={() => (codeManuallyEdited = true)}
						/>
						{#if errors.code}
							<p class="text-sm text-destructive">{errors.code}</p>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<Label for="parent">Parent Category</Label>
					<Select.Root
						type="single"
						value={selectedParentId ?? 'none'}
						onValueChange={handleParentChange}
					>
						<Select.Trigger class="w-full">
							{#if selectedParentId}
								{availableParents.find((c) => c.categoryId === selectedParentId)?.name ??
									'Select parent'}
							{:else}
								None (Root Category)
							{/if}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="none">None (Root Category)</Select.Item>
							{#each availableParents as parent (parent.categoryId)}
								<Select.Item value={parent.categoryId}>
									{'—'.repeat(parent.level)}
									{parent.name}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="description">Description</Label>
					<Textarea
						id="description"
						bind:value={description}
						placeholder="Category description"
						rows={3}
						class={errors.description ? 'border-destructive' : ''}
					/>
					{#if errors.description}
						<p class="text-sm text-destructive">{errors.description}</p>
					{/if}
				</div>
			</div>

			<!-- Display Settings -->
			<div class="space-y-4">
				<h3 class="text-sm font-medium">Display Settings</h3>
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="displayOrder">Display Order</Label>
						<Input
							id="displayOrder"
							type="number"
							bind:value={displayOrder}
							min={0}
							class={errors.displayOrder ? 'border-destructive' : ''}
						/>
						{#if errors.displayOrder}
							<p class="text-sm text-destructive">{errors.displayOrder}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="icon">Icon (emoji)</Label>
						<Input id="icon" bind:value={icon} placeholder="📁" />
					</div>
				</div>

				<div class="flex gap-6">
					<div class="flex items-center gap-2">
						<Checkbox id="isActive" bind:checked={isActive} />
						<Label for="isActive" class="font-normal">Active</Label>
					</div>
					<div class="flex items-center gap-2">
						<Checkbox id="isVisible" bind:checked={isVisible} />
						<Label for="isVisible" class="font-normal">Visible in Store</Label>
					</div>
				</div>
			</div>

			<!-- Image -->
			<div class="space-y-4">
				<h3 class="text-sm font-medium">Image</h3>
				<Tabs.Root bind:value={imageMode} class="w-full">
					<Tabs.List class="grid w-full grid-cols-2">
						<Tabs.Trigger value="upload">Upload</Tabs.Trigger>
						<Tabs.Trigger value="url">URL</Tabs.Trigger>
					</Tabs.List>

					<Tabs.Content value="upload" class="space-y-4 pt-4">
						<div class="space-y-2">
							<Label for="imageFile">Choose Image</Label>
							<div class="flex items-center gap-2">
								<Input
									id="imageFile"
									type="file"
									accept="image/*"
									onchange={handleFileSelect}
									bind:ref={fileInputRef}
									class="flex-1"
								/>
								{#if imageUrl && imageMode === 'upload'}
									<Button type="button" variant="outline" size="sm" onclick={clearImage}>
										Clear
									</Button>
								{/if}
							</div>
							<p class="text-xs text-muted-foreground">Accepts JPG, PNG, GIF, WebP. Max 5MB.</p>
							{#if imageError}
								<p class="text-sm text-destructive">{imageError}</p>
							{/if}
							{#if isUploadingImage}
								<p class="text-sm text-muted-foreground">Processing image...</p>
							{/if}
						</div>
					</Tabs.Content>

					<Tabs.Content value="url" class="space-y-4 pt-4">
						<div class="space-y-2">
							<Label for="imageUrl">Image URL</Label>
							<Input
								id="imageUrl"
								type="url"
								bind:value={imageUrl}
								placeholder="https://example.com/image.jpg"
							/>
						</div>
					</Tabs.Content>
				</Tabs.Root>

				<!-- Image Preview -->
				{#if imageUrl}
					<div class="flex flex-col items-center gap-2">
						<img
							src={imageUrl}
							alt="Category preview"
							class="h-32 w-32 rounded-lg border object-cover"
							onerror={(e) => {
								const target = e.target as HTMLImageElement;
								target.style.display = 'none';
							}}
						/>
						{#if imageFile}
							<p class="text-xs text-muted-foreground">{imageFile.name}</p>
						{/if}
					</div>
				{/if}
			</div>

			<!-- SEO -->
			<details class="space-y-4">
				<summary class="cursor-pointer text-sm font-medium">SEO Settings (optional)</summary>
				<div class="space-y-4 pt-4">
					<div class="space-y-2">
						<Label for="slug">URL Slug</Label>
						<Input
							id="slug"
							bind:value={slug}
							placeholder="category-slug"
							oninput={() => (slugManuallyEdited = true)}
						/>
					</div>
					<div class="space-y-2">
						<Label for="metaTitle">Meta Title</Label>
						<Input id="metaTitle" bind:value={metaTitle} placeholder="SEO title" />
					</div>
					<div class="space-y-2">
						<Label for="metaDescription">Meta Description</Label>
						<Textarea
							id="metaDescription"
							bind:value={metaDescription}
							placeholder="SEO description"
							rows={2}
						/>
					</div>
				</div>
			</details>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={onClose} disabled={isSubmitting}>
					Cancel
				</Button>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						Saving...
					{:else}
						{isEditing ? 'Update Category' : 'Create Category'}
					{/if}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
