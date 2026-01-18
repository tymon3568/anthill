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
	import { loginSchema, type LoginForm } from '$lib/validation/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { safeParse } from 'valibot';
	import { onMount } from 'svelte';
	import { setPersistedTenantSlug, getTenantContext, type TenantContext } from '$lib/tenant';
	import { authApi } from '$lib/api/auth';

	// Form state using Svelte 5 runes
	let formData = $state<LoginForm>({
		email: '',
		password: ''
	});

	// Tenant state
	let tenantSlug = $state('');
	let tenantContext = $state<TenantContext>({ slug: null, source: null, hasContext: false });
	let showTenantInput = $state(false);

	let isLoading = $state(false);
	let error = $state('');
	let fieldErrors = $state<Record<string, string>>({});
	let successMessage = $state('');

	// Initialize tenant context on mount
	onMount(() => {
		tenantContext = getTenantContext();
		if (tenantContext.hasContext && tenantContext.slug) {
			tenantSlug = tenantContext.slug;
			// Set tenant in API client
			authApi.setTenantSlug(tenantContext.slug);

			// Only hide input if detected from subdomain (trusted source)
			// If from storage, show input with pre-filled value so user can change if needed
			if (tenantContext.source === 'subdomain') {
				showTenantInput = false;
			} else {
				// From storage - show input with pre-filled value
				showTenantInput = true;
			}
		} else {
			// No context detected, show tenant input
			showTenantInput = true;
		}
	});

	// Reactively update success message based on URL params
	$effect(() => {
		if ($page.url.searchParams.get('reset') === 'success') {
			successMessage =
				'Your password has been reset successfully. Please sign in with your new password.';
		} else {
			successMessage = '';
		}
	});

	// Update tenant context when slug changes - always sync with API client
	function handleTenantChange() {
		// Always sync API client with current input value (even if empty)
		authApi.setTenantSlug(tenantSlug.trim() || null);
	}

	// Form submission handler
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Determine effective tenant slug based on UI state
		// When showTenantInput is true, require manual input and do NOT fall back to tenantContext.slug
		// This prevents silently logging into the old tenant when user clears input after "Switch organization"
		let effectiveTenantSlug: string | null;
		if (showTenantInput) {
			// Manual input mode - require user-provided value
			effectiveTenantSlug = tenantSlug.trim() || null;
		} else {
			// Subdomain mode - use context slug
			effectiveTenantSlug = tenantContext.slug;
		}

		if (!effectiveTenantSlug) {
			error = 'Please enter your organization/tenant name';
			fieldErrors = { ...fieldErrors, tenant: 'Organization is required' };
			return;
		}

		// Set tenant in API client before login
		authApi.setTenantSlug(effectiveTenantSlug);

		// Validate form data
		const result = safeParse(loginSchema, formData);

		if (!result.success) {
			// Extract field-specific errors
			fieldErrors = {};
			result.issues.forEach((issue) => {
				if (issue.path) {
					const field = issue.path[0]?.key as string;
					if (field) {
						fieldErrors[field] = issue.message;
					}
				}
			});
			error = 'Please correct the errors below';
			return;
		}

		// Clear previous errors
		error = '';
		fieldErrors = {};
		isLoading = true;

		try {
			// Persist tenant slug for future use
			setPersistedTenantSlug(effectiveTenantSlug);

			// Call the auth store login method
			const loginResult = await authStore.emailLogin(formData.email, formData.password);

			if (loginResult.success) {
				// Redirect to dashboard or intended page
				goto('/dashboard');
			} else {
				error = loginResult.error || 'Login failed';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			isLoading = false;
		}
	}

	// Clear field error when user starts typing
	function clearFieldError(field: string) {
		if (fieldErrors[field]) {
			fieldErrors = { ...fieldErrors };
			delete fieldErrors[field];
		}
		// Clear general error when user starts typing
		if (error) {
			error = '';
		}
	}
</script>

