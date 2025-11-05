<script lang="ts">
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { useAuth } from '$lib/hooks/useAuth';
	import type { RegisterForm } from '$lib/types';
	import { registerSchema, validatePasswordConfirmation, calculatePasswordStrength } from '$lib/auth/validation';
	import { parse, safeParse } from 'valibot';

	// Form state using Svelte 5 runes
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let name = $state('');
	let isLoading = $state(false);
	let error = $state('');

	// Auth hook
	const { isAuthenticated } = useAuth();

	// Password strength calculation using Valibot helper
	let passwordStrength = $derived(() => calculatePasswordStrength(password));

	let passwordStrengthText = $derived(() => passwordStrength().strength);
	let passwordStrengthColor = $derived(() => passwordStrength().color);

	// Form validation using Valibot
	let isFormValid = $derived(() => {
		try {
			parse(registerSchema, { name, email, password, confirmPassword });
			const passwordsMatch = validatePasswordConfirmation(password, confirmPassword);
			const strength = passwordStrength();
			return passwordsMatch && strength.score >= 3;
		} catch {
			return false;
		}
	});

	// Get validation errors
	let validationErrors = $derived(() => {
		const result = safeParse(registerSchema, { name, email, password, confirmPassword });
		const errors: Record<string, string> = {};

		if (!result.success) {
			result.issues.forEach((issue: any) => {
				const field = issue.path?.[0]?.key as string;
				if (field) {
					errors[field] = issue.message;
				}
			});
		}

		// Check password confirmation
		if (!validatePasswordConfirmation(password, confirmPassword)) {
			errors.confirmPassword = 'Passwords do not match';
		}

		// Check password strength
		const strength = passwordStrength();
		if (strength.score < 3) {
			errors.password = 'Password is too weak';
		}

		return errors;
	});

	// Redirect if already authenticated
	$effect(() => {
		if (isAuthenticated) {
			goto('/dashboard');
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();

		const validation = safeParse(registerSchema, { name, email, password, confirmPassword });
		const passwordsMatch = validatePasswordConfirmation(password, confirmPassword);
		const strength = passwordStrength();

		if (!validation.success || !passwordsMatch || strength.score < 3) {
			error = 'Please fix the errors above';
			return;
		}

		isLoading = true;
		error = '';

		try {
			// TODO: Implement registration API call
			// For now, just redirect to login
			goto('/login?message=Registration successful, please login');
		} catch (err) {
			error = 'Registration failed. Please try again.';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Register - Anthill Inventory</title>
	<meta name="description" content="Create your Anthill Inventory account" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Anthill Inventory</h1>
			<p class="mt-2 text-sm text-gray-600">Create your account</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="text-2xl text-center">Register</CardTitle>
				<CardDescription class="text-center">
					Enter your details to create a new account
				</CardDescription>
			</CardHeader>
			<CardContent>
				<form onsubmit={handleSubmit} class="space-y-4">
					<div class="space-y-2">
						<Label for="name">Full Name</Label>
						<Input
							id="name"
							type="text"
							placeholder="Enter your full name"
							bind:value={name}
							required
							autocomplete="name"
							disabled={isLoading}
							aria-describedby={validationErrors().name ? "name-error" : undefined}
						/>
						{#if validationErrors().name}
							<p id="name-error" class="text-sm text-red-600" role="alert">
								{validationErrors().name}
							</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="email">Email</Label>
						<Input
							id="email"
							type="email"
							placeholder="Enter your email"
							bind:value={email}
							required
							autocomplete="email"
							disabled={isLoading}
							aria-describedby={validationErrors().email ? "email-error" : undefined}
						/>
						{#if validationErrors().email}
							<p id="email-error" class="text-sm text-red-600" role="alert">
								{validationErrors().email}
							</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="password">Password</Label>
						<Input
							id="password"
							type="password"
							placeholder="Create a password"
							bind:value={password}
							required
							autocomplete="new-password"
							disabled={isLoading}
							aria-describedby={validationErrors().password ? "password-error" : "password-strength"}
						/>
						{#if password}
							<div id="password-strength" class="flex items-center space-x-2">
								<div class="flex-1 bg-gray-200 rounded-full h-2">
								<div
									class="h-2 rounded-full transition-all duration-300 {passwordStrengthColor().replace('text-', 'bg-')}"
									style="width: {(passwordStrength().score / 5) * 100}%"
								></div>
								</div>
								<span class="text-sm {passwordStrengthColor()}">
									{passwordStrengthText()}
								</span>
							</div>
							<div class="text-xs text-gray-500 mt-1">
								Password must contain at least 8 characters with uppercase, lowercase, and numbers
							</div>
						{/if}
						{#if validationErrors().password}
							<p id="password-error" class="text-sm text-red-600" role="alert">
								{validationErrors().password}
							</p>
						{/if}
					</div>

					<div class="space-y-2">
						<Label for="confirmPassword">Confirm Password</Label>
						<Input
							id="confirmPassword"
							type="password"
							placeholder="Confirm your password"
							bind:value={confirmPassword}
							required
							autocomplete="new-password"
							disabled={isLoading}
							aria-describedby={validationErrors().confirmPassword ? "confirm-error" : undefined}
						/>
						{#if validationErrors().confirmPassword}
							<p id="confirm-error" class="text-sm text-red-600" role="alert">
								{validationErrors().confirmPassword}
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
						type="submit"
						class="w-full"
						disabled={!isFormValid || isLoading}
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
