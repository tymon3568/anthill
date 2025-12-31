<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';

	interface Props {
		/** Number of main navigation items to show */
		mainItemCount?: number;
		/** Number of settings items to show */
		settingsItemCount?: number;
		/** Show sub-items under main nav */
		showSubItems?: boolean;
	}

	let { mainItemCount = 4, settingsItemCount = 1, showSubItems = true }: Props = $props();

	// Generate random widths for more natural look
	function getRandomWidth() {
		return `${Math.floor(Math.random() * 30) + 60}%`;
	}
</script>

<!-- Sidebar Header Skeleton -->
<Sidebar.Header>
	<Sidebar.Menu>
		<Sidebar.MenuItem>
			<div class="flex h-12 items-center gap-2 px-2">
				<Skeleton class="size-8 rounded-lg" />
				<div class="flex flex-1 flex-col gap-1">
					<Skeleton class="h-4 w-20" />
					<Skeleton class="h-3 w-28" />
				</div>
			</div>
		</Sidebar.MenuItem>
	</Sidebar.Menu>
</Sidebar.Header>

<Sidebar.Content>
	<!-- Main Navigation Skeleton -->
	<Sidebar.Group>
		<Sidebar.GroupLabel>
			<Skeleton class="h-3 w-16" />
		</Sidebar.GroupLabel>
		<Sidebar.Menu>
			{#each Array(mainItemCount) as _, i (i)}
				<Sidebar.MenuItem>
					<Sidebar.MenuSkeleton showIcon />
				</Sidebar.MenuItem>
				{#if showSubItems && i === 1}
					<!-- Show expanded sub-items for second item -->
					<div class="ml-4 flex flex-col gap-1 py-1">
						{#each Array(3) as _, j (j)}
							<div class="flex h-7 items-center gap-2 px-2">
								<Skeleton class="h-3" style="width: {getRandomWidth()}" />
							</div>
						{/each}
					</div>
				{/if}
			{/each}
		</Sidebar.Menu>
	</Sidebar.Group>

	<!-- Settings Navigation Skeleton -->
	<Sidebar.Group class="mt-auto">
		<Sidebar.GroupLabel>
			<Skeleton class="h-3 w-14" />
		</Sidebar.GroupLabel>
		<Sidebar.Menu>
			{#each Array(settingsItemCount) as _, i (i)}
				<Sidebar.MenuItem>
					<Sidebar.MenuSkeleton showIcon />
				</Sidebar.MenuItem>
			{/each}
		</Sidebar.Menu>
	</Sidebar.Group>
</Sidebar.Content>

<!-- User Profile Skeleton -->
<Sidebar.Footer>
	<Sidebar.Menu>
		<Sidebar.MenuItem>
			<div class="flex h-12 items-center gap-2 px-2">
				<Skeleton class="size-8 rounded-lg" />
				<div class="flex flex-1 flex-col gap-1">
					<Skeleton class="h-4 w-24" />
					<Skeleton class="h-3 w-32" />
				</div>
				<Skeleton class="size-4" />
			</div>
		</Sidebar.MenuItem>
	</Sidebar.Menu>
</Sidebar.Footer>
