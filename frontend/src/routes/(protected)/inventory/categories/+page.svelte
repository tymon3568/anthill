<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Label } from '$lib/components/ui/label';
	import { Folder } from 'lucide-svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { CategoryTree, CategoryForm } from '$lib/components/inventory';
	import { categoryState, categoryStore } from '$lib/stores/inventory.svelte';
	import type {
		CategoryResponse,
		CategoryCreateRequest,
		CategoryUpdateRequest
	} from '$lib/types/inventory';

	// Page state
	let showInactive = $state(false);
	let isFormOpen = $state(false);
	let editingCategory = $state<CategoryResponse | null>(null);
	let parentIdForNew = $state<string | null>(null);
	let isSubmitting = $state(false);

	// Delete confirmation state
	let deleteDialogOpen = $state(false);
	let categoryToDelete = $state<CategoryResponse | null>(null);
	let isDeleting = $state(false);

	// Computed values
	const categories = $derived(
		showInactive ? categoryState.items : categoryState.items.filter((c) => c.isActive)
	);

	const selectedCategory = $derived(categoryState.selected);

	const stats = $derived.by(() => {
		const total = categoryState.items.length;
		const active = categoryState.items.filter((c) => c.isActive).length;
		const inactive = total - active;
		const rootCount = categoryState.items.filter((c) => !c.parentCategoryId).length;
		return { total, active, inactive, rootCount };
	});

	async function loadCategories() {
		await categoryStore.load({ pageSize: 100 }); // Max pageSize allowed by backend
	}

	function handleSelect(category: CategoryResponse) {
		categoryStore.select(category);
	}

	function handleAddRoot() {
		editingCategory = null;
		parentIdForNew = null;
		isFormOpen = true;
	}

	function handleAddChild(parentId: string) {
		editingCategory = null;
		parentIdForNew = parentId;
		isFormOpen = true;
	}

	function handleEdit(category: CategoryResponse) {
		editingCategory = category;
		parentIdForNew = null;
		isFormOpen = true;
	}

	function handleDeleteClick(category: CategoryResponse) {
		categoryToDelete = category;
		deleteDialogOpen = true;
	}

	async function handleDelete() {
		if (!categoryToDelete) return;

		isDeleting = true;
		const success = await categoryStore.delete(categoryToDelete.categoryId);

		if (success) {
			deleteDialogOpen = false;
			categoryToDelete = null;
		}
		isDeleting = false;
	}

	function handleFormClose() {
		isFormOpen = false;
		editingCategory = null;
		parentIdForNew = null;
	}

	async function handleFormSubmit(data: CategoryCreateRequest | CategoryUpdateRequest) {
		isSubmitting = true;

		if (editingCategory) {
			const result = await categoryStore.update(editingCategory.categoryId, data);
			if (result) {
				handleFormClose();
			}
		} else {
			const createData = data as CategoryCreateRequest;
			if (parentIdForNew) {
				createData.parentCategoryId = parentIdForNew;
			}
			const result = await categoryStore.create(createData);
			if (result) {
				handleFormClose();
			}
		}

		isSubmitting = false;
	}

	onMount(() => {
		loadCategories();
	});
</script>

<svelte:head>
	<title>Categories | Inventory</title>
</svelte:head>