<svelte:head>
	<title>Login - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
		<div class="mb-8 text-center">
			<h1 class="text-3xl font-bold text-gray-900">Welcome back</h1>
			<p class="mt-2 text-sm text-gray-600">Sign in to your Anthill account</p>
		</div>

		<Card>
			<CardHeader>
				<CardTitle>Sign In</CardTitle>
				{#if tenantContext.hasContext && tenantContext.source === 'subdomain' && !showTenantInput}
					<CardDescription>
						Signing in to <strong class="text-gray-900">{tenantContext.slug}</strong>
					</CardDescription>
				{/if}
			</CardHeader>

			<CardContent>
				<form onsubmit={handleSubmit} class="space-y-4">
					<!-- Tenant/Organization Input - shown when not detected from subdomain or user wants to switch -->
					{#if showTenantInput}
						<div class="space-y-2">
							<Label for="tenant">Organization</Label>
							<Input
								id="tenant"
								type="text"
								placeholder="Enter your organization (e.g., acme)"
								bind:value={tenantSlug}
								required
								disabled={isLoading}
								class={fieldErrors.tenant ? 'border-red-500' : ''}
								oninput={() => {
									clearFieldError('tenant');
									handleTenantChange();
								}}
							/>
							{#if fieldErrors.tenant}
								<p class="text-sm text-red-600">{fieldErrors.tenant}</p>
							{:else}
								<p class="text-xs text-gray-500">This is your organization's unique identifier</p>
							{/if}
						</div>

						<div class="relative">
							<div class="absolute inset-0 flex items-center">
								<span class="w-full border-t border-gray-200"></span>
							</div>
							<div class="relative flex justify-center text-xs uppercase">
								<span class="bg-white px-2 text-gray-500">credentials</span>
							</div>
						</div>
					{/if}

					<div class="space-y-2">
						<Label for="email">Email</Label>
						<Input
							id="email"
							type="email"
							placeholder="Enter your email"
							bind:value={formData.email}
							required
							disabled={isLoading}
							class={fieldErrors.email ? 'border-red-500' : ''}
							oninput={() => clearFieldError('email')}
						/>
						{#if fieldErrors.email}
							<p class="text-sm text-red-600">{fieldErrors.email}</p>
						{/if}
					</div>

					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<Label for="password">Password</Label>
							<a
								href="/forgot-password"
								class="text-sm font-medium text-blue-600 hover:text-blue-500"
							>
								Forgot password?
							</a>
						</div>
						<Input
							id="password"
							type="password"
							placeholder="Enter your password"
							bind:value={formData.password}
							required
							disabled={isLoading}
							class={fieldErrors.password ? 'border-red-500' : ''}
							oninput={() => clearFieldError('password')}
						/>
						{#if fieldErrors.password}
							<p class="text-sm text-red-600">{fieldErrors.password}</p>
						{/if}
					</div>

					{#if successMessage}
						<div
							class="rounded-md border border-green-200 bg-green-50 p-3 text-sm text-green-700"
							role="status"
						>
							{successMessage}
						</div>
					{/if}

					{#if error && !Object.keys(fieldErrors).length}
						<div
							class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600"
							role="alert"
						>
							{error}
						</div>
					{/if}

					<Button type="submit" class="w-full" disabled={isLoading}>
						{#if isLoading}
							<span
								class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
							></span>
							Signing in...
						{:else}
							Sign In
						{/if}
					</Button>
				</form>

				<div class="mt-6 text-center">
					<p class="text-sm text-gray-600">
						Don't have an account?
						<a href="/register" class="font-medium text-blue-600 underline hover:text-blue-500">
							Sign up
						</a>
					</p>
				</div>

				<!-- Toggle tenant input visibility if detected from subdomain -->
				{#if tenantContext.hasContext && tenantContext.source === 'subdomain' && !showTenantInput}
					<div class="mt-4 text-center">
						<button
							type="button"
							class="text-xs text-gray-500 underline hover:text-gray-700"
							onclick={() => {
								showTenantInput = true;
								// Clear the tenant slug when switching to manual mode
								// so user must explicitly enter a new value
								tenantSlug = '';
								authApi.setTenantSlug(null);
							}}
						>
							Switch organization
						</button>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Help text for subdomain usage -->
		<div class="mt-4 text-center text-xs text-gray-500">
			<p>
				Tip: Access your organization directly at
				<code class="rounded bg-gray-100 px-1 py-0.5">your-org.anthill.example.com</code>
			</p>
		</div>
	</div>
</div>
