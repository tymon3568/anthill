<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { useAuth } from '$lib/hooks/useAuth';
	import { loginSchema } from '$lib/auth/validation';
	import { parse, safeParse } from 'valibot';

	// Form state using Svelte 5 runes
	let email = $state('');
	let password = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let submitted = $state(false);
	let touched = $state({ email: false, password: false });

	// Auth hook
	const { login, isAuthenticated } = useAuth();

	// Get success message from URL params
	let successMessage = $state(page.url.searchParams.get('message'));

	// Get error message from URL params (for OAuth errors)
	let errorMessage = $state(page.url.searchParams.get('error_description') || page.url.searchParams.get('error'));

	// Form validation using Valibot
	let isFormValid = $derived.by(() => {
		try {
			parse(loginSchema, { email, password });
			return true;
		} catch {
			return false;
		}
	});

	// Get validation errors - only show after form submission attempt
	let validationErrors = $derived.by(() => {
		const result = safeParse(loginSchema, { email, password });
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
	});	// Handle form submission
	async function handleSubmit(event: Event) {
		event.preventDefault();

		// Mark all fields as touched and enable error display
		touched = { email: true, password: true };
		submitted = true;

		if (!isFormValid) return;

		isLoading = true;
		error = '';

		try {
			await login(email, password);
			// Redirect will be handled by auth store
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
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

				<form class="space-y-4">
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
							placeholder="Enter your password"
							bind:value={password}
							required
							autocomplete="current-password"
							disabled={isLoading}
							aria-describedby={validationErrors.password ? "password-error" : undefined}
							onblur={() => touched.password = true}
						/>
						{#if touched.password && validationErrors.password}
							<p id="password-error" class="text-sm text-red-600 mt-1" role="alert">
								{validationErrors.password}
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
							tabindex={isLoading ? -1 : 0}
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
