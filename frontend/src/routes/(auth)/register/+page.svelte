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
	let submitted = $state(false);
	let touched = $state({ name: false, email: false, password: false, confirmPassword: false });

	// Auth hook
	const { register, isAuthenticated } = useAuth();

	// Password strength calculation using Valibot helper
	let passwordStrength = $derived(() => calculatePasswordStrength(password));

	let passwordStrengthText = $derived(() => passwordStrength().strength);
	let passwordStrengthColor = $derived(() => passwordStrength().color);

	// Form validation using Valibot
	let isFormValid = $derived.by(() => {
		try {
			parse(registerSchema, { name, email, password, confirmPassword });
			const passwordsMatch = validatePasswordConfirmation(password, confirmPassword);
			const strength = passwordStrength();
			return passwordsMatch && strength.score >= 3;
		} catch {
			return false;
		}
	});

	// Get validation errors - only show after form submission attempt
	let validationErrors = $derived.by(() => {
		const result = safeParse(registerSchema, { name, email, password, confirmPassword });
		const errors: Record<string, string> = {};

		if (!result.success) {
			result.issues.forEach((issue: any) => {
				const field = issue.path?.[0]?.key as string;
				if (field && !errors[field]) { // Only take the first error per field
					errors[field] = issue.message;
				}
			});
		}
		return errors;
	});				// Handle form submission
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Mark all fields as touched and enable error display
		touched = { name: true, email: true, password: true, confirmPassword: true };
		submitted = true;

		if (!isFormValid) return;

		isLoading = true;
		error = '';

		try {
			await register({ name, email, password, confirmPassword });
			// Redirect to login with success message
			goto('/login?message=Registration successful, please login');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Registration failed';
		} finally {
			isLoading = false;
		}
	}

	// Redirect if already authenticated
	$effect(() => {
		if (isAuthenticated) {
			goto('/dashboard');
		}
	});
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
											style="width: {(passwordStrength().score / 5) * 100}%; background-color: {passwordStrengthColor()}"
										></div>
									</div>
									<span class="text-xs text-gray-600">{passwordStrengthText()}</span>
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