<div class="container mx-auto space-y-6 py-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Product Categories</h1>
			<p class="text-muted-foreground">Organize your products with hierarchical categories</p>
		</div>
		<Button onclick={handleAddRoot}>+ Add Category</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold">{stats.total}</div>
				<p class="text-sm text-muted-foreground">Total Categories</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold text-green-600">{stats.active}</div>
				<p class="text-sm text-muted-foreground">Active</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold text-orange-600">{stats.inactive}</div>
				<p class="text-sm text-muted-foreground">Inactive</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold">{stats.rootCount}</div>
				<p class="text-sm text-muted-foreground">Root Categories</p>
			</CardContent>
		</Card>
	</div>

	<!-- Main Content -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<!-- Category Tree -->
		<div class="lg:col-span-2">
			<div class="mb-4 flex items-center gap-2">
				<Checkbox id="showInactive" bind:checked={showInactive} />
				<Label for="showInactive" class="font-normal">Show inactive categories</Label>
			</div>

			{#if categoryState.error}
				<Card class="border-destructive">
					<CardContent class="pt-6">
						<p class="text-destructive">{categoryState.error}</p>
						<Button variant="outline" size="sm" class="mt-2" onclick={loadCategories}>Retry</Button>
					</CardContent>
				</Card>
			{:else}
				<CategoryTree
					{categories}
					isLoading={categoryState.isLoading}
					selectedId={selectedCategory?.categoryId ?? null}
					{showInactive}
					onSelect={handleSelect}
					onAddRoot={handleAddRoot}
					onAddChild={handleAddChild}
					onEdit={handleEdit}
					onDelete={handleDeleteClick}
				/>
			{/if}
		</div>

		<!-- Category Details Panel -->
		<div>
			<Card class="sticky top-6">
				<CardHeader>
					<CardTitle class="text-sm font-medium">Category Details</CardTitle>
				</CardHeader>
				<CardContent>
					{#if selectedCategory}
						<div class="space-y-4">
							<!-- Header -->
							<div class="flex items-start justify-between">
								<div class="flex items-center gap-2">
									<Folder class="h-6 w-6 text-muted-foreground" />
									<div>
										<h3 class="font-semibold">{selectedCategory.name}</h3>
										{#if selectedCategory.code}
											<p class="text-sm text-muted-foreground">{selectedCategory.code}</p>
										{/if}
									</div>
								</div>
								<Badge variant={selectedCategory.isActive ? 'default' : 'secondary'}>
									{selectedCategory.isActive ? 'Active' : 'Inactive'}
								</Badge>
							</div>

							<!-- Description -->
							{#if selectedCategory.description}
								<p class="text-sm text-muted-foreground">{selectedCategory.description}</p>
							{/if}

							<!-- Stats -->
							<div class="grid grid-cols-2 gap-4 rounded-lg bg-muted/50 p-4">
								<div>
									<p class="text-2xl font-bold">{selectedCategory.productCount}</p>
									<p class="text-xs text-muted-foreground">Direct Products</p>
								</div>
								<div>
									<p class="text-2xl font-bold">{selectedCategory.totalProductCount}</p>
									<p class="text-xs text-muted-foreground">Total Products</p>
								</div>
							</div>

							<!-- Breadcrumbs -->
							{#if selectedCategory.breadcrumbs && selectedCategory.breadcrumbs.length > 0}
								<div>
									<p class="mb-1 text-xs text-muted-foreground">Path</p>
									<div class="flex flex-wrap items-center gap-1 text-sm">
										{#each selectedCategory.breadcrumbs as crumb, i (crumb.categoryId)}
											{#if i > 0}
												<span class="text-muted-foreground">/</span>
											{/if}
											<span>{crumb.name}</span>
										{/each}
									</div>
								</div>
							{/if}

							<!-- Meta Info -->
							<div class="space-y-2 text-sm">
								<div class="flex justify-between">
									<span class="text-muted-foreground">Level</span>
									<span>{selectedCategory.level}</span>
								</div>
								<div class="flex justify-between">
									<span class="text-muted-foreground">Display Order</span>
									<span>{selectedCategory.displayOrder}</span>
								</div>
								<div class="flex justify-between">
									<span class="text-muted-foreground">Visible in Store</span>
									<span>{selectedCategory.isVisible ? 'Yes' : 'No'}</span>
								</div>
							</div>

							<!-- Actions -->
							<div class="flex gap-2 pt-2">
								<Button
									variant="outline"
									size="sm"
									class="flex-1"
									onclick={() => handleEdit(selectedCategory!)}
								>
									Edit
								</Button>
								<Button
									variant="outline"
									size="sm"
									class="flex-1"
									onclick={() => handleAddChild(selectedCategory!.categoryId)}
								>
									Add Child
								</Button>
								<Button
									variant="outline"
									size="sm"
									onclick={() =>
										goto(`/inventory/products?category=${selectedCategory!.categoryId}`)}
								>
									View Products
								</Button>
							</div>
						</div>
					{:else}
						<div class="py-8 text-center text-muted-foreground">
							<p>Select a category to view details</p>
						</div>
					{/if}
				</CardContent>
			</Card>
		</div>
	</div>
</div>

<!-- Category Form Modal -->
<CategoryForm
	open={isFormOpen}
	category={editingCategory}
	parentId={parentIdForNew}
	categories={categoryState.items}
	{isSubmitting}
	onClose={handleFormClose}
	onSubmit={handleFormSubmit}
/>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Delete Category</Dialog.Title>
			<Dialog.Description>
				{#if categoryToDelete}
					Are you sure you want to delete "{categoryToDelete.name}"?
					{#if categoryToDelete.productCount > 0}
						<p class="mt-2 font-medium text-destructive">
							Warning: This category has {categoryToDelete.productCount} products. Products will need
							to be reassigned to another category.
						</p>
					{/if}
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" disabled={isDeleting} onclick={() => (deleteDialogOpen = false)}>
				Cancel
			</Button>
			<Button variant="destructive" onclick={handleDelete} disabled={isDeleting}>
				{isDeleting ? 'Deleting...' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
