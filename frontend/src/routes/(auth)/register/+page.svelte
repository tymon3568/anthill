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
				<!-- Rate limit warning -->
				{#if isRateLimited}
					<div
						class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md p-3 mb-4"
						role="alert"
						aria-live="assertive"
					>
						<strong>Too many registration attempts.</strong>
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
					<div bind:this={fullNameInputEl}>
						<Label for="fullName">Full Name</Label>
						<Input
							id="fullName"
							name="fullName"
							type="text"
							placeholder="Enter your full name"
							bind:value={fullName}
							required
							autocomplete="name"
							disabled={isLoading || isRateLimited}
							aria-describedby={fieldErrors.full_name ? "name-error" : undefined}
							aria-invalid={fieldErrors.full_name ? "true" : "false"}
							aria-label="Full name"
						/>
						{#if fieldErrors.full_name}
							<p id="name-error" class="text-sm text-red-600 mt-1" role="alert" aria-live="polite">
								{fieldErrors.full_name}
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
						<Label for="tenantName">Company/Organization Name</Label>
						<Input
							id="tenantName"
							name="tenantName"
							type="text"
							placeholder="Enter your company name (optional)"
							bind:value={tenantName}
							autocomplete="organization"
							disabled={isLoading || isRateLimited}
							aria-describedby="tenant-help"
							aria-label="Company or organization name"
						/>
						<p id="tenant-help" class="text-xs text-gray-500 mt-1">
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
							disabled={isLoading || isRateLimited}
							aria-describedby={fieldErrors.password ? "password-error" : "password-help"}
							aria-invalid={fieldErrors.password ? "true" : "false"}
							aria-label="Password"
						/>
						{#if fieldErrors.password}
							<p id="password-error" class="text-sm text-red-600 mt-1" role="alert" aria-live="polite">
								{fieldErrors.password}
							</p>
						{:else if password}
							<div id="password-help" class="mt-2">
								<div class="flex items-center space-x-2">
									<div class="flex-1 bg-gray-200 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all duration-300"
											style="width: {(passwordStrength.score / 5) * 100}%; background-color: {passwordStrength.color}"
											role="progressbar"
											aria-valuenow={passwordStrength.score}
											aria-valuemin={0}
											aria-valuemax={5}
											aria-label="Password strength"
										></div>
									</div>
									<span class="text-xs text-gray-600">{passwordStrength.label}</span>
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
							disabled={isLoading || isRateLimited}
							aria-describedby={fieldErrors.confirmPassword ? "confirm-error" : undefined}
							aria-invalid={fieldErrors.confirmPassword ? "true" : "false"}
							aria-label="Confirm password"
						/>
						{#if fieldErrors.confirmPassword}
							<p id="confirm-error" class="text-sm text-red-600 mt-1" role="alert" aria-live="polite">
								{fieldErrors.confirmPassword}
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
							tabindex={isLoading || isRateLimited ? -1 : 0}
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
