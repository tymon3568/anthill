<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { fullRegisterSchema, type RegisterForm } from '$lib/validation/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { goto } from '$app/navigation';
	import { safeParse } from 'valibot';

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
						fieldErrors[field] = issue.message;
					}
				}
			});
			error = 'Please correct the errors below';
			return;
		}

		// Additional password confirmation check
		if (formData.password !== formData.confirmPassword) {
			fieldErrors = { confirmPassword: 'Passwords do not match' };
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
				// Redirect to dashboard or intended page
				goto('/dashboard');
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
</script>

<svelte:head>
	<title>Register - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
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
							class={fieldErrors.tenantName ? 'border-red-500' : ''}
							oninput={() => clearFieldError('tenantName')}
						/>
						{#if fieldErrors.tenantName}
							<p class="text-sm text-red-600">{fieldErrors.tenantName}</p>
						{:else}
							<p class="text-xs text-gray-500">This will create a new organization for you</p>
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
							<p class="text-xs text-gray-500">Password must be at least 8 characters long</p>
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
	</div>
</div>
