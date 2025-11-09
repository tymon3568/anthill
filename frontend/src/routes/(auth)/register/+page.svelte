<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { registerAction } from '$lib/hooks/useAuthActions';
	import { validateRegister, type RegisterInput } from '$lib/validation/auth-validation';
	import { rateLimiter, formatBlockedTime } from '$lib/utils/rate-limiter';
	import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';

	// Form state using Svelte 5 runes
	let fullName = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let tenantName = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let fieldErrors = $state<Partial<Record<keyof RegisterInput, string>>>({});
	let fullNameInputEl: HTMLElement | undefined = $state();

	// Rate limiting state
	const RATE_LIMIT_KEY = 'register';
	let isRateLimited = $state(false);
	let blockedTimeRemaining = $state(0);
	let remainingAttempts = $state(5);
	let rateLimitInterval: ReturnType<typeof setInterval> | undefined;

	// Debounce timer for form submission
	let submitTimeout: ReturnType<typeof setTimeout> | undefined;
	const SUBMIT_DEBOUNCE_MS = 300;

	// Password strength indicator
	let passwordStrength = $derived.by(() => {
		if (!password) return { score: 0, label: '', color: '' };

		let score = 0;
		if (password.length >= 8) score++;
		if (password.length >= 12) score++;
		if (/[A-Z]/.test(password)) score++;
		if (/[a-z]/.test(password)) score++;
		if (/[0-9]/.test(password)) score++;

		const labels = ['', 'Weak', 'Fair', 'Good', 'Strong', 'Very Strong'];
		const colors = ['', '#ef4444', '#f59e0b', '#eab308', '#22c55e', '#16a34a'];

		return {
			score,
			label: labels[score] || '',
			color: colors[score] || ''
		};
	});

	// Check rate limit status on mount
	onMount(() => {
		// Auto-focus first input
		if (fullNameInputEl) {
			const input = fullNameInputEl.querySelector('input');
			input?.focus();
		}

		checkRateLimitStatus();

		// Update blocked time countdown every second
		rateLimitInterval = setInterval(() => {
			if (isRateLimited) {
				const remaining = rateLimiter.getBlockedTimeRemaining(RATE_LIMIT_KEY);
				if (remaining === 0) {
					isRateLimited = false;
					remainingAttempts = rateLimiter.getRemainingAttempts(RATE_LIMIT_KEY);
				} else {
					blockedTimeRemaining = remaining;
				}
			}
		}, 1000);

		return () => {
			if (rateLimitInterval) clearInterval(rateLimitInterval);
			if (submitTimeout) clearTimeout(submitTimeout);
		};
	});

	function checkRateLimitStatus() {
		const blocked = rateLimiter.getBlockedTimeRemaining(RATE_LIMIT_KEY);
		if (blocked > 0) {
			isRateLimited = true;
			blockedTimeRemaining = blocked;
			remainingAttempts = 0;
		} else {
			isRateLimited = false;
			remainingAttempts = rateLimiter.getRemainingAttempts(RATE_LIMIT_KEY);
		}
	}

	// Handle form submission with validation and rate limiting
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Clear previous debounce
		if (submitTimeout) {
			clearTimeout(submitTimeout);
		}

		// Debounce submission (300ms)
		submitTimeout = setTimeout(async () => {
			// Check rate limit before validation
			if (!rateLimiter.isAllowed(RATE_LIMIT_KEY)) {
				checkRateLimitStatus();
				error = `Too many registration attempts. Please try again in ${formatBlockedTime(blockedTimeRemaining)}.`;
				return;
			}

			// Validate form using Valibot
			const result = validateRegister({
				full_name: fullName,
				email,
				password,
				confirmPassword,
				tenant_name: tenantName || undefined
			});

			if (!result.success) {
				fieldErrors = result.errors;
				error = '';
				return;
			}

			// Clear field errors on successful validation
			fieldErrors = {};
			isLoading = true;
			error = '';

			try {
				// Call register with proper field names
				await registerAction({
					name: result.data.full_name,
					email: result.data.email,
					password: result.data.password,
					confirmPassword: result.data.confirmPassword,
					tenantName: result.data.tenant_name
				});

				// Redirect to login with success message
				goto('/login?message=Registration successful. Please sign in.');
			} catch (err) {
				// Map errors according to production standards
				let errorMsg = 'Registration failed';

				if (err instanceof Error) {
					const message = err.message.toLowerCase();

					// 409 Conflict - email already exists
					if (message.includes('409') || message.includes('already exists') || message.includes('conflict')) {
						errorMsg = 'An account with this email already exists.';
					}
					// 500 Server Error
					else if (message.includes('500') || message.includes('server') || message.includes('unavailable')) {
						errorMsg = 'Service temporarily unavailable. Please try again later.';
					}
					// Timeout
					else if (message.includes('timeout')) {
						errorMsg = 'Request timeout. Please check your connection and try again.';
					}
					// Other errors
					else {
						errorMsg = err.message;
					}
				}

				error = errorMsg;
				checkRateLimitStatus();
			} finally {
				isLoading = false;
			}
		}, SUBMIT_DEBOUNCE_MS);
	}
