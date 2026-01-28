<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { ChevronRight, Folder, FolderOpen, Plus, Pencil, Trash2 } from 'lucide-svelte';
	import CategoryTreeNode from './CategoryTreeNode.svelte';
	import type { CategoryResponse } from '$lib/types/inventory';

	interface Props {
		category: CategoryResponse;
		children?: CategoryResponse[];
		selectedId?: string | null;
		expandedIds?: Set<string>;
		level?: number;
		showInactive?: boolean;
		onSelect?: (category: CategoryResponse) => void;
		onToggle?: (categoryId: string) => void;
		onAddChild?: (parentId: string) => void;
		onEdit?: (category: CategoryResponse) => void;
		onDelete?: (category: CategoryResponse) => void;
	}

	let {
		category,
		children = [],
		selectedId = null,
		expandedIds = new Set(),
		level = 0,
		showInactive = false,
		onSelect,
		onToggle,
		onAddChild,
		onEdit,
		onDelete
	}: Props = $props();

	const isExpanded = $derived(expandedIds.has(category.categoryId));
	const isSelected = $derived(selectedId === category.categoryId);
	const hasChildren = $derived(children.length > 0);
	const indent = $derived(level * 24);

	function handleToggle(e: MouseEvent | KeyboardEvent) {
		e.stopPropagation();
		onToggle?.(category.categoryId);
	}

	function handleSelect() {
		onSelect?.(category);
	}

	function handleAddChild(e: MouseEvent) {
		e.stopPropagation();
		onAddChild?.(category.categoryId);
	}

	function handleEdit(e: MouseEvent) {
		e.stopPropagation();
		onEdit?.(category);
	}

	function handleDelete(e: MouseEvent) {
		e.stopPropagation();
		onDelete?.(category);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			handleToggle(e);
		}
	}
</script>

<div class="category-node">
	<!-- Row container using div instead of nested buttons -->
	<div
		role="button"
		tabindex="0"
		class="group flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-muted/50"
		class:bg-accent={isSelected}
		class:text-accent-foreground={isSelected}
		class:opacity-50={!category.isActive && !showInactive}
		style:padding-left="{indent + 8}px"
		onclick={handleSelect}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				handleSelect();
			}
		}}
	>
		<!-- Expand/Collapse Toggle -->
		{#if hasChildren}
			<span
				role="button"
				tabindex="0"
				class="flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-muted"
				onclick={handleToggle}
				onkeydown={handleKeyDown}
			>
				<ChevronRight
					class="h-4 w-4 text-muted-foreground transition-transform duration-200 {isExpanded
						? 'rotate-90'
						: ''}"
				/>
			</span>
		{:else}
			<span class="w-5"></span>
		{/if}

		<!-- Category Icon -->
		{#if isExpanded && hasChildren}
			<FolderOpen class="h-4 w-4 text-muted-foreground" />
		{:else}
			<Folder class="h-4 w-4 text-muted-foreground" />
		{/if}

		<!-- Category Name -->
		<span class="flex-1 truncate font-medium">{category.name}</span>

		<!-- Product Count -->
		<span class="text-xs text-muted-foreground">
			({category.productCount})
		</span>

		<!-- Status Badge -->
		{#if !category.isActive}
			<Badge variant="secondary" class="text-xs">Inactive</Badge>
		{/if}

		<!-- Action Buttons (visible on hover) -->
		<div class="invisible flex items-center gap-1 group-hover:visible">
			<Button
				variant="ghost"
				size="sm"
				class="h-6 w-6 p-0"
				onclick={handleAddChild}
				title="Add child category"
			>
				<Plus class="h-3.5 w-3.5" />
			</Button>
			<Button
				variant="ghost"
				size="sm"
				class="h-6 w-6 p-0"
				onclick={handleEdit}
				title="Edit category"
			>
				<Pencil class="h-3.5 w-3.5" />
			</Button>
			<Button
				variant="ghost"
				size="sm"
				class="h-6 w-6 p-0 text-destructive hover:text-destructive"
				onclick={handleDelete}
				title="Delete category"
			>
				<Trash2 class="h-3.5 w-3.5" />
			</Button>
		</div>
	</div>

	<!-- Children (recursive using self-import) -->
	{#if isExpanded && hasChildren}
		<div class="children">
			{#each children as child (child.categoryId)}
				<CategoryTreeNode
					category={child}
					children={[]}
					{selectedId}
					{expandedIds}
					level={level + 1}
					{showInactive}
					{onSelect}
					{onToggle}
					{onAddChild}
					{onEdit}
					{onDelete}
				/>
			{/each}
		</div>
	{/if}
</div>
