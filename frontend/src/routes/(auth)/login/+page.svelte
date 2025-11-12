<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { loginSchema, type LoginForm } from '$lib/validation/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { safeParse } from 'valibot';

	// Form state using Svelte 5 runes
	let formData = $state<LoginForm>({
		email: '',
		password: ''
	});
	let isLoading = $state(false);
	let error = $state('');
	let fieldErrors = $state<Record<string, string>>({});

	// Form submission handler
	async function handleSubmit(event: Event) {
		event.preventDefault();

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
			// Call the auth store login method
			const result = await authStore.emailLogin(formData.email, formData.password);

			if (result.success) {
				// Redirect to dashboard or intended page
				goto('/dashboard');
			} else {
				error = result.error || 'Login failed';
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
			</CardHeader>

			<CardContent>
				<form onsubmit={handleSubmit} class="space-y-4">
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
						<Label for="password">Password</Label>
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
			</CardContent>
		</Card>
	</div>
</div>
