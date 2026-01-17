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
	import { authApi } from '$lib/api/auth';
	import Mail from '@lucide/svelte/icons/mail';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import CheckCircle from '@lucide/svelte/icons/check-circle';

	let email = $state('');
	let isLoading = $state(false);
	let isSubmitted = $state(false);
	let error = $state('');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		isLoading = true;
		error = '';

		try {
			const response = await authApi.forgotPassword(email);

			if (response.success) {
				isSubmitted = true;
			} else {
				// Check for rate limiting
				if (
					response.error?.toLowerCase().includes('rate') ||
					response.error?.toLowerCase().includes('limit')
				) {
					error = 'Too many requests. Please try again later.';
				} else {
					// Always show success for security (timing-safe)
					isSubmitted = true;
				}
			}
		} catch (err) {
			console.error('Forgot password request failed:', err);
			// Always show success for security (timing-safe) except for rate limiting
			isSubmitted = true;
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Forgot Password - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
		<div class="mb-8 text-center">
			<h1 class="text-3xl font-bold text-gray-900">Reset Password</h1>
			<p class="mt-2 text-sm text-gray-600">We'll send you a link to reset your password</p>
		</div>

		<Card>
			<CardHeader class="text-center">
				{#if isSubmitted}
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100"
					>
						<CheckCircle class="h-8 w-8 text-green-600" />
					</div>
					<CardTitle class="text-green-700">Check Your Email</CardTitle>
					<CardDescription>
						If an account exists with <strong class="text-gray-900">{email}</strong>, you'll receive
						a password reset link shortly.
					</CardDescription>
				{:else}
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-100"
					>
						<Mail class="h-8 w-8 text-blue-600" />
					</div>
					<CardTitle>Forgot Your Password?</CardTitle>
					<CardDescription>
						Enter your email address and we'll send you a link to reset your password.
					</CardDescription>
				{/if}
			</CardHeader>

			<CardContent>
				{#if isSubmitted}
					<div class="space-y-4">
						<div class="rounded-md border border-blue-200 bg-blue-50 p-4 text-sm text-blue-800">
							<p>
								Didn't receive the email? Check your spam folder, or make sure you entered the
								correct email address.
							</p>
						</div>

						<a href="/login">
							<Button variant="outline" class="w-full">
								<ArrowLeft class="mr-2 h-4 w-4" />
								Back to Login
							</Button>
						</a>
					</div>
				{:else}
					<form onsubmit={handleSubmit} class="space-y-4">
						<div class="space-y-2">
							<Label for="email">Email</Label>
							<Input
								id="email"
								type="email"
								placeholder="Enter your email"
								bind:value={email}
								required
								disabled={isLoading}
								class={error ? 'border-red-500' : ''}
							/>
						</div>

						{#if error}
							<div
								class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600"
								role="alert"
							>
								{error}
							</div>
						{/if}

						<Button type="submit" class="w-full" disabled={isLoading || !email}>
							{#if isLoading}
								<span
									class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
								></span>
								Sending...
							{:else}
								Send Reset Link
							{/if}
						</Button>

						<a href="/login">
							<Button variant="ghost" class="w-full">
								<ArrowLeft class="mr-2 h-4 w-4" />
								Back to Login
							</Button>
						</a>
					</form>
				{/if}
			</CardContent>
		</Card>
	</div>
</div>
