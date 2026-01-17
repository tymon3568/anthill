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
	import { authApi } from '$lib/api/auth';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import XCircle from '@lucide/svelte/icons/x-circle';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Mail from '@lucide/svelte/icons/mail';

	interface Props {
		data: {
			token: string | null;
		};
	}

	let { data }: Props = $props();

	type VerificationStatus = 'loading' | 'success' | 'error';
	type ErrorType = 'invalid' | 'expired' | 'already_verified' | 'no_token' | 'unknown';

	let status = $state<VerificationStatus>('loading');
	let errorType = $state<ErrorType>('unknown');
	let errorMessage = $state('');

	onMount(() => {
		if (data.token) {
			verifyEmail(data.token);
		} else {
			status = 'error';
			errorType = 'no_token';
			errorMessage = 'No verification token provided. Please check your email link.';
		}
	});

	async function verifyEmail(token: string) {
		try {
			const response = await authApi.verifyEmail(token);

			if (response.success) {
				status = 'success';
			} else {
				status = 'error';
				handleErrorResponse(response.error);
			}
		} catch (error) {
<<<<<<< HEAD
			console.error('Email verification failed:', error);
=======
>>>>>>> aa67ac9 (feat(frontend): Add Email Verification UI [TaskID: 08.02.06])
			status = 'error';
			errorType = 'unknown';
			errorMessage = error instanceof Error ? error.message : 'An unexpected error occurred';
		}
	}

	function handleErrorResponse(error?: string) {
		const errorLower = error?.toLowerCase() || '';

		if (errorLower.includes('expired')) {
			errorType = 'expired';
			errorMessage = 'This verification link has expired. Please request a new one.';
		} else if (errorLower.includes('already') || errorLower.includes('verified')) {
			errorType = 'already_verified';
			errorMessage = 'Your email is already verified. You can log in now.';
		} else if (errorLower.includes('invalid') || errorLower.includes('not found')) {
			errorType = 'invalid';
			errorMessage =
				'This verification link is invalid. Please check your email for the correct link.';
		} else {
			errorType = 'unknown';
			errorMessage = error || 'Failed to verify email. Please try again later.';
		}
	}

	function goToLogin() {
		goto('/login');
	}
</script>

<svelte:head>
	<title>Verify Email - Anthill</title>
</svelte:head>

<div class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8">
	<div class="w-full max-w-md">
		<div class="mb-8 text-center">
			<h1 class="text-3xl font-bold text-gray-900">Email Verification</h1>
			<p class="mt-2 text-sm text-gray-600">Confirming your email address</p>
		</div>

		<Card>
			<CardHeader class="text-center">
				{#if status === 'loading'}
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-100"
					>
						<Loader2 class="h-8 w-8 animate-spin text-blue-600" />
					</div>
					<CardTitle>Verifying your email...</CardTitle>
					<CardDescription>Please wait while we confirm your email address.</CardDescription>
				{:else if status === 'success'}
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100"
					>
						<CheckCircle class="h-8 w-8 text-green-600" />
					</div>
					<CardTitle class="text-green-700">Email Verified!</CardTitle>
					<CardDescription>
						Your email has been successfully verified. You can now sign in to your account.
					</CardDescription>
				{:else}
					<div
						class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-red-100"
					>
						{#if errorType === 'already_verified'}
							<Mail class="h-8 w-8 text-blue-600" />
						{:else}
							<XCircle class="h-8 w-8 text-red-600" />
						{/if}
					</div>
					<CardTitle class={errorType === 'already_verified' ? 'text-blue-700' : 'text-red-700'}>
						{#if errorType === 'expired'}
							Link Expired
						{:else if errorType === 'already_verified'}
							Already Verified
						{:else if errorType === 'invalid' || errorType === 'no_token'}
							Invalid Link
						{:else}
							Verification Failed
						{/if}
					</CardTitle>
					<CardDescription class="text-gray-600">{errorMessage}</CardDescription>
				{/if}
			</CardHeader>

			<CardContent class="space-y-4">
				{#if status === 'success'}
					<Button onclick={goToLogin} class="w-full">Go to Login</Button>
				{:else if status === 'error'}
					{#if errorType === 'already_verified'}
						<Button onclick={goToLogin} class="w-full">Go to Login</Button>
					{:else if errorType === 'expired'}
						<div class="space-y-3">
							<p class="text-center text-sm text-gray-600">
								Need a new verification link? Sign in with your credentials and we'll send you a new
								one.
							</p>
							<Button onclick={goToLogin} class="w-full">Go to Login</Button>
						</div>
					{:else}
						<div class="space-y-3">
							<p class="text-center text-sm text-gray-600">
								If you continue to have issues, please contact support.
							</p>
							<Button onclick={goToLogin} variant="outline" class="w-full">Back to Login</Button>
						</div>
					{/if}
				{/if}
			</CardContent>
		</Card>

		<div class="mt-4 text-center">
			<p class="text-sm text-gray-600">
				Need help?
<<<<<<< HEAD
				<a href="/login" class="font-medium text-blue-600 underline hover:text-blue-500">
					Back to Login
=======
				<a href="/support" class="font-medium text-blue-600 underline hover:text-blue-500">
					Contact Support
>>>>>>> aa67ac9 (feat(frontend): Add Email Verification UI [TaskID: 08.02.06])
				</a>
			</p>
		</div>
	</div>
</div>
