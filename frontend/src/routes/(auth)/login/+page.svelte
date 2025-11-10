<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { loginAction } from '$lib/hooks/useAuthActions';
	import { validateLogin, type LoginInput } from '$lib/validation/auth-validation';
	import { rateLimiter, formatBlockedTime } from '$lib/utils/rate-limiter';
	import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';

	// Form state using Svelte 5 runes
	let email = $state('');
	let password = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let fieldErrors = $state<Partial<Record<keyof LoginInput, string>>>({});
	let emailInputEl: HTMLElement | undefined = $state();

	// Rate limiting state
	const RATE_LIMIT_KEY = 'login';
	let isRateLimited = $state(false);
	let blockedTimeRemaining = $state(0);
	let remainingAttempts = $state(5);
	let rateLimitInterval: ReturnType<typeof setInterval> | undefined;

	// Debounce timer for form submission
	let submitTimeout: ReturnType<typeof setTimeout> | undefined;
	const SUBMIT_DEBOUNCE_MS = 300;

	// Get success message from URL params
	let successMessage = $state(page.url.searchParams.get('message'));

	// Get error message from URL params (for OAuth errors or session expiry)
	let errorMessage = $state(page.url.searchParams.get('error_description') || page.url.searchParams.get('error'));

	// Check rate limit status on mount and set up interval
	onMount(() => {
		// Auto-focus email input for accessibility
		if (emailInputEl) {
			const input = emailInputEl.querySelector('input');
			input?.focus();
		}

		// Check initial rate limit status
		checkRateLimitStatus();

		// Update blocked time countdown every second
		rateLimitInterval = setInterval(() => {
			if (isRateLimited) {
				const remaining = rateLimiter.getBlockedTimeRemaining(RATE_LIMIT_KEY);
				if (remaining === 0) {
					// Block expired, reset state
					isRateLimited = false;
					remainingAttempts = rateLimiter.getRemainingAttempts(RATE_LIMIT_KEY);
				} else {
					blockedTimeRemaining = remaining;
				}
			}
		}, 1000);

		return () => {
			// Cleanup interval on unmount
			if (rateLimitInterval) {
				clearInterval(rateLimitInterval);
			}
			if (submitTimeout) {
				clearTimeout(submitTimeout);
			}
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
				error = `Too many login attempts. Please try again in ${formatBlockedTime(blockedTimeRemaining)}.`;
				return;
			}

			// Validate form using Valibot
			const result = validateLogin({ email, password });

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
				await loginAction(result.data.email, result.data.password);
				// Redirect to dashboard after successful login
				goto('/dashboard');
			} catch (err) {
				// Map errors according to production standards
				let errorMsg = 'Login failed';

				if (err instanceof Error) {
					const message = err.message.toLowerCase();

					// 401 Unauthorized - invalid credentials
					if (message.includes('401') || message.includes('unauthorized') || message.includes('invalid credentials')) {
						errorMsg = 'Invalid email or password. Please try again.';
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

				// Update rate limit status
				checkRateLimitStatus();
			} finally {
				isLoading = false;
			}
		}, SUBMIT_DEBOUNCE_MS);
	}
</script>

<svelte:head>
	<title>Sign In - Anthill</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Welcome back</h1>
			<p class="mt-2 text-sm text-gray-600">Sign in to your account</p>
		</div>

		<Card>
			<CardHeader>
				<CardTitle>Sign In</CardTitle>
				<CardDescription>
					Enter your credentials to access your account
				</CardDescription>
			</CardHeader>

			<CardContent>
				{#if successMessage}
					<div
						class="text-sm text-green-600 bg-green-50 border border-green-200 rounded-md p-3 mb-4"
						role="alert"
						aria-live="polite"
					>
						{successMessage}
					</div>
				{/if}

				{#if errorMessage}
					<div
						class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md p-3 mb-4"
						role="alert"
						aria-live="polite"
					>
						{errorMessage}
					</div>
				{/if}

				<!-- Rate limit warning -->
				{#if isRateLimited}
					<div
						class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md p-3 mb-4"
						role="alert"
						aria-live="assertive"
					>
						<strong>Too many login attempts.</strong>
						<br />
						Please wait {formatBlockedTime(blockedTimeRemaining)} before trying again.
					</div>
				{:else if remainingAttempts < 3 && remainingAttempts > 0}
					<div
						class="text-sm text-amber-600 bg-amber-50 border border-amber-200 rounded-md p-3 mb-4"
						role="alert"
						aria-live="polite"
					>
						{remainingAttempts} {remainingAttempts === 1 ? 'attempt' : 'attempts'} remaining before temporary lockout.
					</div>
				{/if}

				<form class="space-y-4" onsubmit={handleSubmit}>
					<div bind:this={emailInputEl}>
						<Label for="email">Email</Label>
						<Input
							id="email"
							name="email"
							type="email"
							placeholder="Enter your email"
							bind:value={email}
							required
							autocomplete="email"
							disabled={isLoading || isRateLimited}
							aria-describedby={fieldErrors.email ? "email-error" : undefined}
							aria-invalid={fieldErrors.email ? "true" : "false"}
							aria-label="Email address"
						/>
						{#if fieldErrors.email}
							<p id="email-error" class="text-sm text-red-600 mt-1" role="alert" aria-live="polite">
								{fieldErrors.email}
							</p>
						{/if}
					</div>

					<div>
						<Label for="password">Password</Label>
						<Input
							id="password"
							name="password"
							type="password"
							placeholder="Enter your password"
							bind:value={password}
							required
							autocomplete="current-password"
							disabled={isLoading || isRateLimited}
							aria-describedby={fieldErrors.password ? "password-error" : undefined}
							aria-invalid={fieldErrors.password ? "true" : "false"}
							aria-label="Password"
						/>
						{#if fieldErrors.password}
							<p id="password-error" class="text-sm text-red-600 mt-1" role="alert" aria-live="polite">
								{fieldErrors.password}
							</p>
						{/if}
					</div>

					{#if error}
						<div
							class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md p-3"
							role="alert"
							aria-live="assertive"
						>
							{error}
						</div>
					{/if}

					<Button
						type="submit"
						class="w-full"
						disabled={isLoading || isRateLimited}
						aria-busy={isLoading}
					>
						{#if isLoading}
							<span class="flex items-center justify-center gap-2">
								<LoadingSpinner size="sm" class="border-white border-t-transparent" />
								<span>Signing in...</span>
							</span>
						{:else}
							Sign In
						{/if}
					</Button>
				</form>

				<div class="mt-6 text-center">
					<p class="text-sm text-gray-600">
						Don't have an account?
						<a
							href="/register"
							class="text-primary hover:text-primary/80 underline font-medium"
							tabindex={isLoading || isRateLimited ? -1 : 0}
						>
							Create one
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
