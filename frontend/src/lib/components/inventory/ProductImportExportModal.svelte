<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Badge } from '$lib/components/ui/badge';
	import { productImportApi } from '$lib/api/inventory/product-import';
	import type {
		ImportStep,
		ImportValidationResult,
		ImportResult,
		ImportRowError
	} from '$lib/types/product-import';
	import {
		Upload,
		Download,
		FileSpreadsheet,
		CheckCircle,
		AlertCircle,
		Loader2,
		ArrowRight,
		ArrowLeft,
		X
	} from 'lucide-svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
		onImportComplete?: () => void;
	}

	let { open = $bindable(), onClose, onImportComplete }: Props = $props();

	// Wizard state
	let step = $state<ImportStep>('upload');
	let file = $state<File | null>(null);
	let validationResult = $state<ImportValidationResult | null>(null);
	let importResult = $state<ImportResult | null>(null);
	let upsertMode = $state(false);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Reset state when dialog opens/closes
	$effect(() => {
		if (open) {
			resetState();
		}
	});

	function resetState() {
		step = 'upload';
		file = null;
		validationResult = null;
		importResult = null;
		upsertMode = false;
		isLoading = false;
		error = null;
	}

	async function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			file = input.files[0];
			error = null;
		}
	}

	async function handleDrop(event: DragEvent) {
		event.preventDefault();
		if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
			const droppedFile = event.dataTransfer.files[0];
			if (droppedFile.type === 'text/csv' || droppedFile.name.endsWith('.csv')) {
				file = droppedFile;
				error = null;
			} else {
				error = 'Please upload a CSV file';
			}
		}
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
	}

	async function downloadTemplate() {
		try {
			isLoading = true;
			const blob = await productImportApi.getTemplate();
			productImportApi.downloadBlob(blob, 'products_import_template.csv');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to download template';
		} finally {
			isLoading = false;
		}
	}

	async function validateFile() {
		if (!file) return;

		try {
			isLoading = true;
			error = null;
			validationResult = await productImportApi.validateCsv(file);
			step = 'validate';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to validate file';
		} finally {
			isLoading = false;
		}
	}

	async function importProducts() {
		if (!file) return;

		try {
			isLoading = true;
			error = null;
			importResult = await productImportApi.importCsv(file, upsertMode);
			step = 'complete';
			onImportComplete?.();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to import products';
		} finally {
			isLoading = false;
		}
	}

	async function exportProducts() {
		try {
			isLoading = true;
			error = null;
			const blob = await productImportApi.exportCsv();
			const date = new Date().toISOString().split('T')[0];
			productImportApi.downloadBlob(blob, `products_export_${date}.csv`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to export products';
		} finally {
			isLoading = false;
		}
	}

	function goBack() {
		if (step === 'validate') {
			step = 'upload';
			validationResult = null;
		}
	}

	function formatErrors(errors: ImportRowError[]): string {
		return errors
			.slice(0, 5)
			.map((e) => `Row ${e.rowNumber}: ${e.field} - ${e.error}`)
			.join('\n');
	}
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => !isOpen && onClose()}>
	<Dialog.Content class="max-w-2xl">
		<Dialog.Header>
			<Dialog.Title class="flex items-center gap-2">
				<FileSpreadsheet class="h-5 w-5" />
				Import/Export Products
			</Dialog.Title>
			<Dialog.Description>
				Import products from CSV or export your product catalog
			</Dialog.Description>
		</Dialog.Header>

		<div class="py-4">
			{#if error}
				<div
					class="mb-4 flex items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3"
				>
					<AlertCircle class="mt-0.5 h-4 w-4 text-destructive" />
					<div class="text-sm text-destructive">{error}</div>
				</div>
			{/if}

			<!-- Step: Upload -->
			{#if step === 'upload'}
				<div class="space-y-6">
					<!-- Export Section -->
					<div class="rounded-lg border p-4">
						<h3 class="mb-2 font-medium">Export Products</h3>
						<p class="mb-4 text-sm text-muted-foreground">
							Download your product catalog as a CSV file
						</p>
						<Button variant="outline" onclick={exportProducts} disabled={isLoading}>
							{#if isLoading}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{:else}
								<Download class="mr-2 h-4 w-4" />
							{/if}
							Export to CSV
						</Button>
					</div>

					<!-- Import Section -->
					<div class="rounded-lg border p-4">
						<h3 class="mb-2 font-medium">Import Products</h3>
						<p class="mb-4 text-sm text-muted-foreground">
							Upload a CSV file to import products. Maximum 1000 rows per import.
						</p>

						<!-- Template Download -->
						<div class="mb-4">
							<Button variant="link" class="h-auto p-0" onclick={downloadTemplate}>
								<Download class="mr-1 h-3 w-3" />
								Download CSV Template
							</Button>
						</div>

						<!-- File Drop Zone -->
						<div
							class="flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-8 transition-colors hover:border-primary"
							ondrop={handleDrop}
							ondragover={handleDragOver}
							role="button"
							tabindex="0"
						>
							{#if file}
								<div class="flex items-center gap-2">
									<FileSpreadsheet class="h-8 w-8 text-green-600" />
									<div>
										<p class="font-medium">{file.name}</p>
										<p class="text-sm text-muted-foreground">
											{(file.size / 1024).toFixed(1)} KB
										</p>
									</div>
									<Button
										variant="ghost"
										size="sm"
										onclick={() => {
											file = null;
										}}
									>
										<X class="h-4 w-4" />
									</Button>
								</div>
							{:else}
								<Upload class="mb-2 h-8 w-8 text-muted-foreground" />
								<p class="mb-1 text-sm font-medium">Drop CSV file here or click to browse</p>
								<p class="text-xs text-muted-foreground">Supports .csv files only</p>
							{/if}

							<input
								type="file"
								accept=".csv,text/csv"
								class="absolute inset-0 cursor-pointer opacity-0"
								onchange={handleFileSelect}
							/>
						</div>

						<!-- Upsert Mode Toggle -->
						<div class="mt-4 flex items-center gap-2">
							<Checkbox id="upsert" bind:checked={upsertMode} />
							<Label for="upsert" class="text-sm">
								Update existing products if SKU already exists
							</Label>
						</div>
					</div>
				</div>
			{/if}

			<!-- Step: Validate -->
			{#if step === 'validate' && validationResult}
				<div class="space-y-4">
					<div class="flex items-center gap-4">
						{#if validationResult.isValid}
							<div class="flex items-center gap-2 text-green-600">
								<CheckCircle class="h-5 w-5" />
								<span class="font-medium">Validation Passed</span>
							</div>
						{:else}
							<div class="flex items-center gap-2 text-red-600">
								<AlertCircle class="h-5 w-5" />
								<span class="font-medium">Validation Failed</span>
							</div>
						{/if}
					</div>

					<div class="grid grid-cols-3 gap-4">
						<div class="rounded-lg bg-muted p-3 text-center">
							<p class="text-2xl font-bold">{validationResult.totalRows}</p>
							<p class="text-xs text-muted-foreground">Total Rows</p>
						</div>
						<div class="rounded-lg bg-green-100 p-3 text-center dark:bg-green-900/20">
							<p class="text-2xl font-bold text-green-600">{validationResult.validRows}</p>
							<p class="text-xs text-muted-foreground">Valid Rows</p>
						</div>
						<div class="rounded-lg bg-red-100 p-3 text-center dark:bg-red-900/20">
							<p class="text-2xl font-bold text-red-600">{validationResult.errors.length}</p>
							<p class="text-xs text-muted-foreground">Errors</p>
						</div>
					</div>

					{#if validationResult.errors.length > 0}
						<div class="max-h-48 overflow-auto rounded-lg border p-3">
							<p class="mb-2 text-sm font-medium">Errors (first 5):</p>
							<div class="space-y-1 text-sm">
								{#each validationResult.errors.slice(0, 5) as err}
									<div class="flex gap-2">
										<Badge variant="outline">Row {err.rowNumber}</Badge>
										<span class="text-muted-foreground">{err.field}:</span>
										<span>{err.error}</span>
									</div>
								{/each}
								{#if validationResult.errors.length > 5}
									<p class="text-muted-foreground">
										... and {validationResult.errors.length - 5} more errors
									</p>
								{/if}
							</div>
						</div>
					{/if}

					{#if validationResult.preview && validationResult.preview.length > 0}
						<div class="max-h-48 overflow-auto rounded-lg border p-3">
							<p class="mb-2 text-sm font-medium">Preview (first 5 rows):</p>
							<div class="space-y-1 text-sm">
								{#each validationResult.preview as row, i}
									<div class="rounded bg-muted p-2">
										<span class="font-mono">{row.sku}</span> - {row.name}
										{#if row.salePrice}
											<Badge variant="secondary" class="ml-2">
												{row.salePrice.toLocaleString()}
												{row.currency || 'VND'}
											</Badge>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/if}

			<!-- Step: Complete -->
			{#if step === 'complete' && importResult}
				<div class="space-y-4 text-center">
					<CheckCircle class="mx-auto h-16 w-16 text-green-600" />
					<h3 class="text-xl font-semibold">Import Complete</h3>

					<div class="grid grid-cols-3 gap-4">
						<div class="rounded-lg bg-green-100 p-3 text-center dark:bg-green-900/20">
							<p class="text-2xl font-bold text-green-600">{importResult.created}</p>
							<p class="text-xs text-muted-foreground">Created</p>
						</div>
						<div class="rounded-lg bg-blue-100 p-3 text-center dark:bg-blue-900/20">
							<p class="text-2xl font-bold text-blue-600">{importResult.updated}</p>
							<p class="text-xs text-muted-foreground">Updated</p>
						</div>
						<div class="rounded-lg bg-red-100 p-3 text-center dark:bg-red-900/20">
							<p class="text-2xl font-bold text-red-600">{importResult.failed}</p>
							<p class="text-xs text-muted-foreground">Failed</p>
						</div>
					</div>

					{#if importResult.errors.length > 0}
						<div class="max-h-32 overflow-auto rounded-lg border p-3 text-left">
							<p class="mb-2 text-sm font-medium">Failed rows:</p>
							<div class="space-y-1 text-sm">
								{#each importResult.errors.slice(0, 5) as err}
									<div class="flex gap-2">
										<Badge variant="outline">Row {err.rowNumber}</Badge>
										<span>{err.error}</span>
									</div>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			{#if step === 'upload'}
				<Button variant="outline" onclick={onClose}>Cancel</Button>
				<Button onclick={validateFile} disabled={!file || isLoading}>
					{#if isLoading}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Validate
					<ArrowRight class="ml-2 h-4 w-4" />
				</Button>
			{:else if step === 'validate'}
				<Button variant="outline" onclick={goBack}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back
				</Button>
				<Button onclick={importProducts} disabled={!validationResult?.isValid || isLoading}>
					{#if isLoading}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					{/if}
					Import {validationResult?.validRows} Products
				</Button>
			{:else if step === 'complete'}
				<Button onclick={onClose}>Close</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
