<script lang="ts">
	import { untrack } from 'svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import CategoryTreeNode from './CategoryTreeNode.svelte';
	import type { CategoryResponse } from '$lib/types/inventory';

	interface Props {
		categories: CategoryResponse[];
		isLoading?: boolean;
		selectedId?: string | null;
		showInactive?: boolean;
		onSelect?: (category: CategoryResponse) => void;
		onAddRoot?: () => void;
		onAddChild?: (parentId: string) => void;
		onEdit?: (category: CategoryResponse) => void;
		onDelete?: (category: CategoryResponse) => void;
	}

	let {
		categories = [],
		isLoading = false,
		selectedId = null,
		showInactive = false,
		onSelect,
		onAddRoot,
		onAddChild,
		onEdit,
		onDelete
	}: Props = $props();

	let searchQuery = $state('');
	let expandedIds = $state<Set<string>>(new Set());

	// Build tree structure from flat list
	const categoryTree = $derived.by(() => {
		const map = new Map<string, CategoryResponse & { children: CategoryResponse[] }>();
		const roots: (CategoryResponse & { children: CategoryResponse[] })[] = [];

		// First pass: create nodes with children arrays
		for (const cat of categories) {
			map.set(cat.categoryId, { ...cat, children: [] });
		}

		// Second pass: build tree structure
		for (const cat of categories) {
			const node = map.get(cat.categoryId)!;
			if (cat.parentCategoryId && map.has(cat.parentCategoryId)) {
				map.get(cat.parentCategoryId)!.children.push(node);
			} else {
				roots.push(node);
			}
		}

		// Sort by displayOrder
		const sortByOrder = (a: CategoryResponse, b: CategoryResponse) =>
			a.displayOrder - b.displayOrder;
		roots.sort(sortByOrder);
		for (const node of map.values()) {
			node.children.sort(sortByOrder);
		}

		return { roots, map };
	});

	// Matching IDs for search (computed separately to avoid state mutation in derived)
	const matchingIds = $derived.by(() => {
		if (!searchQuery.trim()) return new Set<string>();

		const query = searchQuery.toLowerCase();
		const ids = new Set<string>();

		// Find all matching categories and their ancestors
		for (const cat of categories) {
			if (cat.name.toLowerCase().includes(query) || cat.code?.toLowerCase().includes(query)) {
				ids.add(cat.categoryId);
				// Add all ancestors
				let current = cat;
				while (current.parentCategoryId) {
					ids.add(current.parentCategoryId);
					const parent = categoryTree.map.get(current.parentCategoryId);
					if (!parent) break;
					current = parent;
				}
			}
		}

		return ids;
	});

	// Auto-expand matching paths when search changes
	$effect(() => {
		if (matchingIds.size > 0) {
			// Use untrack to avoid infinite loop - we only want to react to matchingIds changes
			const currentExpanded = untrack(() => expandedIds);
			expandedIds = new Set([...currentExpanded, ...matchingIds]);
		}
	});

	// Filter categories based on search
	const filteredRoots = $derived.by(() => {
		if (!searchQuery.trim()) return categoryTree.roots;

		return categoryTree.roots.filter(
			(root) => matchingIds.has(root.categoryId) || hasMatchingDescendant(root, matchingIds)
		);
	});

	function hasMatchingDescendant(
		node: CategoryResponse & { children: CategoryResponse[] },
		matchingIds: Set<string>
	): boolean {
		for (const child of node.children) {
			if (matchingIds.has(child.categoryId)) return true;
			const childWithChildren = categoryTree.map.get(child.categoryId);
			if (
				childWithChildren &&
				hasMatchingDescendant(
					childWithChildren as CategoryResponse & { children: CategoryResponse[] },
					matchingIds
				)
			) {
				return true;
			}
		}
		return false;
	}

	function handleToggle(categoryId: string) {
		const newSet = new Set(expandedIds);
		if (newSet.has(categoryId)) {
			newSet.delete(categoryId);
		} else {
			newSet.add(categoryId);
		}
		expandedIds = newSet;
	}

	function expandAll() {
		expandedIds = new Set(categories.map((c) => c.categoryId));
	}

	function collapseAll() {
		expandedIds = new Set();
	}

	function getChildren(categoryId: string): CategoryResponse[] {
		return categoryTree.map.get(categoryId)?.children ?? [];
	}
</script>

<Card class="h-full">
	<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-4">
		<CardTitle class="text-sm font-medium">Category Tree</CardTitle>
		<div class="flex items-center gap-2">
			<Button variant="ghost" size="sm" onclick={expandAll}>Expand All</Button>
			<Button variant="ghost" size="sm" onclick={collapseAll}>Collapse All</Button>
		</div>
	</CardHeader>
	<CardContent class="space-y-4">
		<!-- Search -->
		<Input type="search" placeholder="Search categories..." bind:value={searchQuery} />

		<!-- Tree -->
		{#if isLoading}
			<div class="space-y-2">
				{#each Array(5) as _}
					<div class="flex animate-pulse items-center gap-2 py-2">
						<div class="h-4 w-4 rounded bg-muted"></div>
						<div class="h-4 w-32 rounded bg-muted"></div>
					</div>
				{/each}
			</div>
		{:else if filteredRoots.length === 0}
			<div class="py-8 text-center text-muted-foreground">
				{#if searchQuery}
					<p>No categories match "{searchQuery}"</p>
				{:else}
					<p>No categories found</p>
					<p class="text-sm">Create your first category to get started</p>
				{/if}
			</div>
		{:else}
			<div class="max-h-[500px] overflow-y-auto">
				{#each filteredRoots as root (root.categoryId)}
					<CategoryTreeNode
						category={root}
						children={getChildren(root.categoryId)}
						{selectedId}
						{expandedIds}
						level={0}
						{showInactive}
						{onSelect}
						onToggle={handleToggle}
						{onAddChild}
						{onEdit}
						{onDelete}
					/>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
