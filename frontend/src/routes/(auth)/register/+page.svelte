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
	let isResending = $state(false);

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

	// Resend verification email
	async function handleResendVerification() {
		if (!registeredEmail || isResending) return;

		isResending = true;
		try {
			const response = await authApi.resendVerification(registeredEmail);
			if (response.success) {
				toast.success('Verification email sent! Please check your inbox.');
			} else {
				if (response.error?.includes('rate') || response.error?.includes('limit')) {
					toast.error('Please wait a few minutes before requesting another email.');
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
							disabled={isResending}
							class="text-blue-600"
						>
							{#if isResending}
								Sending...
							{:else}
								Resend verification email
							{/if}
						</Button>
					</div>

					<Button onclick={() => goto('/login')} variant="outline" class="w-full">
						Go to Login
					</Button>
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
