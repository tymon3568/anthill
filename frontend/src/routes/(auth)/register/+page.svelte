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
	import { fullRegisterSchema, type RegisterForm } from '$lib/validation/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { safeParse } from 'valibot';
	import { authApi } from '$lib/api/auth';
	import { toast } from 'svelte-sonner';
	import Mail from '@lucide/svelte/icons/mail';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Building2 from '@lucide/svelte/icons/building-2';
	import UserPlus from '@lucide/svelte/icons/user-plus';

	// Form state using Svelte 5 runes
	let formData = $state<RegisterForm>({
		email: '',
		password: '',
		confirmPassword: '',
		fullName: '',
		tenantName: ''
	});
	let isLoading = $state(false);
	let error = $state('');
	let fieldErrors = $state<Record<string, string>>({});
	let registrationSuccess = $state(false);
	let registeredEmail = $state('');
	let registeredTenantId = $state('');
	let isResending = $state(false);

	// Session storage keys for persistence across page refresh
	const STORAGE_KEY_EMAIL = 'anthill_register_email';
	const STORAGE_KEY_TENANT_ID = 'anthill_register_tenant_id';
	const STORAGE_KEY_SUCCESS = 'anthill_register_success';

	// Restore registration state from sessionStorage on mount (browser only)
	$effect(() => {
		if (typeof window !== 'undefined') {
			const storedEmail = sessionStorage.getItem(STORAGE_KEY_EMAIL);
			const storedTenantId = sessionStorage.getItem(STORAGE_KEY_TENANT_ID);
			const storedSuccess = sessionStorage.getItem(STORAGE_KEY_SUCCESS);

			if (storedSuccess === 'true' && storedEmail) {
				registeredEmail = storedEmail;
				registeredTenantId = storedTenantId || '';
				registrationSuccess = true;
			}
		}
	});

	// Helper to persist registration state to sessionStorage
	function persistRegistrationState(email: string, tenantId: string) {
		if (typeof window !== 'undefined') {
			sessionStorage.setItem(STORAGE_KEY_EMAIL, email);
			sessionStorage.setItem(STORAGE_KEY_TENANT_ID, tenantId);
			sessionStorage.setItem(STORAGE_KEY_SUCCESS, 'true');
		}
	}

	// Helper to clear registration state from sessionStorage
	function clearRegistrationState() {
		if (typeof window !== 'undefined') {
			sessionStorage.removeItem(STORAGE_KEY_EMAIL);
			sessionStorage.removeItem(STORAGE_KEY_TENANT_ID);
			sessionStorage.removeItem(STORAGE_KEY_SUCCESS);
		}
	}

	// Handle navigation to login page - clears session storage
	function handleGoToLogin() {
		clearRegistrationState();
		goto('/login');
	}

	// Cooldown timer for resend button
	let resendCooldown = $state(0);
	let cooldownInterval: ReturnType<typeof setInterval> | null = null;

	const RESEND_COOLDOWN_SECONDS = 60;

	// Tenant availability check state
	let tenantCheckStatus = $state<'idle' | 'checking' | 'available' | 'exists' | 'error'>('idle');
	let tenantCheckSlug = $state('');
	let existingTenantName = $state<string | null>(null);

	// Convert tenant name to slug for display
	function toSlug(name: string): string {
		return name
			.trim()
			.toLowerCase()
			.replace(/\s+/g, '-')
			.replace(/[^a-z0-9-]/g, '');
	}

	// Derived slug from tenant name
	let tenantSlug = $derived(toSlug(formData.tenantName));

	// Check tenant availability with debounce
	async function checkTenantAvailability(tenantName: string) {
		const slug = toSlug(tenantName);

		// Reset if empty
		if (!slug) {
			tenantCheckStatus = 'idle';
			tenantCheckSlug = '';
			existingTenantName = null;
			return;
		}

		// Don't check if slug hasn't changed
		if (slug === tenantCheckSlug && tenantCheckStatus !== 'idle') {
			return;
		}

		tenantCheckStatus = 'checking';
		tenantCheckSlug = slug;

		try {
			const response = await authApi.checkTenantSlug(slug);
			if (response.success && response.data) {
				// Make sure we're still checking the same slug (in case user typed more)
				if (toSlug(formData.tenantName) === slug) {
					if (response.data.available) {
						tenantCheckStatus = 'available';
						existingTenantName = null;
					} else {
						tenantCheckStatus = 'exists';
						existingTenantName = response.data.existing_tenant_name;
					}
				}
			} else {
				tenantCheckStatus = 'error';
			}
		} catch {
			tenantCheckStatus = 'error';
		}
	}

	// Handle tenant name input - just clear errors
	function handleTenantNameInput() {
		clearFieldError('tenantName');
		// Reset status when user is typing
		if (tenantCheckStatus !== 'idle') {
			tenantCheckStatus = 'idle';
			tenantCheckSlug = '';
			existingTenantName = null;
		}
	}

	// Handle tenant name blur - check availability when user leaves input
	function handleTenantNameBlur() {
		if (formData.tenantName.trim()) {
			checkTenantAvailability(formData.tenantName);
		}
	}

	// Form submission handler
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Validate with valibot schema
		const result = safeParse(fullRegisterSchema, formData);

		if (!result.success) {
			// Extract field-specific errors
			fieldErrors = {};
			result.issues.forEach((issue) => {
				if (issue.path) {
					const field = issue.path[0]?.key as string;
					if (field) {
						// Only set first error for each field
						if (!fieldErrors[field]) {
							fieldErrors[field] = issue.message;
						}
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
			// Call the auth store register method
			const result = await authStore.emailRegister(
				formData.email,
				formData.password,
				formData.fullName.trim(),
				formData.tenantName.trim()
			);

			if (result.success) {
				// Show verification email message instead of redirecting
				registeredEmail = formData.email;
				// Get tenant_id from the response for resend functionality
				if (result.data?.tenant_id) {
					registeredTenantId = result.data.tenant_id;
				}
				// Persist state to sessionStorage for page refresh resilience
				persistRegistrationState(registeredEmail, registeredTenantId);
				registrationSuccess = true;
			} else {
				error = result.error || 'Registration failed';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Registration failed';
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
	}

	// Start cooldown timer
	function startCooldown() {
		resendCooldown = RESEND_COOLDOWN_SECONDS;
		if (cooldownInterval) {
			clearInterval(cooldownInterval);
		}
		cooldownInterval = setInterval(() => {
			resendCooldown--;
			if (resendCooldown <= 0) {
				if (cooldownInterval) {
					clearInterval(cooldownInterval);
					cooldownInterval = null;
				}
			}
		}, 1000);
	}

	// Resend verification email
	async function handleResendVerification() {
		if (!registeredEmail || isResending || resendCooldown > 0) return;

		isResending = true;
		try {
			const response = await authApi.resendVerification(registeredEmail, registeredTenantId);
			if (response.success) {
				toast.success('Verification email sent! Please check your inbox.');
				startCooldown();
			} else {
				if (response.error?.includes('rate') || response.error?.includes('limit')) {
					toast.error('Please wait a few minutes before requesting another email.');
					startCooldown();
				} else {
					toast.error(response.error || 'Failed to resend verification email');
				}
			}
		} catch (error) {
			console.error('Failed to resend verification email:', error);
			toast.error('Failed to resend verification email. Please try again later.');
		} finally {
			isResending = false;
		}
	}
</script>

<svelte:head>
	<title>Register - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
		{#if registrationSuccess}
			<!-- Success State: Check Your Email -->
			<div class="mb-8 text-center">
				<h1 class="text-3xl font-bold text-gray-900">Check your email</h1>
				<p class="mt-2 text-sm text-gray-600">We've sent you a verification link</p>
			</div>

			<Card>
				<CardHeader class="text-center">
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100"
					>
						<Mail class="h-8 w-8 text-green-600" />
					</div>
					<CardTitle class="text-green-700">Account Created!</CardTitle>
					<CardDescription>
						We've sent a verification email to <strong class="text-gray-900"
							>{registeredEmail}</strong
						>. Please check your inbox and click the verification link to activate your account.
					</CardDescription>
				</CardHeader>

				<CardContent class="space-y-4">
					<div class="rounded-md border border-blue-200 bg-blue-50 p-4 text-sm text-blue-800">
						<div class="flex items-start gap-3">
							<CheckCircle class="mt-0.5 h-5 w-5 flex-shrink-0 text-blue-600" />
							<div>
								<p class="font-medium">What's next?</p>
								<ul class="mt-1 list-inside list-disc space-y-1 text-blue-700">
									<li>Check your email inbox</li>
									<li>Click the verification link</li>
									<li>Sign in to your account</li>
								</ul>
							</div>
						</div>
					</div>

					<div class="text-center">
						<p class="text-sm text-gray-600">Didn't receive the email?</p>
						<Button
							variant="link"
							onclick={handleResendVerification}
							disabled={isResending || resendCooldown > 0}
							class="text-blue-600"
						>
							{#if isResending}
								Sending...
							{:else if resendCooldown > 0}
								Resend in {resendCooldown}s
							{:else}
								Resend verification email
							{/if}
						</Button>
					</div>

					<Button onclick={handleGoToLogin} variant="outline" class="w-full">Go to Login</Button>
				</CardContent>
			</Card>

			<div class="mt-4 text-center text-xs text-gray-500">
				<p>The verification link will expire in 24 hours.</p>
			</div>
		{:else}
			<!-- Registration Form -->
			<div class="mb-8 text-center">
				<h1 class="text-3xl font-bold text-gray-900">Create your account</h1>
				<p class="mt-2 text-sm text-gray-600">Join Anthill to manage your inventory</p>
			</div>

			<Card>
				<CardHeader>
					<CardTitle>Sign Up</CardTitle>
				</CardHeader>

				<CardContent>
					<form onsubmit={handleSubmit} class="space-y-4">
						<div class="space-y-2">
							<Label for="fullName">Full Name</Label>
							<Input
								id="fullName"
								type="text"
								placeholder="Enter your full name"
								bind:value={formData.fullName}
								required
								disabled={isLoading}
								class={fieldErrors.fullName ? 'border-red-500' : ''}
								oninput={() => clearFieldError('fullName')}
							/>
							{#if fieldErrors.fullName}
								<p class="text-sm text-red-600">{fieldErrors.fullName}</p>
							{/if}
						</div>

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
							<Label for="tenantName">Organization</Label>
							<Input
								id="tenantName"
								type="text"
								placeholder="Enter your organization name"
								bind:value={formData.tenantName}
								required
								disabled={isLoading}
								class={fieldErrors.tenantName
									? 'border-red-500'
									: tenantCheckStatus === 'available'
										? 'border-green-500 focus:ring-green-500'
										: tenantCheckStatus === 'exists'
											? 'border-amber-500 focus:ring-amber-500'
											: ''}
								oninput={handleTenantNameInput}
								onblur={handleTenantNameBlur}
							/>

							<!-- Tenant availability feedback -->
							{#if fieldErrors.tenantName}
								<p class="text-sm text-red-600">{fieldErrors.tenantName}</p>
							{:else if tenantCheckStatus === 'checking'}
								<div class="flex items-center gap-2 text-sm text-gray-500">
									<Loader2 class="h-4 w-4 animate-spin" />
									<span>Checking availability...</span>
								</div>
							{:else if tenantCheckStatus === 'available'}
								<div
									class="flex items-center gap-2 rounded-md border border-green-200 bg-green-50 p-2 text-sm"
								>
									<Building2 class="h-4 w-4 flex-shrink-0 text-green-600" />
									<div class="text-green-700">
										<span class="font-medium">{formData.tenantName}</span>
										<span class="text-green-600">({tenantSlug})</span> is available.
										<span class="font-medium">You will be the Owner.</span>
									</div>
								</div>
							{:else if tenantCheckStatus === 'exists'}
								<div
									class="flex items-center gap-2 rounded-md border border-amber-200 bg-amber-50 p-2 text-sm"
								>
									<UserPlus class="h-4 w-4 flex-shrink-0 text-amber-600" />
									<div class="text-amber-700">
										<span class="font-medium">{existingTenantName || tenantSlug}</span> already
										exists.
										<span class="font-medium">You will request to join.</span>
									</div>
								</div>
							{:else if tenantCheckStatus === 'error'}
								<div class="flex items-center gap-2 text-sm text-gray-500">
									<CircleAlert class="h-4 w-4 text-gray-400" />
									<span>Could not check availability</span>
								</div>
							{:else}
								<p class="text-xs text-gray-500">
									Enter an organization name to create or join a workspace
								</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="password">Password</Label>
							<Input
								id="password"
								type="password"
								placeholder="Create a password"
								bind:value={formData.password}
								required
								disabled={isLoading}
								class={fieldErrors.password ? 'border-red-500' : ''}
								oninput={() => clearFieldError('password')}
							/>
							{#if fieldErrors.password}
								<p class="text-sm text-red-600">{fieldErrors.password}</p>
							{:else}
								<p class="text-xs text-gray-500">
									Min 8 characters with uppercase, lowercase, and number
								</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="confirmPassword">Confirm Password</Label>
							<Input
								id="confirmPassword"
								type="password"
								placeholder="Confirm your password"
								bind:value={formData.confirmPassword}
								required
								disabled={isLoading}
								class={fieldErrors.confirmPassword ? 'border-red-500' : ''}
								oninput={() => clearFieldError('confirmPassword')}
							/>
							{#if fieldErrors.confirmPassword}
								<p class="text-sm text-red-600">{fieldErrors.confirmPassword}</p>
							{/if}
						</div>

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
								Creating account...
							{:else}
								Create Account
							{/if}
						</Button>
					</form>

					<div class="mt-6 text-center">
						<p class="text-sm text-gray-600">
							Already have an account?
							<a href="/login" class="font-medium text-blue-600 underline hover:text-blue-500">
								Sign in
							</a>
						</p>
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
</div>
