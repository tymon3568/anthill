<script lang="ts">
	import { onMount } from 'svelte';
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
	import { Badge } from '$lib/components/ui/badge';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		PaymentSettings,
		PaymentGateway,
		PaymentGatewayCredentials,
		PaymentMethodSettings,
		CurrencyConfig,
		PaymentRegionConfig,
		PaymentGatewayHealth,
		PaymentAnalytics,
		PaymentProvider
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import CreditCard from '@lucide/svelte/icons/credit-card';
	import Shield from '@lucide/svelte/icons/shield';
	import Globe from '@lucide/svelte/icons/globe';
	import DollarSign from '@lucide/svelte/icons/dollar-sign';
	import Activity from '@lucide/svelte/icons/activity';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Check from '@lucide/svelte/icons/check';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Eye from '@lucide/svelte/icons/eye';
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import TestTube from '@lucide/svelte/icons/test-tube';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import CircleDollarSign from '@lucide/svelte/icons/circle-dollar-sign';
	import MapPin from '@lucide/svelte/icons/map-pin';

	let { data } = $props();

	// Navigation sections
	type Section =
		| 'gateways'
		| 'methods'
		| 'currencies'
		| 'regions'
		| 'fees'
		| 'security'
		| 'analytics'
		| 'health';
	let activeSection = $state<Section>('gateways');

	// Loading states
	let isLoading = $state(true);
	let isSaving = $state(false);
	let isTesting = $state(false);

	// Data
	let settings = $state<PaymentSettings | null>(null);
	let healthData = $state<PaymentGatewayHealth[]>([]);
	let analyticsData = $state<PaymentAnalytics | null>(null);

	// Dialogs
	let showAddGatewayDialog = $state(false);
	let showCredentialsDialog = $state(false);
	let showDeleteGatewayDialog = $state(false);
	let selectedGateway = $state<PaymentGateway | null>(null);
	let gatewayCredentials = $state<PaymentGatewayCredentials | null>(null);
	let showSecretKey = $state(false);

	// Form state for new gateway
	let newGateway = $state({
		provider: 'stripe' as PaymentProvider,
		name: '',
		isSandbox: true,
		isDefault: false,
		publicKey: '',
		secretKey: '',
		merchantId: '',
		webhookSecret: ''
	});

	// Provider options
	const providers: { value: PaymentProvider; label: string }[] = [
		{ value: 'stripe', label: 'Stripe' },
		{ value: 'paypal', label: 'PayPal' },
		{ value: 'square', label: 'Square' },
		{ value: 'braintree', label: 'Braintree' },
		{ value: 'adyen', label: 'Adyen' },
		{ value: 'momo', label: 'MoMo (Vietnam)' },
		{ value: 'vnpay', label: 'VNPay (Vietnam)' },
		{ value: 'zalopay', label: 'ZaloPay (Vietnam)' }
	];

	// Load data on mount
	async function loadSettings() {
		isLoading = true;
		try {
			const response = await userServiceApi.getPaymentSettings();
			if (response.success && response.data) {
				settings = response.data;
			} else {
				settings = {
					gateways: [],
					paymentMethods: [],
					currencies: [],
					regions: [],
					security: {
						require3DSecure: false,
						fraudDetectionEnabled: true,
						fraudRiskThreshold: 'medium',
						velocityChecksEnabled: false,
						blockedCountries: [],
						blockedBinRanges: []
					}
				};
			}
		} catch (error) {
			console.error('Failed to load payment settings:', error);
			toast.error('Failed to load payment settings');
		} finally {
			isLoading = false;
		}
	}

	async function loadHealth() {
		try {
			const response = await userServiceApi.getPaymentGatewayHealth();
			if (response.success && response.data) {
				healthData = response.data;
			}
		} catch (error) {
			console.error('Failed to load health data:', error);
		}
	}

	async function loadAnalytics() {
		try {
			const response = await userServiceApi.getPaymentAnalytics(undefined, 'month');
			if (response.success && response.data) {
				analyticsData = response.data;
			}
		} catch (error) {
			console.error('Failed to load analytics:', error);
		}
	}

	$effect(() => {
		if (activeSection === 'health' && healthData.length === 0) {
			loadHealth();
		}
		if (activeSection === 'analytics' && !analyticsData) {
			loadAnalytics();
		}
	});

	// Load settings on mount only
	onMount(() => {
		loadSettings();
	});

	async function handleAddGateway() {
		isSaving = true;
		try {
			const response = await userServiceApi.upsertPaymentGateway(undefined, newGateway);
			if (response.success && response.data) {
				settings?.gateways.push(response.data);
				settings = settings;
				showAddGatewayDialog = false;
				resetNewGateway();
				toast.success('Payment gateway added successfully');
			} else {
				toast.error(response.error || 'Failed to add gateway');
			}
		} catch (error) {
			console.error('Failed to add gateway:', error);
			toast.error('Failed to add payment gateway');
		} finally {
			isSaving = false;
		}
	}

	async function handleDeleteGateway() {
		if (!selectedGateway) return;
		isSaving = true;
		try {
			const response = await userServiceApi.deletePaymentGateway(selectedGateway.id);
			if (response.success) {
				if (settings) {
					settings.gateways = settings.gateways.filter((g) => g.id !== selectedGateway?.id);
				}
				showDeleteGatewayDialog = false;
				selectedGateway = null;
				toast.success('Payment gateway deleted');
			} else {
				toast.error(response.error || 'Failed to delete gateway');
			}
		} catch (error) {
			console.error('Failed to delete gateway:', error);
			toast.error('Failed to delete payment gateway');
		} finally {
			isSaving = false;
		}
	}

	async function handleTestGateway(gateway: PaymentGateway) {
		isTesting = true;
		try {
			const response = await userServiceApi.testPaymentGateway(gateway.id);
			if (response.success && response.data) {
				if (response.data.success) {
					toast.success(`Connection test passed (${response.data.latencyMs}ms)`);
				} else {
					toast.error(`Test failed: ${response.data.errorMessage}`);
				}
			} else {
				toast.error(response.error || 'Test failed');
			}
		} catch (error) {
			console.error('Failed to test gateway:', error);
			toast.error('Failed to test gateway connection');
		} finally {
			isTesting = false;
		}
	}

	async function handleSetDefaultGateway(gateway: PaymentGateway) {
		try {
			const response = await userServiceApi.setDefaultGateway(gateway.id);
			if (response.success) {
				if (settings) {
					settings.gateways = settings.gateways.map((g) => ({
						...g,
						isDefault: g.id === gateway.id
					}));
				}
				toast.success(`${gateway.name} set as default`);
			} else {
				toast.error(response.error || 'Failed to set default');
			}
		} catch (error) {
			console.error('Failed to set default gateway:', error);
			toast.error('Failed to set default gateway');
		}
	}

	async function handleViewCredentials(gateway: PaymentGateway) {
		selectedGateway = gateway;
		try {
			const response = await userServiceApi.getGatewayCredentials(gateway.id);
			if (response.success && response.data) {
				gatewayCredentials = response.data;
				showCredentialsDialog = true;
			} else {
				toast.error('Failed to load credentials');
			}
		} catch (error) {
			console.error('Failed to load credentials:', error);
			toast.error('Failed to load credentials');
		}
	}

	async function handleTogglePaymentMethod(method: PaymentMethodSettings) {
		if (!settings) return;
		const updatedMethods = settings.paymentMethods.map((m) =>
			m.type === method.type ? { ...m, enabled: !m.enabled } : m
		);
		try {
			const response = await userServiceApi.updatePaymentMethods({ methods: updatedMethods });
			if (response.success) {
				settings.paymentMethods = updatedMethods;
				toast.success(`${method.displayName} ${method.enabled ? 'disabled' : 'enabled'}`);
			} else {
				toast.error(response.error || 'Failed to update');
			}
		} catch (error) {
			console.error('Failed to update payment method:', error);
			toast.error('Failed to update payment method');
		}
	}

	async function handleToggleCurrency(currency: CurrencyConfig) {
		if (!settings) return;
		const updatedCurrencies = settings.currencies.map((c) =>
			c.code === currency.code ? { ...c, enabled: !c.enabled } : c
		);
		try {
			const response = await userServiceApi.updateCurrencies({ currencies: updatedCurrencies });
			if (response.success) {
				settings.currencies = updatedCurrencies;
				toast.success(`${currency.code} ${currency.enabled ? 'disabled' : 'enabled'}`);
			} else {
				toast.error(response.error || 'Failed to update');
			}
		} catch (error) {
			console.error('Failed to update currency:', error);
			toast.error('Failed to update currency');
		}
	}

	async function handleSetDefaultCurrency(currency: CurrencyConfig) {
		if (!settings) return;
		const updatedCurrencies = settings.currencies.map((c) => ({
			...c,
			isDefault: c.code === currency.code,
			enabled: c.code === currency.code ? true : c.enabled
		}));
		try {
			const response = await userServiceApi.updateCurrencies({ currencies: updatedCurrencies });
			if (response.success) {
				settings.currencies = updatedCurrencies;
				toast.success(`${currency.code} set as default`);
			} else {
				toast.error(response.error || 'Failed to update');
			}
		} catch (error) {
			console.error('Failed to set default currency:', error);
			toast.error('Failed to set default currency');
		}
	}

	async function handleToggleRegion(region: PaymentRegionConfig) {
		if (!settings) return;
		const updatedRegions = settings.regions.map((r) =>
			r.code === region.code ? { ...r, enabled: !r.enabled } : r
		);
		try {
			const response = await userServiceApi.updateRegions({ regions: updatedRegions });
			if (response.success) {
				settings.regions = updatedRegions;
				toast.success(`${region.name} ${region.enabled ? 'disabled' : 'enabled'}`);
			} else {
				toast.error(response.error || 'Failed to update');
			}
		} catch (error) {
			console.error('Failed to update region:', error);
			toast.error('Failed to update region');
		}
	}

	async function handleUpdateSecurity() {
		if (!settings) return;
		isSaving = true;
		try {
			const response = await userServiceApi.updatePaymentSecurity(settings.security);
			if (response.success) {
				toast.success('Security settings updated');
			} else {
				toast.error(response.error || 'Failed to update');
			}
		} catch (error) {
			console.error('Failed to update security:', error);
			toast.error('Failed to update security settings');
		} finally {
			isSaving = false;
		}
	}

	function resetNewGateway() {
		newGateway = {
			provider: 'stripe',
			name: '',
			isSandbox: true,
			isDefault: false,
			publicKey: '',
			secretKey: '',
			merchantId: '',
			webhookSecret: ''
		};
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'active':
			case 'healthy':
				return 'bg-green-100 text-green-800';
			case 'inactive':
			case 'degraded':
				return 'bg-yellow-100 text-yellow-800';
			case 'error':
			case 'down':
				return 'bg-red-100 text-red-800';
			case 'pending_setup':
				return 'bg-blue-100 text-blue-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	function formatCurrency(amount: number, currency: string = 'USD'): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: currency
		}).format(amount);
	}
