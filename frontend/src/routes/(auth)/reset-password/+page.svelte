<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
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
	import { authApi } from '$lib/api/auth';
	import { passwordStrengthSchema } from '$lib/validation/auth';
	import { safeParse } from 'valibot';
	import Lock from '@lucide/svelte/icons/lock';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import XCircle from '@lucide/svelte/icons/x-circle';
	import Eye from '@lucide/svelte/icons/eye';
	import EyeOff from '@lucide/svelte/icons/eye-off';

	interface Props {
		data: {
			token: string | null;
		};
	}

	let { data }: Props = $props();

	// Token validation state
	let isValidating = $state(true);
	let tokenValid = $state(false);
	let tokenError = $state('');

	// Form state
	let newPassword = $state('');
	let confirmPassword = $state('');
	let showPassword = $state(false);
	let showConfirmPassword = $state(false);
	let isLoading = $state(false);
	let isSuccess = $state(false);
	let error = $state('');
	let fieldErrors = $state<Record<string, string>>({});

	// Derived state
	let passwordsMatch = $derived(newPassword === confirmPassword);
	let passwordStrength = $derived(getPasswordStrength(newPassword));

	function getPasswordStrength(password: string): { score: number; label: string; color: string } {
		if (!password) return { score: 0, label: '', color: '' };

		let score = 0;
		if (password.length >= 8) score++;
		if (password.length >= 12) score++;
		if (/[a-z]/.test(password)) score++;
		if (/[A-Z]/.test(password)) score++;
		if (/[0-9]/.test(password)) score++;
		if (/[^a-zA-Z0-9]/.test(password)) score++;

		if (score <= 2) return { score, label: 'Weak', color: 'bg-red-500' };
		if (score <= 4) return { score, label: 'Fair', color: 'bg-yellow-500' };
		return { score, label: 'Strong', color: 'bg-green-500' };
	}

	onMount(() => {
		if (data.token) {
			validateToken(data.token);
		} else {
			isValidating = false;
			tokenError = 'No reset token provided. Please check your email link.';
		}
	});

	async function validateToken(token: string) {
		try {
			const response = await authApi.validateResetToken(token);

			if (response.success) {
				tokenValid = true;
			} else {
				handleTokenError(response.error);
			}
		} catch (err) {
			console.error('Token validation failed:', err);
			tokenError = 'Failed to validate reset link. Please try again.';
		} finally {
			isValidating = false;
		}
	}

	function handleTokenError(error?: string) {
		const errorLower = error?.toLowerCase() || '';

		if (errorLower.includes('expired')) {
			tokenError = 'This reset link has expired. Please request a new one.';
		} else if (errorLower.includes('invalid') || errorLower.includes('not found')) {
			tokenError = 'This reset link is invalid. Please check your email for the correct link.';
		} else {
			tokenError = error || 'Invalid reset link.';
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		// Validate password
		const result = safeParse(passwordStrengthSchema, newPassword);
		if (!result.success) {
			fieldErrors = { password: result.issues[0]?.message || 'Invalid password' };
			return;
		}

		if (!passwordsMatch) {
			fieldErrors = { confirmPassword: 'Passwords do not match' };
			return;
		}

		isLoading = true;
		error = '';
		fieldErrors = {};

		try {
			const response = await authApi.resetPassword(data.token!, newPassword, confirmPassword);

			if (response.success) {
				isSuccess = true;
			} else {
				if (response.error?.toLowerCase().includes('expired')) {
					tokenValid = false;
					tokenError = 'This reset link has expired. Please request a new one.';
				} else {
					error = response.error || 'Failed to reset password';
				}
			}
		} catch (err) {
			console.error('Password reset failed:', err);
			error = 'Failed to reset password. Please try again.';
		} finally {
			isLoading = false;
		}
	}

	function goToLogin() {
		goto('/login?reset=success');
	}
</script>

<svelte:head>
	<title>Reset Password - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
		<div class="mb-8 text-center">
			<h1 class="text-3xl font-bold text-gray-900">Reset Password</h1>
			<p class="mt-2 text-sm text-gray-600">Create a new password for your account</p>
		</div>

		<Card>
			{#if isValidating}
				<!-- Loading State -->
				<CardHeader class="text-center">
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-100"
					>
						<Loader2 class="h-8 w-8 animate-spin text-blue-600" />
					</div>
					<CardTitle>Validating Reset Link</CardTitle>
					<CardDescription>Please wait while we verify your reset link...</CardDescription>
				</CardHeader>
			{:else if !tokenValid}
				<!-- Token Error State -->
				<CardHeader class="text-center">
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-red-100"
					>
						<XCircle class="h-8 w-8 text-red-600" />
					</div>
					<CardTitle class="text-red-700">Invalid Reset Link</CardTitle>
					<CardDescription class="text-gray-600">{tokenError}</CardDescription>
				</CardHeader>
				<CardContent>
					<a href="/forgot-password">
						<Button class="w-full">Request New Reset Link</Button>
					</a>
				</CardContent>
			{:else if isSuccess}
				<!-- Success State -->
				<CardHeader class="text-center">
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100"
					>
						<CheckCircle class="h-8 w-8 text-green-600" />
					</div>
					<CardTitle class="text-green-700">Password Reset!</CardTitle>
					<CardDescription>
						Your password has been successfully reset. You can now sign in with your new password.
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Button onclick={goToLogin} class="w-full">Go to Login</Button>
				</CardContent>
			{:else}
				<!-- Password Reset Form -->
				<CardHeader class="text-center">
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-100"
					>
						<Lock class="h-8 w-8 text-blue-600" />
					</div>
					<CardTitle>Set New Password</CardTitle>
					<CardDescription>
						Enter your new password below. Make sure it's at least 8 characters with a mix of
						letters, numbers, and symbols.
					</CardDescription>
				</CardHeader>

				<CardContent>
					<form onsubmit={handleSubmit} class="space-y-4">
						<div class="space-y-2">
							<Label for="newPassword">New Password</Label>
							<div class="relative">
								<Input
									id="newPassword"
									type={showPassword ? 'text' : 'password'}
									placeholder="Enter new password"
									bind:value={newPassword}
									required
									disabled={isLoading}
									class={fieldErrors.password ? 'border-red-500 pr-10' : 'pr-10'}
								/>
								<button
									type="button"
									class="absolute top-1/2 right-3 -translate-y-1/2 text-gray-400 hover:text-gray-600"
									onclick={() => (showPassword = !showPassword)}
								>
									{#if showPassword}
										<EyeOff class="h-4 w-4" />
									{:else}
										<Eye class="h-4 w-4" />
									{/if}
								</button>
							</div>
							{#if fieldErrors.password}
								<p class="text-sm text-red-600">{fieldErrors.password}</p>
							{/if}

							<!-- Password Strength Indicator -->
							{#if newPassword}
								<div class="space-y-1">
									<div class="flex gap-1">
										{#each [1, 2, 3, 4, 5, 6] as level (level)}
											<div
												class="h-1 flex-1 rounded-full {passwordStrength.score >= level
													? passwordStrength.color
													: 'bg-gray-200'}"
											></div>
										{/each}
									</div>
									<p
										class="text-xs {passwordStrength.score <= 2
											? 'text-red-600'
											: passwordStrength.score <= 4
												? 'text-yellow-600'
												: 'text-green-600'}"
									>
										Password strength: {passwordStrength.label}
									</p>
								</div>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="confirmPassword">Confirm Password</Label>
							<div class="relative">
								<Input
									id="confirmPassword"
									type={showConfirmPassword ? 'text' : 'password'}
									placeholder="Confirm new password"
									bind:value={confirmPassword}
									required
									disabled={isLoading}
									class={fieldErrors.confirmPassword || (confirmPassword && !passwordsMatch)
										? 'border-red-500 pr-10'
										: 'pr-10'}
								/>
								<button
									type="button"
									class="absolute top-1/2 right-3 -translate-y-1/2 text-gray-400 hover:text-gray-600"
									onclick={() => (showConfirmPassword = !showConfirmPassword)}
								>
									{#if showConfirmPassword}
										<EyeOff class="h-4 w-4" />
									{:else}
										<Eye class="h-4 w-4" />
									{/if}
								</button>
							</div>
							{#if fieldErrors.confirmPassword}
								<p class="text-sm text-red-600">{fieldErrors.confirmPassword}</p>
							{:else if confirmPassword && !passwordsMatch}
								<p class="text-sm text-red-600">Passwords do not match</p>
							{:else if confirmPassword && passwordsMatch}
								<p class="text-sm text-green-600">Passwords match</p>
							{/if}
						</div>

						{#if error}
							<div
								class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600"
								role="alert"
							>
								{error}
							</div>
						{/if}

						<Button
							type="submit"
							class="w-full"
							disabled={isLoading || !newPassword || !confirmPassword || !passwordsMatch}
						>
							{#if isLoading}
								<span
									class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
								></span>
								Resetting Password...
							{:else}
								Reset Password
							{/if}
						</Button>
					</form>
				</CardContent>
			{/if}
		</Card>
	</div>
</div>
