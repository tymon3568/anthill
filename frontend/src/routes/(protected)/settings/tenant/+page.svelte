<script lang="ts">
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardDescription
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { Badge } from '$lib/components/ui/badge';
	import * as Select from '$lib/components/ui/select';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { Progress } from '$lib/components/ui/progress';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		TenantSettings,
		TenantBilling,
		TenantAnalytics,
		AuditLogEntry
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import BuildingIcon from '@lucide/svelte/icons/building';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import CreditCardIcon from '@lucide/svelte/icons/credit-card';
	import AlertTriangleIcon from '@lucide/svelte/icons/alert-triangle';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import UploadIcon from '@lucide/svelte/icons/upload';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let { data } = $props();

	// Settings sections
	let activeSection = $state('organization');

	// Loading states
	let isLoading = $state(true);
	let isSaving = $state(false);
	let isUploadingLogo = $state(false);

	// Data
	let settings = $state<TenantSettings | null>(null);
	let billing = $state<TenantBilling | null>(null);
	let analytics = $state<TenantAnalytics | null>(null);
	let auditLogs = $state<AuditLogEntry[]>([]);
	let auditLogsPage = $state(1);
	let auditLogsTotal = $state(0);
	let auditLogsTotalPages = $state(1);

	// Form states
	let orgForm = $state({
		name: '',
		contactEmail: '',
		contactPhone: '',
		address: ''
	});

	let brandingForm = $state({
		primaryColor: '#3b82f6',
		secondaryColor: '#64748b',
		accentColor: '#f59e0b',
		logoUrl: ''
	});

	let localizationForm = $state({
		defaultTimezone: 'UTC',
		defaultCurrency: 'USD',
		defaultLanguage: 'en',
		dateFormat: 'MM/DD/YYYY',
		timeFormat: '12h' as '12h' | '24h'
	});

	let securityForm = $state({
		passwordMinLength: 8,
		passwordRequireUppercase: true,
		passwordRequireLowercase: true,
		passwordRequireNumbers: true,
		passwordRequireSpecialChars: false,
		sessionTimeoutMinutes: 60,
		maxLoginAttempts: 5,
		lockoutDurationMinutes: 15,
		mfaRequired: false
	});

	let dataRetentionForm = $state({
		auditLogRetentionDays: 90,
		deletedUserRetentionDays: 30,
		sessionHistoryRetentionDays: 30,
		backupEnabled: true,
		backupFrequency: 'daily' as 'daily' | 'weekly' | 'monthly'
	});

	// Danger Zone states
	let showDeleteDialog = $state(false);
	let deleteConfirmName = $state('');
	let deleteReason = $state('');
	let isDeleting = $state(false);
	let isExporting = $state(false);

	// Logo file input ref
	let logoInput = $state<HTMLInputElement | null>(null);

	// Options lists
	const timezones = [
		{ value: 'UTC', label: 'UTC' },
		{ value: 'America/New_York', label: 'Eastern Time (US)' },
		{ value: 'America/Chicago', label: 'Central Time (US)' },
		{ value: 'America/Denver', label: 'Mountain Time (US)' },
		{ value: 'America/Los_Angeles', label: 'Pacific Time (US)' },
		{ value: 'Europe/London', label: 'London (GMT)' },
		{ value: 'Europe/Paris', label: 'Paris (CET)' },
		{ value: 'Asia/Tokyo', label: 'Tokyo (JST)' },
		{ value: 'Asia/Ho_Chi_Minh', label: 'Ho Chi Minh (ICT)' },
		{ value: 'Australia/Sydney', label: 'Sydney (AEST)' }
	];

	const currencies = [
		{ value: 'USD', label: 'USD - US Dollar' },
		{ value: 'EUR', label: 'EUR - Euro' },
		{ value: 'GBP', label: 'GBP - British Pound' },
		{ value: 'VND', label: 'VND - Vietnamese Dong' },
		{ value: 'JPY', label: 'JPY - Japanese Yen' },
		{ value: 'CNY', label: 'CNY - Chinese Yuan' }
	];

	const languages = [
		{ value: 'en', label: 'English' },
		{ value: 'vi', label: 'Tiếng Việt' },
		{ value: 'es', label: 'Español' },
		{ value: 'fr', label: 'Français' },
		{ value: 'de', label: 'Deutsch' },
		{ value: 'ja', label: '日本語' }
	];

	const dateFormats = [
		{ value: 'MM/DD/YYYY', label: 'MM/DD/YYYY' },
		{ value: 'DD/MM/YYYY', label: 'DD/MM/YYYY' },
		{ value: 'YYYY-MM-DD', label: 'YYYY-MM-DD' }
	];

	const backupFrequencies = [
		{ value: 'daily', label: 'Daily' },
		{ value: 'weekly', label: 'Weekly' },
		{ value: 'monthly', label: 'Monthly' }
	];

	// Load settings on mount
	onMount(async () => {
		if (!data.isOwner) {
			toast.error('Only tenant owners can access this page');
			return;
		}
		await loadSettings();
	});

	async function loadSettings() {
		isLoading = true;
		try {
			const response = await userServiceApi.getTenantSettings();
			if (response.success && response.data) {
				settings = response.data;
				populateForms(response.data);
			}
		} catch (error) {
			console.error('Failed to load tenant settings:', error);
			toast.error('Failed to load tenant settings');
		} finally {
			isLoading = false;
		}
	}

	function populateForms(data: TenantSettings) {
		orgForm = {
			name: data.tenant.name || '',
			contactEmail: '',
			contactPhone: '',
			address: ''
		};

		if (data.branding) {
			brandingForm = {
				primaryColor: data.branding.primaryColor || '#3b82f6',
				secondaryColor: data.branding.secondaryColor || '#64748b',
				accentColor: data.branding.accentColor || '#f59e0b',
				logoUrl: data.branding.logoUrl || ''
			};
		}

		if (data.localization) {
			localizationForm = {
				defaultTimezone: data.localization.defaultTimezone || 'UTC',
				defaultCurrency: data.localization.defaultCurrency || 'USD',
				defaultLanguage: data.localization.defaultLanguage || 'en',
				dateFormat: data.localization.dateFormat || 'MM/DD/YYYY',
				timeFormat: data.localization.timeFormat || '12h'
			};
		}

		if (data.securityPolicy) {
			securityForm = {
				passwordMinLength: data.securityPolicy.passwordMinLength || 8,
				passwordRequireUppercase: data.securityPolicy.passwordRequireUppercase ?? true,
				passwordRequireLowercase: data.securityPolicy.passwordRequireLowercase ?? true,
				passwordRequireNumbers: data.securityPolicy.passwordRequireNumbers ?? true,
				passwordRequireSpecialChars: data.securityPolicy.passwordRequireSpecialChars ?? false,
				sessionTimeoutMinutes: data.securityPolicy.sessionTimeoutMinutes || 60,
				maxLoginAttempts: data.securityPolicy.maxLoginAttempts || 5,
				lockoutDurationMinutes: data.securityPolicy.lockoutDurationMinutes || 15,
				mfaRequired: data.securityPolicy.mfaRequired ?? false
			};
		}

		if (data.dataRetention) {
			dataRetentionForm = {
				auditLogRetentionDays: data.dataRetention.auditLogRetentionDays || 90,
				deletedUserRetentionDays: data.dataRetention.deletedUserRetentionDays || 30,
				sessionHistoryRetentionDays: data.dataRetention.sessionHistoryRetentionDays || 30,
				backupEnabled: data.dataRetention.backupEnabled ?? true,
				backupFrequency: data.dataRetention.backupFrequency || 'daily'
			};
		}
	}

	async function saveOrganization() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateTenant(orgForm);
			if (response.success) {
				toast.success('Organization settings saved');
				if (response.data) {
					settings = response.data;
				}
			} else {
				toast.error(response.error || 'Failed to save organization settings');
			}
		} catch (error) {
			console.error('Failed to save organization:', error);
			toast.error('Failed to save organization settings');
		} finally {
			isSaving = false;
		}
	}

	async function saveBranding() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateBranding({
				primaryColor: brandingForm.primaryColor,
				secondaryColor: brandingForm.secondaryColor,
				accentColor: brandingForm.accentColor
			});
			if (response.success) {
				toast.success('Branding settings saved');
			} else {
				toast.error(response.error || 'Failed to save branding settings');
			}
		} catch (error) {
			console.error('Failed to save branding:', error);
			toast.error('Failed to save branding settings');
		} finally {
			isSaving = false;
		}
	}

	async function handleLogoUpload(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (file.size > 5 * 1024 * 1024) {
			toast.error('Logo file must be less than 5MB');
			return;
		}

		if (!file.type.startsWith('image/')) {
			toast.error('Please select an image file');
			return;
		}

		isUploadingLogo = true;
		try {
			const response = await userServiceApi.uploadLogo(file);
			if (response.success && response.data) {
				brandingForm.logoUrl = response.data.logoUrl;
				toast.success('Logo uploaded successfully');
			} else {
				toast.error(response.error || 'Failed to upload logo');
			}
		} catch (error) {
			console.error('Failed to upload logo:', error);
			toast.error('Failed to upload logo');
		} finally {
			isUploadingLogo = false;
		}
	}

	async function saveLocalization() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateLocalization(localizationForm);
			if (response.success) {
				toast.success('Localization settings saved');
			} else {
				toast.error(response.error || 'Failed to save localization settings');
			}
		} catch (error) {
			console.error('Failed to save localization:', error);
			toast.error('Failed to save localization settings');
		} finally {
			isSaving = false;
		}
	}

	async function saveSecurityPolicy() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateSecurityPolicy(securityForm);
			if (response.success) {
				toast.success('Security policy saved');
			} else {
				toast.error(response.error || 'Failed to save security policy');
			}
		} catch (error) {
			console.error('Failed to save security policy:', error);
			toast.error('Failed to save security policy');
		} finally {
			isSaving = false;
		}
	}

	async function saveDataRetention() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateDataRetention(dataRetentionForm);
			if (response.success) {
				toast.success('Data retention settings saved');
			} else {
				toast.error(response.error || 'Failed to save data retention settings');
			}
		} catch (error) {
			console.error('Failed to save data retention:', error);
			toast.error('Failed to save data retention settings');
		} finally {
			isSaving = false;
		}
	}

	async function loadBilling() {
		try {
			const response = await userServiceApi.getTenantBilling();
			if (response.success && response.data) {
				billing = response.data;
			}
		} catch (error) {
			console.error('Failed to load billing:', error);
			toast.error('Failed to load billing information');
		}
	}

	async function loadAnalytics() {
		try {
			const response = await userServiceApi.getTenantAnalytics();
			if (response.success && response.data) {
				analytics = response.data;
			}
		} catch (error) {
			console.error('Failed to load analytics:', error);
			toast.error('Failed to load analytics');
		}
	}

	async function loadAuditLogs() {
		try {
			const response = await userServiceApi.listAuditLogs({
				page: auditLogsPage,
				perPage: 10
			});
			if (response.success && response.data) {
				auditLogs = response.data.data;
				auditLogsTotal = response.data.total;
				auditLogsTotalPages = response.data.totalPages;
			}
		} catch (error) {
			console.error('Failed to load audit logs:', error);
			toast.error('Failed to load audit logs');
		}
	}

	async function handleExportData() {
		isExporting = true;
		try {
			const response = await userServiceApi.exportTenantData({
				format: 'json',
				includeUsers: true,
				includeAuditLogs: true,
				includeSettings: true
			});
			if (response.success && response.data) {
				window.open(response.data.downloadUrl, '_blank', 'noopener,noreferrer');
				toast.success('Export started - download will begin shortly');
			} else {
				toast.error(response.error || 'Failed to export data');
			}
		} catch (error) {
			console.error('Failed to export data:', error);
			toast.error('Failed to export tenant data');
		} finally {
			isExporting = false;
		}
	}

	async function handleDeleteTenant() {
		if (!settings || deleteConfirmName !== settings.tenant.name) {
			toast.error('Please type the tenant name exactly to confirm');
			return;
		}

		isDeleting = true;
		try {
			const response = await userServiceApi.deleteTenant({
				confirmTenantName: deleteConfirmName,
				reason: deleteReason || undefined
			});
			if (response.success) {
				toast.success('Tenant deletion initiated');
				// Redirect to logout
				window.location.href = '/auth/logout';
			} else {
				toast.error(response.error || 'Failed to delete tenant');
			}
		} catch (error) {
			console.error('Failed to delete tenant:', error);
			toast.error('Failed to delete tenant');
		} finally {
			isDeleting = false;
		}
	}

	function handleDialogClose(open: boolean) {
		if (!open) {
			deleteConfirmName = '';
			deleteReason = '';
			showDeleteDialog = false;
		}
	}

	// Load section-specific data when switching
	$effect(() => {
		if (activeSection === 'billing' && !billing) {
			loadBilling();
		}
		if (activeSection === 'analytics' && !analytics) {
			loadAnalytics();
		}
		if (activeSection === 'audit' && auditLogs.length === 0) {
			loadAuditLogs();
		}
	});

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString();
	}

	function getPlanBadgeClass(plan: string): string {
		switch (plan) {
			case 'enterprise':
				return 'bg-purple-100 text-purple-800';
			case 'professional':
				return 'bg-blue-100 text-blue-800';
			case 'starter':
				return 'bg-green-100 text-green-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}
</script>

<svelte:head>
	<title>Tenant Settings - Anthill</title>
</svelte:head>

{#if !data.isOwner}
	<div class="flex h-[50vh] items-center justify-center">
		<Card class="w-96">
			<CardContent class="pt-6 text-center">
				<ShieldIcon class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
				<h2 class="mb-2 text-xl font-semibold">Access Denied</h2>
				<p class="text-muted-foreground">Only tenant owners can access organization settings.</p>
				<Button href="/settings" variant="outline" class="mt-4">Back to Settings</Button>
			</CardContent>
		</Card>
	</div>
{:else}
	<div class="space-y-6">
		<div>
			<h1 class="text-2xl font-bold">Organization Settings</h1>
			<p class="text-muted-foreground">Manage your organization's settings and configuration</p>
		</div>

		<div class="flex gap-6">
			<!-- Sidebar -->
			<div class="w-56 space-y-1">
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'organization'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'organization')}
				>
					<BuildingIcon class="h-4 w-4" />
					Organization
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'branding'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'branding')}
				>
					<PaletteIcon class="h-4 w-4" />
					Branding
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'localization'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'localization')}
				>
					<GlobeIcon class="h-4 w-4" />
					Localization
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'security'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'security')}
				>
					<ShieldIcon class="h-4 w-4" />
					Security Policy
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'data'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'data')}
				>
					<DatabaseIcon class="h-4 w-4" />
					Data & Backup
				</button>

				<Separator class="my-2" />

				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'billing'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'billing')}
				>
					<CreditCardIcon class="h-4 w-4" />
					Billing
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'analytics'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'analytics')}
				>
					<ActivityIcon class="h-4 w-4" />
					Analytics
				</button>
				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
					'audit'
						? 'bg-muted font-medium'
						: ''}"
					onclick={() => (activeSection = 'audit')}
				>
					<RefreshCwIcon class="h-4 w-4" />
					Audit Log
				</button>

				<Separator class="my-2" />

				<button
					class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm text-red-600 hover:bg-red-50 {activeSection ===
					'danger'
						? 'bg-red-50 font-medium'
						: ''}"
					onclick={() => (activeSection = 'danger')}
				>
					<AlertTriangleIcon class="h-4 w-4" />
					Danger Zone
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 space-y-6">
				{#if isLoading}
					<Card>
						<CardContent class="py-12">
							<div class="flex items-center justify-center">
								<RefreshCwIcon class="h-6 w-6 animate-spin text-muted-foreground" />
								<span class="ml-2 text-muted-foreground">Loading settings...</span>
							</div>
						</CardContent>
					</Card>
				{:else if activeSection === 'organization'}
					<!-- Organization Settings -->
					<Card>
						<CardHeader>
							<CardTitle>Organization Information</CardTitle>
							<CardDescription>Basic information about your organization</CardDescription>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="orgName">Organization Name</Label>
									<Input id="orgName" bind:value={orgForm.name} placeholder="Acme Corp" />
								</div>
								<div class="space-y-2">
									<Label for="slug">Slug</Label>
									<Input id="slug" value={settings?.tenant.slug || ''} disabled class="bg-muted" />
									<p class="text-xs text-muted-foreground">Cannot be changed</p>
								</div>
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="contactEmail">Contact Email</Label>
									<Input
										id="contactEmail"
										type="email"
										bind:value={orgForm.contactEmail}
										placeholder="contact@example.com"
									/>
								</div>
								<div class="space-y-2">
									<Label for="contactPhone">Contact Phone</Label>
									<Input
										id="contactPhone"
										type="tel"
										bind:value={orgForm.contactPhone}
										placeholder="+1 555 123 4567"
									/>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="address">Address</Label>
								<Input
									id="address"
									bind:value={orgForm.address}
									placeholder="123 Main St, City, Country"
								/>
							</div>
						</CardContent>
					</Card>

					<div class="flex justify-end">
						<Button onclick={saveOrganization} disabled={isSaving}>
							{isSaving ? 'Saving...' : 'Save Changes'}
						</Button>
					</div>
				{:else if activeSection === 'branding'}
					<!-- Branding Settings -->
					<Card>
						<CardHeader>
							<CardTitle>Branding & Customization</CardTitle>
							<CardDescription>Customize the look and feel of your workspace</CardDescription>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Logo -->
							<div class="space-y-2">
								<Label>Organization Logo</Label>
								<div class="flex items-center gap-4">
									<div
										class="flex h-20 w-20 items-center justify-center rounded-lg border bg-muted"
									>
										{#if brandingForm.logoUrl}
											<img
												src={brandingForm.logoUrl}
												alt="Logo"
												class="h-full w-full rounded-lg object-contain"
											/>
										{:else}
											<BuildingIcon class="h-10 w-10 text-muted-foreground" />
										{/if}
									</div>
									<div class="space-y-2">
										<input
											bind:this={logoInput}
											type="file"
											accept="image/*"
											class="hidden"
											onchange={handleLogoUpload}
										/>
										<Button
											variant="outline"
											disabled={isUploadingLogo}
											onclick={() => logoInput?.click()}
										>
											<UploadIcon class="mr-2 h-4 w-4" />
											{isUploadingLogo ? 'Uploading...' : 'Upload Logo'}
										</Button>
										<p class="text-xs text-muted-foreground">PNG, JPG or SVG. Max 5MB.</p>
									</div>
								</div>
							</div>

							<Separator />

							<!-- Colors -->
							<div class="space-y-4">
								<h4 class="font-medium">Brand Colors</h4>
								<div class="grid grid-cols-3 gap-4">
									<div class="space-y-2">
										<Label for="primaryColor">Primary Color</Label>
										<div class="flex gap-2">
											<Input
												id="primaryColor"
												type="color"
												bind:value={brandingForm.primaryColor}
												class="h-10 w-14 p-1"
											/>
											<Input
												bind:value={brandingForm.primaryColor}
												placeholder="#3b82f6"
												class="flex-1"
											/>
										</div>
									</div>
									<div class="space-y-2">
										<Label for="secondaryColor">Secondary Color</Label>
										<div class="flex gap-2">
											<Input
												id="secondaryColor"
												type="color"
												bind:value={brandingForm.secondaryColor}
												class="h-10 w-14 p-1"
											/>
											<Input
												bind:value={brandingForm.secondaryColor}
												placeholder="#64748b"
												class="flex-1"
											/>
										</div>
									</div>
									<div class="space-y-2">
										<Label for="accentColor">Accent Color</Label>
										<div class="flex gap-2">
											<Input
												id="accentColor"
												type="color"
												bind:value={brandingForm.accentColor}
												class="h-10 w-14 p-1"
											/>
											<Input
												bind:value={brandingForm.accentColor}
												placeholder="#f59e0b"
												class="flex-1"
											/>
										</div>
									</div>
								</div>
							</div>
						</CardContent>
					</Card>

					<div class="flex justify-end">
						<Button onclick={saveBranding} disabled={isSaving}>
							{isSaving ? 'Saving...' : 'Save Branding'}
						</Button>
					</div>
				{:else if activeSection === 'localization'}
					<!-- Localization Settings -->
					<Card>
						<CardHeader>
							<CardTitle>Localization Settings</CardTitle>
							<CardDescription
								>Configure default regional settings for your organization</CardDescription
							>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="timezone">Default Timezone</Label>
									<Select.Root
										type="single"
										name="timezone"
										value={localizationForm.defaultTimezone}
										onValueChange={(v) => (localizationForm.defaultTimezone = v)}
									>
										<Select.Trigger class="w-full">
											{timezones.find((t) => t.value === localizationForm.defaultTimezone)?.label ||
												'Select timezone'}
										</Select.Trigger>
										<Select.Content>
											{#each timezones as tz (tz.value)}
												<Select.Item value={tz.value}>{tz.label}</Select.Item>
											{/each}
										</Select.Content>
									</Select.Root>
								</div>
								<div class="space-y-2">
									<Label for="currency">Default Currency</Label>
									<Select.Root
										type="single"
										name="currency"
										value={localizationForm.defaultCurrency}
										onValueChange={(v) => (localizationForm.defaultCurrency = v)}
									>
										<Select.Trigger class="w-full">
											{currencies.find((c) => c.value === localizationForm.defaultCurrency)
												?.label || 'Select currency'}
										</Select.Trigger>
										<Select.Content>
											{#each currencies as curr (curr.value)}
												<Select.Item value={curr.value}>{curr.label}</Select.Item>
											{/each}
										</Select.Content>
									</Select.Root>
								</div>
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="language">Default Language</Label>
									<Select.Root
										type="single"
										name="language"
										value={localizationForm.defaultLanguage}
										onValueChange={(v) => (localizationForm.defaultLanguage = v)}
									>
										<Select.Trigger class="w-full">
											{languages.find((l) => l.value === localizationForm.defaultLanguage)?.label ||
												'Select language'}
										</Select.Trigger>
										<Select.Content>
											{#each languages as lang (lang.value)}
												<Select.Item value={lang.value}>{lang.label}</Select.Item>
											{/each}
										</Select.Content>
									</Select.Root>
								</div>
								<div class="space-y-2">
									<Label for="dateFormat">Date Format</Label>
									<Select.Root
										type="single"
										name="dateFormat"
										value={localizationForm.dateFormat}
										onValueChange={(v) => (localizationForm.dateFormat = v)}
									>
										<Select.Trigger class="w-full">
											{localizationForm.dateFormat || 'Select date format'}
										</Select.Trigger>
										<Select.Content>
											{#each dateFormats as fmt (fmt.value)}
												<Select.Item value={fmt.value}>{fmt.label}</Select.Item>
											{/each}
										</Select.Content>
									</Select.Root>
								</div>
							</div>
							<div class="space-y-2">
								<Label>Time Format</Label>
								<div class="flex gap-4">
									<label class="flex items-center gap-2">
										<input
											type="radio"
											name="timeFormat"
											value="12h"
											checked={localizationForm.timeFormat === '12h'}
											onchange={() => (localizationForm.timeFormat = '12h')}
										/>
										<span>12-hour (AM/PM)</span>
									</label>
									<label class="flex items-center gap-2">
										<input
											type="radio"
											name="timeFormat"
											value="24h"
											checked={localizationForm.timeFormat === '24h'}
											onchange={() => (localizationForm.timeFormat = '24h')}
										/>
										<span>24-hour</span>
									</label>
								</div>
							</div>
						</CardContent>
					</Card>

					<div class="flex justify-end">
						<Button onclick={saveLocalization} disabled={isSaving}>
							{isSaving ? 'Saving...' : 'Save Localization'}
						</Button>
					</div>
				{:else if activeSection === 'security'}
					<!-- Security Policy Settings -->
					<Card>
						<CardHeader>
							<CardTitle>Security Policy</CardTitle>
							<CardDescription
								>Configure security requirements for your organization</CardDescription
							>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Password Policy -->
							<div class="space-y-4">
								<h4 class="font-medium">Password Policy</h4>
								<div class="grid grid-cols-2 gap-4">
									<div class="space-y-2">
										<Label for="minLength">Minimum Length</Label>
										<Input
											id="minLength"
											type="number"
											min="6"
											max="32"
											bind:value={securityForm.passwordMinLength}
										/>
									</div>
								</div>
								<div class="space-y-3">
									<div class="flex items-center justify-between">
										<div>
											<Label>Require Uppercase</Label>
											<p class="text-sm text-muted-foreground">At least one uppercase letter</p>
										</div>
										<Switch bind:checked={securityForm.passwordRequireUppercase} />
									</div>
									<div class="flex items-center justify-between">
										<div>
											<Label>Require Lowercase</Label>
											<p class="text-sm text-muted-foreground">At least one lowercase letter</p>
										</div>
										<Switch bind:checked={securityForm.passwordRequireLowercase} />
									</div>
									<div class="flex items-center justify-between">
										<div>
											<Label>Require Numbers</Label>
											<p class="text-sm text-muted-foreground">At least one number</p>
										</div>
										<Switch bind:checked={securityForm.passwordRequireNumbers} />
									</div>
									<div class="flex items-center justify-between">
										<div>
											<Label>Require Special Characters</Label>
											<p class="text-sm text-muted-foreground">At least one special character</p>
										</div>
										<Switch bind:checked={securityForm.passwordRequireSpecialChars} />
									</div>
								</div>
							</div>

							<Separator />

							<!-- Session Policy -->
							<div class="space-y-4">
								<h4 class="font-medium">Session Policy</h4>
								<div class="grid grid-cols-2 gap-4">
									<div class="space-y-2">
										<Label for="sessionTimeout">Session Timeout (minutes)</Label>
										<Input
											id="sessionTimeout"
											type="number"
											min="5"
											max="480"
											bind:value={securityForm.sessionTimeoutMinutes}
										/>
									</div>
								</div>
							</div>

							<Separator />

							<!-- Login Policy -->
							<div class="space-y-4">
								<h4 class="font-medium">Login Policy</h4>
								<div class="grid grid-cols-2 gap-4">
									<div class="space-y-2">
										<Label for="maxAttempts">Max Login Attempts</Label>
										<Input
											id="maxAttempts"
											type="number"
											min="3"
											max="10"
											bind:value={securityForm.maxLoginAttempts}
										/>
									</div>
									<div class="space-y-2">
										<Label for="lockoutDuration">Lockout Duration (minutes)</Label>
										<Input
											id="lockoutDuration"
											type="number"
											min="5"
											max="60"
											bind:value={securityForm.lockoutDurationMinutes}
										/>
									</div>
								</div>
								<div class="flex items-center justify-between">
									<div>
										<Label>Require MFA</Label>
										<p class="text-sm text-muted-foreground">
											Require multi-factor authentication for all users
										</p>
									</div>
									<Switch bind:checked={securityForm.mfaRequired} />
								</div>
							</div>
						</CardContent>
					</Card>

					<div class="flex justify-end">
						<Button onclick={saveSecurityPolicy} disabled={isSaving}>
							{isSaving ? 'Saving...' : 'Save Security Policy'}
						</Button>
					</div>
				{:else if activeSection === 'data'}
					<!-- Data Retention Settings -->
					<Card>
						<CardHeader>
							<CardTitle>Data Retention & Backup</CardTitle>
							<CardDescription
								>Configure data retention policies and backup settings</CardDescription
							>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Retention Periods -->
							<div class="space-y-4">
								<h4 class="font-medium">Retention Periods</h4>
								<div class="grid grid-cols-2 gap-4">
									<div class="space-y-2">
										<Label for="auditRetention">Audit Log Retention (days)</Label>
										<Input
											id="auditRetention"
											type="number"
											min="30"
											max="365"
											bind:value={dataRetentionForm.auditLogRetentionDays}
										/>
									</div>
									<div class="space-y-2">
										<Label for="userRetention">Deleted User Data (days)</Label>
										<Input
											id="userRetention"
											type="number"
											min="7"
											max="90"
											bind:value={dataRetentionForm.deletedUserRetentionDays}
										/>
									</div>
								</div>
								<div class="space-y-2">
									<Label for="sessionRetention">Session History (days)</Label>
									<Input
										id="sessionRetention"
										type="number"
										min="7"
										max="90"
										bind:value={dataRetentionForm.sessionHistoryRetentionDays}
										class="w-1/2"
									/>
								</div>
							</div>

							<Separator />

							<!-- Backup Settings -->
							<div class="space-y-4">
								<h4 class="font-medium">Backup Settings</h4>
								<div class="flex items-center justify-between">
									<div>
										<Label>Enable Automatic Backup</Label>
										<p class="text-sm text-muted-foreground">
											Automatically backup your data at regular intervals
										</p>
									</div>
									<Switch bind:checked={dataRetentionForm.backupEnabled} />
								</div>
								{#if dataRetentionForm.backupEnabled}
									<div class="space-y-2">
										<Label for="backupFreq">Backup Frequency</Label>
										<Select.Root
											type="single"
											name="backupFreq"
											value={dataRetentionForm.backupFrequency}
											onValueChange={(v) =>
												(dataRetentionForm.backupFrequency = v as 'daily' | 'weekly' | 'monthly')}
										>
											<Select.Trigger class="w-48">
												{backupFrequencies.find(
													(f) => f.value === dataRetentionForm.backupFrequency
												)?.label || 'Select frequency'}
											</Select.Trigger>
											<Select.Content>
												{#each backupFrequencies as freq (freq.value)}
													<Select.Item value={freq.value}>{freq.label}</Select.Item>
												{/each}
											</Select.Content>
										</Select.Root>
									</div>
								{/if}
							</div>
						</CardContent>
					</Card>

					<div class="flex justify-end">
						<Button onclick={saveDataRetention} disabled={isSaving}>
							{isSaving ? 'Saving...' : 'Save Data Settings'}
						</Button>
					</div>
				{:else if activeSection === 'billing'}
					<!-- Billing Section -->
					<Card>
						<CardHeader>
							<CardTitle>Subscription & Billing</CardTitle>
							<CardDescription>Manage your subscription and billing information</CardDescription>
						</CardHeader>
						<CardContent class="space-y-6">
							{#if billing}
								<!-- Current Plan -->
								<div class="rounded-lg border p-4">
									<div class="flex items-center justify-between">
										<div>
											<p class="text-sm text-muted-foreground">Current Plan</p>
											<div class="flex items-center gap-2">
												<span class="text-2xl font-bold capitalize">{billing.plan}</span>
												<Badge class={getPlanBadgeClass(billing.plan)}>{billing.plan}</Badge>
											</div>
										</div>
										<Button variant="outline">Upgrade Plan</Button>
									</div>
									{#if billing.currentPeriodEnd}
										<p class="mt-2 text-sm text-muted-foreground">
											Current period ends: {new Date(billing.currentPeriodEnd).toLocaleDateString()}
										</p>
									{/if}
								</div>

								<!-- Usage Stats -->
								{#if billing.usageStats}
									<div class="space-y-4">
										<h4 class="font-medium">Usage</h4>
										<div class="grid grid-cols-3 gap-4">
											<div class="rounded-lg border p-4">
												<p class="text-sm text-muted-foreground">Users</p>
												<p class="text-2xl font-bold">
													{billing.usageStats.usersCount}/{billing.usageStats.usersLimit}
												</p>
												<Progress
													value={(billing.usageStats.usersCount / billing.usageStats.usersLimit) *
														100}
													class="mt-2 h-2"
												/>
											</div>
											<div class="rounded-lg border p-4">
												<p class="text-sm text-muted-foreground">Storage</p>
												<p class="text-2xl font-bold">
													{Math.round(billing.usageStats.storageUsedMb)}/{billing.usageStats
														.storageLimitMb}
													MB
												</p>
												<Progress
													value={(billing.usageStats.storageUsedMb /
														billing.usageStats.storageLimitMb) *
														100}
													class="mt-2 h-2"
												/>
											</div>
											<div class="rounded-lg border p-4">
												<p class="text-sm text-muted-foreground">API Calls</p>
												<p class="text-2xl font-bold">
													{billing.usageStats.apiCallsCount.toLocaleString()}/{billing.usageStats.apiCallsLimit.toLocaleString()}
												</p>
												<Progress
													value={(billing.usageStats.apiCallsCount /
														billing.usageStats.apiCallsLimit) *
														100}
													class="mt-2 h-2"
												/>
											</div>
										</div>
									</div>
								{/if}
							{:else}
								<div class="py-8 text-center text-muted-foreground">
									<RefreshCwIcon class="mx-auto mb-2 h-6 w-6 animate-spin" />
									Loading billing information...
								</div>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'analytics'}
					<!-- Analytics Section -->
					<Card>
						<CardHeader>
							<CardTitle>Usage Analytics</CardTitle>
							<CardDescription>View usage statistics for your organization</CardDescription>
						</CardHeader>
						<CardContent>
							{#if analytics}
								<div class="grid grid-cols-4 gap-4">
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Active Users (30d)</p>
										<p class="text-3xl font-bold">{analytics.activeUsersLast30Days}</p>
									</div>
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Total Users</p>
										<p class="text-3xl font-bold">{analytics.totalUsers}</p>
									</div>
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Total Orders</p>
										<p class="text-3xl font-bold">{analytics.totalOrders.toLocaleString()}</p>
									</div>
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Total Products</p>
										<p class="text-3xl font-bold">{analytics.totalProducts.toLocaleString()}</p>
									</div>
								</div>

								{#if analytics.topActiveUsers.length > 0}
									<div class="mt-6">
										<h4 class="mb-4 font-medium">Top Active Users</h4>
										<Table.Root>
											<Table.Header>
												<Table.Row>
													<Table.Head>User</Table.Head>
													<Table.Head class="text-right">Actions</Table.Head>
												</Table.Row>
											</Table.Header>
											<Table.Body>
												{#each analytics.topActiveUsers as user (user.userId)}
													<Table.Row>
														<Table.Cell>{user.email}</Table.Cell>
														<Table.Cell class="text-right"
															>{user.actionsCount.toLocaleString()}</Table.Cell
														>
													</Table.Row>
												{/each}
											</Table.Body>
										</Table.Root>
									</div>
								{/if}
							{:else}
								<div class="py-8 text-center text-muted-foreground">
									<RefreshCwIcon class="mx-auto mb-2 h-6 w-6 animate-spin" />
									Loading analytics...
								</div>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'audit'}
					<!-- Audit Log Section -->
					<Card>
						<CardHeader>
							<CardTitle>Audit Log</CardTitle>
							<CardDescription>View activity history for your organization</CardDescription>
						</CardHeader>
						<CardContent>
							{#if auditLogs.length > 0}
								<Table.Root>
									<Table.Header>
										<Table.Row>
											<Table.Head>Time</Table.Head>
											<Table.Head>User</Table.Head>
											<Table.Head>Action</Table.Head>
											<Table.Head>Resource</Table.Head>
											<Table.Head>IP Address</Table.Head>
										</Table.Row>
									</Table.Header>
									<Table.Body>
										{#each auditLogs as log (log.id)}
											<Table.Row>
												<Table.Cell class="text-sm">{formatDate(log.createdAt)}</Table.Cell>
												<Table.Cell>{log.userEmail}</Table.Cell>
												<Table.Cell>
													<Badge variant="outline">{log.action}</Badge>
												</Table.Cell>
												<Table.Cell>{log.resource}</Table.Cell>
												<Table.Cell class="text-muted-foreground">{log.ipAddress || '-'}</Table.Cell
												>
											</Table.Row>
										{/each}
									</Table.Body>
								</Table.Root>

								<!-- Pagination -->
								<div class="mt-4 flex items-center justify-between">
									<p class="text-sm text-muted-foreground">
										Showing page {auditLogsPage} of {auditLogsTotalPages} ({auditLogsTotal} total)
									</p>
									<div class="flex gap-2">
										<Button
											variant="outline"
											size="sm"
											disabled={auditLogsPage <= 1}
											onclick={() => {
												auditLogsPage--;
												loadAuditLogs();
											}}
										>
											<ChevronLeftIcon class="h-4 w-4" />
										</Button>
										<Button
											variant="outline"
											size="sm"
											disabled={auditLogsPage >= auditLogsTotalPages}
											onclick={() => {
												auditLogsPage++;
												loadAuditLogs();
											}}
										>
											<ChevronRightIcon class="h-4 w-4" />
										</Button>
									</div>
								</div>
							{:else}
								<div class="py-8 text-center text-muted-foreground">
									<RefreshCwIcon class="mx-auto mb-2 h-6 w-6 animate-spin" />
									Loading audit logs...
								</div>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'danger'}
					<!-- Danger Zone -->
					<Card class="border-red-200">
						<CardHeader>
							<CardTitle class="text-red-600">Danger Zone</CardTitle>
							<CardDescription>
								Irreversible actions that affect your entire organization
							</CardDescription>
						</CardHeader>
						<CardContent class="space-y-6">
							<!-- Export Data -->
							<div
								class="flex items-center justify-between rounded-lg border border-amber-200 bg-amber-50 p-4"
							>
								<div>
									<h4 class="font-medium">Export Tenant Data</h4>
									<p class="text-sm text-muted-foreground">
										Download all your organization's data in JSON format
									</p>
								</div>
								<Button variant="outline" onclick={handleExportData} disabled={isExporting}>
									<DownloadIcon class="mr-2 h-4 w-4" />
									{isExporting ? 'Exporting...' : 'Export Data'}
								</Button>
							</div>

							<!-- Delete Tenant -->
							<div
								class="flex items-center justify-between rounded-lg border border-red-200 bg-red-50 p-4"
							>
								<div>
									<h4 class="font-medium text-red-600">Delete Organization</h4>
									<p class="text-sm text-muted-foreground">
										Permanently delete your organization and all associated data. This action cannot
										be undone.
									</p>
								</div>
								<Button variant="destructive" onclick={() => (showDeleteDialog = true)}>
									<TrashIcon class="mr-2 h-4 w-4" />
									Delete Organization
								</Button>
							</div>
						</CardContent>
					</Card>
				{/if}
			</div>
		</div>
	</div>

	<!-- Delete Confirmation Dialog -->
	<Dialog.Root open={showDeleteDialog} onOpenChange={handleDialogClose}>
		<Dialog.Content class="max-w-md">
			<Dialog.Header>
				<Dialog.Title class="text-red-600">Delete Organization</Dialog.Title>
				<Dialog.Description>
					This action is permanent and cannot be undone. All data including users, orders, products,
					and settings will be permanently deleted.
				</Dialog.Description>
			</Dialog.Header>

			<div class="space-y-4 py-4">
				<div class="rounded-lg border border-red-200 bg-red-50 p-3">
					<p class="text-sm font-medium text-red-800">
						To confirm, type "{settings?.tenant.name}" below:
					</p>
				</div>
				<div class="space-y-2">
					<Label for="confirmName">Organization Name</Label>
					<Input
						id="confirmName"
						bind:value={deleteConfirmName}
						placeholder={settings?.tenant.name}
					/>
				</div>
				<div class="space-y-2">
					<Label for="deleteReason">Reason (optional)</Label>
					<Input id="deleteReason" bind:value={deleteReason} placeholder="Why are you leaving?" />
				</div>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={() => (showDeleteDialog = false)}>Cancel</Button>
				<Button
					variant="destructive"
					onclick={handleDeleteTenant}
					disabled={isDeleting || deleteConfirmName !== settings?.tenant.name}
				>
					{isDeleting ? 'Deleting...' : 'Delete Forever'}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