</script>

<svelte:head>
	<title>Payment Settings | Anthill</title>
</svelte:head>

{#if !data.isOwner}
	<div class="container mx-auto py-8">
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<div class="flex items-start gap-3">
				<AlertTriangle class="h-5 w-5 text-destructive" />
				<div>
					<h3 class="font-semibold text-destructive">Access Denied</h3>
					<p class="text-sm text-destructive/90">
						Payment gateway settings are only accessible to organization owners.
					</p>
				</div>
			</div>
		</div>
	</div>
{:else}
	<div class="container mx-auto py-6">
		<div class="mb-6">
			<h1 class="text-2xl font-bold">Payment Settings</h1>
			<p class="text-muted-foreground">
				Configure payment gateways, methods, currencies, and security settings
			</p>
		</div>

		<div class="flex gap-6">
			<!-- Sidebar Navigation -->
			<nav class="w-56 flex-shrink-0">
				<div class="space-y-1">
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'gateways'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'gateways')}
					>
						<CreditCard class="h-4 w-4" />
						Payment Gateways
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'methods'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'methods')}
					>
						<DollarSign class="h-4 w-4" />
						Payment Methods
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'currencies'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'currencies')}
					>
						<CircleDollarSign class="h-4 w-4" />
						Currencies
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'regions'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'regions')}
					>
						<MapPin class="h-4 w-4" />
						Regions
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'fees'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'fees')}
					>
						<TrendingUp class="h-4 w-4" />
						Transaction Fees
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'security'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'security')}
					>
						<Shield class="h-4 w-4" />
						Security
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'analytics'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'analytics')}
					>
						<BarChart3 class="h-4 w-4" />
						Analytics
					</button>
					<button
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors {activeSection ===
						'health'
							? 'bg-primary text-primary-foreground'
							: 'hover:bg-muted'}"
						onclick={() => (activeSection = 'health')}
					>
						<Activity class="h-4 w-4" />
						Health Monitor
					</button>
				</div>
			</nav>

			<!-- Main Content -->
			<div class="flex-1">
				{#if isLoading}
					<div class="flex items-center justify-center py-12">
						<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
					</div>
				{:else if activeSection === 'gateways'}
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<div>
									<CardTitle>Payment Gateways</CardTitle>
									<CardDescription
										>Configure payment providers for processing transactions</CardDescription
									>
								</div>
								<Button onclick={() => (showAddGatewayDialog = true)}>
									<Plus class="mr-2 h-4 w-4" />
									Add Gateway
								</Button>
							</div>
						</CardHeader>
						<CardContent>
							{#if settings?.gateways.length === 0}
								<div class="flex flex-col items-center justify-center py-12 text-center">
									<CreditCard class="mb-4 h-12 w-12 text-muted-foreground" />
									<h3 class="text-lg font-medium">No payment gateways configured</h3>
									<p class="mb-4 text-sm text-muted-foreground">
										Add a payment gateway to start accepting payments
									</p>
									<Button onclick={() => (showAddGatewayDialog = true)}>
										<Plus class="mr-2 h-4 w-4" />
										Add Your First Gateway
									</Button>
								</div>
							{:else}
								<div class="space-y-4">
									{#each settings?.gateways || [] as gateway}
										<div
											class="flex items-center justify-between rounded-lg border p-4 {gateway.isDefault
												? 'border-primary bg-primary/5'
												: ''}"
										>
											<div class="flex items-center gap-4">
												<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-muted">
													<CreditCard class="h-6 w-6" />
												</div>
												<div>
													<div class="flex items-center gap-2">
														<span class="font-medium">{gateway.name}</span>
														{#if gateway.isDefault}
															<Badge variant="default">Default</Badge>
														{/if}
														{#if gateway.isSandbox}
															<Badge variant="outline">Sandbox</Badge>
														{/if}
													</div>
													<p class="text-sm text-muted-foreground">
														{providers.find((p) => p.value === gateway.provider)?.label ||
															gateway.provider}
													</p>
												</div>
											</div>
											<div class="flex items-center gap-2">
												<Badge class={getStatusColor(gateway.status)}
													>{gateway.status.replace('_', ' ')}</Badge
												>
												<Button
													variant="ghost"
													size="sm"
													onclick={() => handleViewCredentials(gateway)}
													title="View credentials"
												>
													<Eye class="h-4 w-4" />
												</Button>
												<Button
													variant="ghost"
													size="sm"
													onclick={() => handleTestGateway(gateway)}
													disabled={isTesting}
													title="Test connection"
												>
													<TestTube class="h-4 w-4" />
												</Button>
												{#if !gateway.isDefault}
													<Button
														variant="ghost"
														size="sm"
														onclick={() => handleSetDefaultGateway(gateway)}
														title="Set as default"
													>
														<Check class="h-4 w-4" />
													</Button>
												{/if}
												<Button
													variant="ghost"
													size="sm"
													onclick={() => {
														selectedGateway = gateway;
														showDeleteGatewayDialog = true;
													}}
													title="Delete gateway"
												>
													<Trash2 class="h-4 w-4 text-destructive" />
												</Button>
											</div>
										</div>
									{/each}
								</div>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'methods'}
					<Card>
						<CardHeader>
							<CardTitle>Payment Methods</CardTitle>
							<CardDescription>Enable or disable payment methods for your customers</CardDescription
							>
						</CardHeader>
						<CardContent>
							<div class="space-y-4">
								{#each settings?.paymentMethods || [] as method}
									<div class="flex items-center justify-between rounded-lg border p-4">
										<div class="flex items-center gap-4">
											<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
												<DollarSign class="h-5 w-5" />
											</div>
											<div>
												<span class="font-medium">{method.displayName}</span>
												<p class="text-sm text-muted-foreground">
													{#if method.minAmount && method.maxAmount}
														{formatCurrency(method.minAmount)} - {formatCurrency(method.maxAmount)}
													{:else}
														No amount limits
													{/if}
												</p>
											</div>
										</div>
										<Switch
											checked={method.enabled}
											onCheckedChange={() => handleTogglePaymentMethod(method)}
										/>
									</div>
								{:else}
									<div class="py-8 text-center text-muted-foreground">
										No payment methods configured. Add a gateway first.
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{:else if activeSection === 'currencies'}
					<Card>
						<CardHeader>
							<CardTitle>Currencies</CardTitle>
							<CardDescription>Configure supported currencies for payments</CardDescription>
						</CardHeader>
						<CardContent>
							<Table.Root>
								<Table.Header>
									<Table.Row>
										<Table.Head>Currency</Table.Head>
										<Table.Head>Symbol</Table.Head>
										<Table.Head>Status</Table.Head>
										<Table.Head class="text-right">Actions</Table.Head>
									</Table.Row>
								</Table.Header>
								<Table.Body>
									{#each settings?.currencies || [] as currency}
										<Table.Row>
											<Table.Cell>
												<div class="flex items-center gap-2">
													<span class="font-medium">{currency.code}</span>
													{#if currency.isDefault}
														<Badge variant="default">Default</Badge>
													{/if}
												</div>
												<span class="text-sm text-muted-foreground">{currency.name}</span>
											</Table.Cell>
											<Table.Cell>{currency.symbol}</Table.Cell>
											<Table.Cell>
												<Badge variant={currency.enabled ? 'default' : 'secondary'}>
													{currency.enabled ? 'Enabled' : 'Disabled'}
												</Badge>
											</Table.Cell>
											<Table.Cell class="text-right">
												<div class="flex justify-end gap-2">
													{#if !currency.isDefault && currency.enabled}
														<Button
															variant="ghost"
															size="sm"
															onclick={() => handleSetDefaultCurrency(currency)}
															title="Set as default"
														>
															<Check class="h-4 w-4" />
														</Button>
													{/if}
													<Switch
														checked={currency.enabled}
														disabled={currency.isDefault}
														onCheckedChange={() => handleToggleCurrency(currency)}
													/>
												</div>
											</Table.Cell>
										</Table.Row>
									{:else}
										<Table.Row>
											<Table.Cell colspan={4} class="py-8 text-center text-muted-foreground">
												No currencies configured
											</Table.Cell>
										</Table.Row>
									{/each}
								</Table.Body>
							</Table.Root>
						</CardContent>
					</Card>
				{:else if activeSection === 'regions'}
					<Card>
						<CardHeader>
							<CardTitle>Payment Regions</CardTitle>
							<CardDescription>Configure geographic regions for payment processing</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 md:grid-cols-2">
								{#each settings?.regions || [] as region}
									<div class="flex items-center justify-between rounded-lg border p-4">
										<div>
											<div class="flex items-center gap-2">
												<Globe class="h-4 w-4" />
												<span class="font-medium">{region.name}</span>
											</div>
											<p class="mt-1 text-sm text-muted-foreground">
												Currencies: {region.currencies.join(', ')}
											</p>
											{#if region.taxRate}
												<p class="text-sm text-muted-foreground">Tax rate: {region.taxRate}%</p>
											{/if}
										</div>
										<Switch
											checked={region.enabled}
											onCheckedChange={() => handleToggleRegion(region)}
										/>
									</div>
								{:else}
									<div class="col-span-2 py-8 text-center text-muted-foreground">
										No regions configured
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{:else if activeSection === 'fees'}
					<Card>
						<CardHeader>
							<CardTitle>Transaction Fees</CardTitle>
							<CardDescription>View fee structures for each payment gateway</CardDescription>
						</CardHeader>
						<CardContent>
							{#if settings?.gateways.length === 0}
								<div class="py-8 text-center text-muted-foreground">
									Add a payment gateway to view transaction fees
								</div>
							{:else}
								<Table.Root>
									<Table.Header>
										<Table.Row>
											<Table.Head>Gateway</Table.Head>
											<Table.Head>Fixed Fee</Table.Head>
											<Table.Head>Percentage Fee</Table.Head>
											<Table.Head>Applies To</Table.Head>
										</Table.Row>
									</Table.Header>
									<Table.Body>
										{#each settings?.gateways || [] as gateway}
											<Table.Row>
												<Table.Cell class="font-medium">{gateway.name}</Table.Cell>
												<Table.Cell>$0.30</Table.Cell>
												<Table.Cell>2.9%</Table.Cell>
												<Table.Cell>
													<div class="flex flex-wrap gap-1">
														{#each gateway.supportedMethods.slice(0, 3) as method}
															<Badge variant="outline" class="text-xs"
																>{method.replace('_', ' ')}</Badge
															>
														{/each}
														{#if gateway.supportedMethods.length > 3}
															<Badge variant="outline" class="text-xs"
																>+{gateway.supportedMethods.length - 3}</Badge
															>
														{/if}
													</div>
												</Table.Cell>
											</Table.Row>
										{/each}
									</Table.Body>
								</Table.Root>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'security'}
					<Card>
						<CardHeader>
							<CardTitle>Payment Security</CardTitle>
							<CardDescription>Configure fraud prevention and security settings</CardDescription>
						</CardHeader>
						<CardContent class="space-y-6">
							<div class="flex items-center justify-between">
								<div>
									<Label class="text-base">3D Secure Authentication</Label>
									<p class="text-sm text-muted-foreground">
										Require 3D Secure verification for card payments
									</p>
								</div>
								<Switch
									checked={settings?.security.require3DSecure || false}
									onCheckedChange={(checked) => {
										if (settings) settings.security.require3DSecure = checked;
									}}
								/>
							</div>

							<Separator />

							<div class="flex items-center justify-between">
								<div>
									<Label class="text-base">Fraud Detection</Label>
									<p class="text-sm text-muted-foreground">
										Enable automatic fraud detection on transactions
									</p>
								</div>
								<Switch
									checked={settings?.security.fraudDetectionEnabled || false}
									onCheckedChange={(checked) => {
										if (settings) settings.security.fraudDetectionEnabled = checked;
									}}
								/>
							</div>

							{#if settings?.security.fraudDetectionEnabled}
								<div class="ml-4 space-y-4 rounded-lg border p-4">
									<div class="space-y-2">
										<Label>Risk Threshold</Label>
										<Select.Root
											type="single"
											value={settings.security.fraudRiskThreshold}
											onValueChange={(v) => {
												if (v && settings)
													settings.security.fraudRiskThreshold = v as 'low' | 'medium' | 'high';
											}}
										>
											<Select.Trigger class="w-48">
												<span class="capitalize">{settings.security.fraudRiskThreshold}</span>
											</Select.Trigger>
											<Select.Content>
												<Select.Item value="low">Low (Block fewer)</Select.Item>
												<Select.Item value="medium">Medium (Balanced)</Select.Item>
												<Select.Item value="high">High (Block more)</Select.Item>
											</Select.Content>
										</Select.Root>
									</div>
								</div>
							{/if}

							<Separator />

							<div class="flex items-center justify-between">
								<div>
									<Label class="text-base">Velocity Checks</Label>
									<p class="text-sm text-muted-foreground">
										Limit transaction frequency to prevent abuse
									</p>
								</div>
								<Switch
									checked={settings?.security.velocityChecksEnabled || false}
									onCheckedChange={(checked) => {
										if (settings) settings.security.velocityChecksEnabled = checked;
									}}
								/>
							</div>

							{#if settings?.security.velocityChecksEnabled}
								<div class="ml-4 grid gap-4 rounded-lg border p-4 md:grid-cols-2">
									<div class="space-y-2">
										<Label for="maxTransactionsPerHour">Max Transactions/Hour</Label>
										<Input
											id="maxTransactionsPerHour"
											type="number"
											min="1"
											value={settings?.security.maxTransactionsPerHour || ''}
											oninput={(e) => {
												if (settings)
													settings.security.maxTransactionsPerHour = parseInt(
														e.currentTarget.value
													);
											}}
										/>
									</div>
									<div class="space-y-2">
										<Label for="maxAmountPerDay">Max Amount/Day ($)</Label>
										<Input
											id="maxAmountPerDay"
											type="number"
											min="1"
											value={settings?.security.maxAmountPerDay || ''}
											oninput={(e) => {
												if (settings)
													settings.security.maxAmountPerDay = parseInt(e.currentTarget.value);
											}}
										/>
									</div>
								</div>
							{/if}

							<Separator />

							<div class="flex justify-end">
								<Button onclick={handleUpdateSecurity} disabled={isSaving}>
									{isSaving ? 'Saving...' : 'Save Security Settings'}
								</Button>
							</div>
						</CardContent>
					</Card>
				{:else if activeSection === 'analytics'}
					<Card>
						<CardHeader>
							<CardTitle>Payment Analytics</CardTitle>
							<CardDescription>View transaction statistics and trends</CardDescription>
						</CardHeader>
						<CardContent>
							{#if analyticsData}
								<div class="grid gap-4 md:grid-cols-3">
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Total Transactions</p>
										<p class="text-2xl font-bold">{analyticsData.totalTransactions}</p>
									</div>
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Success Rate</p>
										<p class="text-2xl font-bold">
											{analyticsData.totalTransactions > 0
												? (
														(analyticsData.successfulTransactions /
															analyticsData.totalTransactions) *
														100
													).toFixed(1)
												: 0}%
										</p>
									</div>
									<div class="rounded-lg border p-4">
										<p class="text-sm text-muted-foreground">Total Volume</p>
										<p class="text-2xl font-bold">
											{formatCurrency(analyticsData.totalVolume, analyticsData.currency)}
										</p>
									</div>
								</div>

								<Separator class="my-6" />

								<div>
									<h4 class="mb-4 font-medium">Top Payment Methods</h4>
									<div class="space-y-3">
										{#each analyticsData.topPaymentMethods as method}
											<div class="flex items-center justify-between">
												<span class="capitalize">{method.method.replace('_', ' ')}</span>
												<div class="flex items-center gap-4">
													<span class="text-sm text-muted-foreground"
														>{method.count} transactions</span
													>
													<span class="font-medium"
														>{formatCurrency(method.volume, analyticsData.currency)}</span
													>
												</div>
											</div>
										{/each}
									</div>
								</div>
							{:else}
								<div class="flex items-center justify-center py-12">
									<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
								</div>
							{/if}
						</CardContent>
					</Card>
				{:else if activeSection === 'health'}
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<div>
									<CardTitle>Gateway Health</CardTitle>
									<CardDescription>Monitor payment gateway status and performance</CardDescription>
								</div>
								<Button variant="outline" size="sm" onclick={loadHealth}>
									<RefreshCw class="mr-2 h-4 w-4" />
									Refresh
								</Button>
							</div>
						</CardHeader>
						<CardContent>
							{#if healthData.length === 0}
								<div class="py-8 text-center text-muted-foreground">
									No gateway health data available
								</div>
							{:else}
								<div class="space-y-4">
									{#each healthData as health}
										<div class="rounded-lg border p-4">
											<div class="flex items-center justify-between">
												<div class="flex items-center gap-3">
													<div
														class="h-3 w-3 rounded-full {health.status === 'healthy'
															? 'bg-green-500'
															: health.status === 'degraded'
																? 'bg-yellow-500'
																: 'bg-red-500'}"
													></div>
													<span class="font-medium capitalize">{health.provider}</span>
												</div>
												<Badge class={getStatusColor(health.status)}>{health.status}</Badge>
											</div>
											<div class="mt-3 grid gap-2 text-sm md:grid-cols-3">
												<div>
													<span class="text-muted-foreground">Latency:</span>
													<span class="ml-2">{health.latencyMs || 'N/A'}ms</span>
												</div>
												<div>
													<span class="text-muted-foreground">Success Rate:</span>
													<span class="ml-2">{health.successRate || 'N/A'}%</span>
												</div>
												<div>
													<span class="text-muted-foreground">Last Checked:</span>
													<span class="ml-2"
														>{new Date(health.lastCheckedAt).toLocaleTimeString()}</span
													>
												</div>
											</div>
											{#if health.recentErrors && health.recentErrors.length > 0}
												<div class="mt-3">
													<p class="text-sm font-medium text-destructive">Recent Errors:</p>
													<ul class="mt-1 space-y-1 text-sm text-muted-foreground">
														{#each health.recentErrors.slice(0, 3) as error}
															<li>{error.message} ({error.errorCode})</li>
														{/each}
													</ul>
												</div>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</CardContent>
					</Card>
				{/if}
			</div>
		</div>
	</div>

	<!-- Add Gateway Dialog -->
	<Dialog.Root bind:open={showAddGatewayDialog}>
		<Dialog.Content class="max-w-lg">
			<Dialog.Header>
				<Dialog.Title>Add Payment Gateway</Dialog.Title>
				<Dialog.Description
					>Configure a new payment provider for your organization</Dialog.Description
				>
			</Dialog.Header>
			<div class="space-y-4">
				<div class="space-y-2">
					<Label for="provider">Provider</Label>
					<Select.Root
						type="single"
						value={newGateway.provider}
						onValueChange={(v) => {
							if (v) {
								newGateway.provider = v as PaymentProvider;
								if (!newGateway.name) {
									newGateway.name = providers.find((p) => p.value === v)?.label || '';
								}
							}
						}}
					>
						<Select.Trigger class="w-full">
							<span
								>{providers.find((p) => p.value === newGateway.provider)?.label ||
									'Select provider'}</span
							>
						</Select.Trigger>
						<Select.Content>
							{#each providers as provider}
								<Select.Item value={provider.value}>{provider.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="gatewayName">Display Name</Label>
					<Input
						id="gatewayName"
						bind:value={newGateway.name}
						placeholder="e.g., Stripe Production"
					/>
				</div>

				<div class="flex items-center justify-between">
					<div>
						<Label>Sandbox Mode</Label>
						<p class="text-sm text-muted-foreground">Use test credentials</p>
					</div>
					<Switch bind:checked={newGateway.isSandbox} />
				</div>

				<Separator />

				<div class="space-y-2">
					<Label for="publicKey">Public Key / Client ID</Label>
					<Input id="publicKey" bind:value={newGateway.publicKey} placeholder="pk_..." />
				</div>

				<div class="space-y-2">
					<Label for="secretKey">Secret Key / Client Secret</Label>
					<div class="relative">
						<Input
							id="secretKey"
							type={showSecretKey ? 'text' : 'password'}
							bind:value={newGateway.secretKey}
							placeholder="sk_..."
						/>
						<button
							type="button"
							class="absolute top-1/2 right-3 -translate-y-1/2"
							onclick={() => (showSecretKey = !showSecretKey)}
						>
							{#if showSecretKey}
								<EyeOff class="h-4 w-4 text-muted-foreground" />
							{:else}
								<Eye class="h-4 w-4 text-muted-foreground" />
							{/if}
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="webhookSecret">Webhook Secret (Optional)</Label>
					<Input id="webhookSecret" bind:value={newGateway.webhookSecret} placeholder="whsec_..." />
				</div>
			</div>
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (showAddGatewayDialog = false)}>Cancel</Button>
				<Button
					onclick={handleAddGateway}
					disabled={isSaving || !newGateway.name || !newGateway.secretKey}
				>
					{isSaving ? 'Adding...' : 'Add Gateway'}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- View Credentials Dialog -->
	<Dialog.Root bind:open={showCredentialsDialog}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Gateway Credentials</Dialog.Title>
				<Dialog.Description>{selectedGateway?.name}</Dialog.Description>
			</Dialog.Header>
			{#if gatewayCredentials}
				<div class="space-y-4">
					<div class="space-y-2">
						<Label>Public Key</Label>
						<Input value={gatewayCredentials.publicKey || 'Not configured'} readonly />
					</div>
					<div class="space-y-2">
						<Label>Secret Key (Masked)</Label>
						<Input value={gatewayCredentials.secretKeyMasked || '••••••••'} readonly />
					</div>
					{#if gatewayCredentials.merchantId}
						<div class="space-y-2">
							<Label>Merchant ID</Label>
							<Input value={gatewayCredentials.merchantId} readonly />
						</div>
					{/if}
					{#if gatewayCredentials.lastVerifiedAt}
						<p class="text-sm text-muted-foreground">
							Last verified: {new Date(gatewayCredentials.lastVerifiedAt).toLocaleString()}
						</p>
					{/if}
				</div>
			{/if}
			<Dialog.Footer>
				<Button onclick={() => (showCredentialsDialog = false)}>Close</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Delete Gateway Confirmation -->
	<Dialog.Root bind:open={showDeleteGatewayDialog}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Delete Payment Gateway</Dialog.Title>
				<Dialog.Description>
					Are you sure you want to delete "{selectedGateway?.name}"? This action cannot be undone.
				</Dialog.Description>
			</Dialog.Header>
			<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
				<div class="flex items-start gap-3">
					<AlertTriangle class="h-5 w-5 text-destructive" />
					<div>
						<h4 class="font-semibold text-destructive">Warning</h4>
						<p class="text-sm text-destructive/90">
							Deleting this gateway will disable all associated payment methods and may affect
							pending transactions.
						</p>
					</div>
				</div>
			</div>
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (showDeleteGatewayDialog = false)}>Cancel</Button>
				<Button variant="destructive" onclick={handleDeleteGateway} disabled={isSaving}>
					{isSaving ? 'Deleting...' : 'Delete Gateway'}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
