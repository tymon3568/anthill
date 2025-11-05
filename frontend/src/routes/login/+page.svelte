<script lang="ts">
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { useAuth } from '$lib/hooks/useAuth';
	import type { LoginForm } from '$lib/types';
	import { loginSchema, type LoginForm as ValibotLoginForm } from '$lib/auth/validation';
	import { parse, safeParse } from 'valibot';

	// Form state using Svelte 5 runes
	let email = $state('');
	let password = $state('');
	let rememberMe = $state(false);
	let isLoading = $state(false);
	let error = $state('');

	// Auth hook
	const { login, isAuthenticated } = useAuth();

	// Redirect if already authenticated
	$effect(() => {
		if (isAuthenticated) {
			goto('/dashboard');
		}
	});

	// Form validation using Valibot
	let isFormValid = $derived(() => {
		try {
			parse(loginSchema, { email, password });
			return true;
		} catch {
			return false;
		}
	});

	// Get validation errors
	let validationErrors = $derived(() => {
		const result = safeParse(loginSchema, { email, password });
		if (!result.success) {
			return result.issues.reduce((acc: Record<string, string>, issue: any) => {
				const field = issue.path?.[0]?.key as string;
				if (field) {
					acc[field] = issue.message;
				}
				return acc;
			}, {});
		}
		return {};
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();

		const validation = safeParse(loginSchema, { email, password });
		if (!validation.success) {
			error = 'Please fix the errors below';
			return;
		}

		isLoading = true;
		error = '';

		try {
			const result = await login(email, password);
			if (result.success) {
				goto('/dashboard');
			} else {
				error = result.error || 'Login failed';
			}
		} catch (err) {
			error = 'An unexpected error occurred';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Login - Anthill Inventory</title>
	<meta name="description" content="Login to your Anthill Inventory account" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900">Anthill Inventory</h1>
			<p class="mt-2 text-sm text-gray-600">Sign in to your account</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="text-2xl text-center">Login</CardTitle>
				<CardDescription class="text-center">
					Enter your email and password to access your account
				</CardDescription>
			</CardHeader>
			<CardContent>
				<form onsubmit={handleSubmit} class="space-y-4">
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
							aria-describedby={validationErrors().email ? "email-error" : error ? "error-message" : undefined}
						/>
					</div>
						{#if validationErrors().email}
							<p id="email-error" class="text-sm text-red-600" role="alert">
								{validationErrors().email}
							</p>
						{/if}					<div class="space-y-2">
						<Label for="password">Password</Label>
						<Input
							id="password"
							type="password"
							placeholder="Enter your password"
							bind:value={password}
							required
							autocomplete="current-password"
							disabled={isLoading}
							aria-describedby={validationErrors().password ? "password-error" : error ? "error-message" : undefined}
						/>
					</div>
						{#if validationErrors().password}
							<p id="password-error" class="text-sm text-red-600" role="alert">
								{validationErrors().password}
							</p>
						{/if}					<div class="flex items-center justify-between">
						<div class="flex items-center space-x-2">
							<input
								id="remember-me"
								type="checkbox"
								bind:checked={rememberMe}
								disabled={isLoading}
								class="h-4 w-4 text-primary focus:ring-primary border-gray-300 rounded"
							/>
							<Label for="remember-me" class="text-sm">Remember me</Label>
						</div>

						<a
							href="/forgot-password"
							class="text-sm text-primary hover:text-primary/80 underline"
							tabindex={isLoading ? -1 : 0}
						>
							Forgot password?
						</a>
					</div>

					{#if error}
						<div
							id="error-message"
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
								<span>Signing in...</span>
							</span>
						{:else}
							Sign in
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
							Sign up
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
