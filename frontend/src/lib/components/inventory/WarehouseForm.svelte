<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import * as Select from '$lib/components/ui/select';
	import { Switch } from '$lib/components/ui/switch';
	import type { WarehouseResponse, CreateWarehouseRequest } from '$lib/types/inventory';

	interface Props {
		open: boolean;
		warehouse?: WarehouseResponse | null;
		isSubmitting?: boolean;
		onClose: () => void;
		onSubmit: (data: CreateWarehouseRequest) => void;
	}

	let {
		open = $bindable(),
		warehouse = null,
		isSubmitting = false,
		onClose,
		onSubmit
	}: Props = $props();

	// Form state
	let warehouseCode = $state('');
	let warehouseName = $state('');
	let warehouseType = $state('main');
	let description = $state('');
	let isActive = $state(true);

	// Address fields
	let street = $state('');
	let city = $state('');
	let stateProvince = $state('');
	let country = $state('');
	let postalCode = $state('');

	// Contact fields
	let contactName = $state('');
	let contactPhone = $state('');
	let contactEmail = $state('');

	const warehouseTypes = [
		{ value: 'main', label: 'Main Warehouse' },
		{ value: 'satellite', label: 'Satellite' },
		{ value: 'distribution', label: 'Distribution Center' },
		{ value: 'storage', label: 'Storage Facility' }
	];

	const isEditing = $derived(!!warehouse);
	const dialogTitle = $derived(isEditing ? 'Edit Warehouse' : 'Create Warehouse');

	// Reset form when dialog opens/closes
	$effect(() => {
		if (open) {
			if (warehouse) {
				warehouseCode = warehouse.warehouseCode;
				warehouseName = warehouse.warehouseName;
				warehouseType = warehouse.warehouseType;
				description = warehouse.description ?? '';
				isActive = warehouse.isActive;

				// Parse address
				const addr = warehouse.address as Record<string, string> | undefined;
				street = addr?.street ?? '';
				city = addr?.city ?? '';
				stateProvince = addr?.state ?? '';
				country = addr?.country ?? '';
				postalCode = addr?.postalCode ?? '';

				// Parse contact
				const contact = warehouse.contactInfo as Record<string, string> | undefined;
				contactName = contact?.name ?? '';
				contactPhone = contact?.phone ?? '';
				contactEmail = contact?.email ?? '';
			} else {
				resetForm();
			}
		}
	});

	function resetForm() {
		warehouseCode = '';
		warehouseName = '';
		warehouseType = 'main';
		description = '';
		isActive = true;
		street = '';
		city = '';
		stateProvince = '';
		country = '';
		postalCode = '';
		contactName = '';
		contactPhone = '';
		contactEmail = '';
	}

	function handleSubmit(e: Event) {
		e.preventDefault();

		const data: CreateWarehouseRequest = {
			warehouseCode,
			warehouseName,
			warehouseType,
			description: description || undefined
		};

		// Add address if any field is filled
		if (street || city || stateProvince || country || postalCode) {
			data.address = { street, city, state: stateProvince, country, postalCode };
		}

		// Add contact if any field is filled
		if (contactName || contactPhone || contactEmail) {
			data.contactInfo = { name: contactName, phone: contactPhone, email: contactEmail };
		}

		onSubmit(data);
	}

	function handleClose() {
		resetForm();
		onClose();
	}
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => !isOpen && handleClose()}>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{dialogTitle}</Dialog.Title>
			<Dialog.Description>
				{isEditing ? 'Update warehouse information' : 'Add a new warehouse to manage inventory'}
			</Dialog.Description>
		</Dialog.Header>

		<form onsubmit={handleSubmit} class="space-y-6">
			<!-- Basic Info -->
			<div class="space-y-4">
				<h4 class="text-sm font-medium">Basic Information</h4>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="code">Warehouse Code *</Label>
						<Input
							id="code"
							bind:value={warehouseCode}
							placeholder="WH-001"
							required
							disabled={isEditing}
						/>
					</div>
					<div class="space-y-2">
						<Label for="type">Type *</Label>
						<Select.Root
							type="single"
							value={warehouseType}
							onValueChange={(v) => v && (warehouseType = v)}
						>
							<Select.Trigger id="type" class="w-full">
								{warehouseTypes.find((t) => t.value === warehouseType)?.label ?? 'Select type'}
							</Select.Trigger>
							<Select.Content>
								{#each warehouseTypes as type (type.value)}
									<Select.Item value={type.value}>{type.label}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="name">Warehouse Name *</Label>
					<Input id="name" bind:value={warehouseName} placeholder="Main Warehouse" required />
				</div>

				<div class="space-y-2">
					<Label for="description">Description</Label>
					<Textarea
						id="description"
						bind:value={description}
						placeholder="Description of the warehouse..."
						rows={2}
					/>
				</div>

				{#if isEditing}
					<div class="flex items-center gap-2">
						<Switch id="active" bind:checked={isActive} />
						<Label for="active">Active</Label>
					</div>
				{/if}
			</div>

			<!-- Address -->
			<div class="space-y-4">
				<h4 class="text-sm font-medium">Address</h4>

				<div class="space-y-2">
					<Label for="street">Street</Label>
					<Input id="street" bind:value={street} placeholder="123 Warehouse St" />
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="city">City</Label>
						<Input id="city" bind:value={city} placeholder="City" />
					</div>
					<div class="space-y-2">
						<Label for="state">State/Province</Label>
						<Input id="state" bind:value={stateProvince} placeholder="State" />
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="country">Country</Label>
						<Input id="country" bind:value={country} placeholder="Country" />
					</div>
					<div class="space-y-2">
						<Label for="postal">Postal Code</Label>
						<Input id="postal" bind:value={postalCode} placeholder="12345" />
					</div>
				</div>
			</div>

			<!-- Contact Info -->
			<div class="space-y-4">
				<h4 class="text-sm font-medium">Contact Information</h4>

				<div class="space-y-2">
					<Label for="contactName">Contact Name</Label>
					<Input id="contactName" bind:value={contactName} placeholder="John Doe" />
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="phone">Phone</Label>
						<Input id="phone" bind:value={contactPhone} placeholder="+1 234 567 890" />
					</div>
					<div class="space-y-2">
						<Label for="email">Email</Label>
						<Input
							id="email"
							type="email"
							bind:value={contactEmail}
							placeholder="contact@warehouse.com"
						/>
					</div>
				</div>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={handleClose} disabled={isSubmitting}>
					Cancel
				</Button>
				<Button type="submit" disabled={isSubmitting || !warehouseCode || !warehouseName}>
					{isSubmitting ? 'Saving...' : isEditing ? 'Update' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