</script>
</script>

<svelte:head>
	<title>Sign Up - Anthill</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Create your account</h1>
			<p class="mt-2 text-sm text-gray-600">Join Anthill to manage your inventory</p>
		</div>

		<Card>
			<CardHeader>
				<CardTitle>Sign Up</CardTitle>
				<CardDescription>
					Create a new account to get started
				</CardDescription>
			</CardHeader>

			<CardContent>
				<form class="space-y-4">
					<div>
						<Label for="name">Full Name</Label>
						<Input
							id="name"
							name="name"
							type="text"
							placeholder="Enter your full name"
							bind:value={name}
							required
							autocomplete="name"
							disabled={isLoading}
							aria-describedby={validationErrors.name ? "name-error" : undefined}
							onblur={() => touched.name = true}
						/>
						{#if touched.name && validationErrors.name}
							<p id="name-error" class="text-sm text-red-600 mt-1" role="alert">
								{validationErrors.name}
							</p>
						{/if}
					</div>

					<div>
						<Label for="email">Email</Label>
						<Input
							id="email"
							name="email"
							type="email"
							placeholder="Enter your email"
							bind:value={email}
							required
							autocomplete="email"
							disabled={isLoading}
							aria-describedby={validationErrors.email ? "email-error" : undefined}
							onblur={() => touched.email = true}
						/>
						{#if touched.email && validationErrors.email}
							<p id="email-error" class="text-sm text-red-600 mt-1" role="alert">
								{validationErrors.email}
							</p>
						{/if}
					</div>

					<div>
						<Label for="tenantName">Company/Organization Name</Label>
						<Input
							id="tenantName"
							name="tenantName"
							type="text"
							placeholder="Enter your company name (optional)"
							bind:value={tenantName}
							autocomplete="organization"
							disabled={isLoading}
							onblur={() => touched.tenantName = true}
						/>
						<p class="text-xs text-gray-500 mt-1">
							Leave empty to join an existing organization
						</p>
					</div>

					<div>
						<Label for="password">Password</Label>
						<Input
							id="password"
							name="password"
							type="password"
							placeholder="Create a password"
							bind:value={password}
							required
							autocomplete="new-password"
							disabled={isLoading}
							aria-describedby={validationErrors.password ? "password-error" : undefined}
							onblur={() => touched.password = true}
						/>
						{#if touched.password && validationErrors.password}
							<p id="password-error" class="text-sm text-red-600 mt-1" role="alert">
								{validationErrors.password}
							</p>
						{/if}
						{#if password}
							<div class="mt-2">
								<div class="flex items-center space-x-2">
									<div class="flex-1 bg-gray-200 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all duration-300"
											style="width: {(passwordStrength.score / 5) * 100}%; background-color: {passwordStrengthColor}"
										></div>
									</div>
									<span class="text-xs text-gray-600">{passwordStrengthText}</span>
								</div>
							</div>
						{/if}
					</div>

					<div>
						<Label for="confirmPassword">Confirm Password</Label>
						<Input
							id="confirmPassword"
							name="confirmPassword"
							type="password"
							placeholder="Confirm your password"
							bind:value={confirmPassword}
							required
							autocomplete="new-password"
							disabled={isLoading}
							aria-describedby={validationErrors.confirmPassword ? "confirm-error" : undefined}
							onblur={() => touched.confirmPassword = true}
						/>
						{#if touched.confirmPassword && validationErrors.confirmPassword}
							<p id="confirm-error" class="text-sm text-red-600 mt-1" role="alert">
								{validationErrors.confirmPassword}
							</p>
						{/if}
					</div>

					{#if error}
						<div
							class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md p-3"
							role="alert"
							aria-live="polite"
						>
							{error}
						</div>
					{/if}

					<Button
						type="button"
						class="w-full"
						disabled={isLoading}
						onclick={handleSubmit}
					>
						{#if isLoading}
							<span class="flex items-center space-x-2">
								<div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
								<span>Creating account...</span>
							</span>
						{:else}
							Create Account
						{/if}
					</Button>
				</form>

				<div class="mt-6 text-center">
					<p class="text-sm text-gray-600">
						Already have an account?
						<a
							href="/login"
							class="text-primary hover:text-primary/80 underline font-medium"
							tabindex={isLoading ? -1 : 0}
						>
							Sign in
						</a>
					</p>
				</div>
			</CardContent>
		</Card>
	</div>
</div>

<style>
	/* Additional responsive styles if needed */
	@media (max-width: 640px) {
		.min-h-screen {
			padding-top: 2rem;
			padding-bottom: 2rem;
		}
	}
</style>
