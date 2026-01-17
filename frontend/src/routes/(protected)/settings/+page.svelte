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
	import { Textarea } from '$lib/components/ui/textarea';
	import { Switch } from '$lib/components/ui/switch';
	import { Progress } from '$lib/components/ui/progress';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		UserProfile,
		VisibilitySettings,
		CompletenessScore,
		UpdateProfileRequest,
		ProfileVisibility
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';

	// Settings sections
	let activeSection = $state('profile');

	// Loading states
	let isLoadingProfile = $state(true);
	let isLoadingCompleteness = $state(true);
	let isSaving = $state(false);
	let isUploadingAvatar = $state(false);

	// Profile data from API
	let profile = $state<UserProfile | null>(null);
	let completeness = $state<CompletenessScore | null>(null);

	// Profile form state
	let profileForm = $state({
		fullName: '',
		bio: '',
		title: '',
		department: '',
		location: '',
		websiteUrl: '',
		timezone: 'UTC',
		language: 'en'
	});

	// Social links
	let socialLinks = $state({
		linkedin: '',
		github: '',
		twitter: ''
	});

	// Visibility settings
	let visibility = $state<VisibilitySettings>({
		profileVisibility: 'team_only',
		showEmail: true,
		showPhone: false
	});

	// Notification settings
	let notifications = $state({
		emailOrders: true,
		emailInventory: true,
		emailMarketing: false,
		pushOrders: true,
		pushInventory: false
	});

	// Avatar file input ref
	let avatarInput = $state<HTMLInputElement | null>(null);

	// Timezones list
	const timezones = [
		{ value: 'UTC', label: 'UTC' },
		{ value: 'America/New_York', label: 'Eastern Time (US)' },
		{ value: 'America/Chicago', label: 'Central Time (US)' },
		{ value: 'America/Denver', label: 'Mountain Time (US)' },
		{ value: 'America/Los_Angeles', label: 'Pacific Time (US)' },
		{ value: 'Europe/London', label: 'London (GMT)' },
		{ value: 'Europe/Paris', label: 'Paris (CET)' },
		{ value: 'Europe/Berlin', label: 'Berlin (CET)' },
		{ value: 'Asia/Tokyo', label: 'Tokyo (JST)' },
		{ value: 'Asia/Shanghai', label: 'Shanghai (CST)' },
		{ value: 'Asia/Singapore', label: 'Singapore (SGT)' },
		{ value: 'Asia/Ho_Chi_Minh', label: 'Ho Chi Minh (ICT)' },
		{ value: 'Australia/Sydney', label: 'Sydney (AEST)' }
	];

	// Languages list
	const languages = [
		{ value: 'en', label: 'English' },
		{ value: 'vi', label: 'Tiếng Việt' },
		{ value: 'es', label: 'Español' },
		{ value: 'fr', label: 'Français' },
		{ value: 'de', label: 'Deutsch' },
		{ value: 'ja', label: '日本語' },
		{ value: 'zh', label: '中文' }
	];

	// Visibility options
	const visibilityOptions = [
		{ value: 'public', label: 'Public - Anyone can view' },
		{ value: 'team_only', label: 'Team Only - Only team members' },
		{ value: 'private', label: 'Private - Only you' }
	];

	// Load profile data on mount
	onMount(async () => {
		await Promise.all([loadProfile(), loadCompleteness()]);
	});

	async function loadProfile() {
		isLoadingProfile = true;
		try {
			const response = await userServiceApi.getProfile();
			if (response.success && response.data) {
				profile = response.data;
				// Populate form with profile data
				profileForm = {
					fullName: profile.fullName || '',
					bio: profile.bio || '',
					title: profile.title || '',
					department: profile.department || '',
					location: profile.location || '',
					websiteUrl: profile.websiteUrl || '',
					timezone: profile.timezone || 'UTC',
					language: profile.language || 'en'
				};
				// Populate social links
				if (profile.socialLinks) {
					socialLinks = {
						linkedin: profile.socialLinks.linkedin || '',
						github: profile.socialLinks.github || '',
						twitter: profile.socialLinks.twitter || ''
					};
				}
				// Populate visibility settings
				visibility = {
					profileVisibility: profile.profileVisibility || 'team_only',
					showEmail: profile.showEmail ?? true,
					showPhone: profile.showPhone ?? false
				};
			}
		} catch (error) {
			console.error('Failed to load profile:', error);
			toast.error('Failed to load profile');
		} finally {
			isLoadingProfile = false;
		}
	}

	async function loadCompleteness() {
		isLoadingCompleteness = true;
		try {
			const response = await userServiceApi.getProfileCompleteness();
			if (response.success && response.data) {
				completeness = response.data;
			}
		} catch (error) {
			console.error('Failed to load completeness:', error);
			// Silent fail - completeness widget is optional
		} finally {
			isLoadingCompleteness = false;
		}
	}

	async function saveProfile() {
		isSaving = true;
		try {
			const updateData: UpdateProfileRequest = {
				fullName: profileForm.fullName,
				bio: profileForm.bio || undefined,
				title: profileForm.title || undefined,
				department: profileForm.department || undefined,
				location: profileForm.location || undefined,
				websiteUrl: profileForm.websiteUrl || undefined,
				timezone: profileForm.timezone,
				language: profileForm.language,
				socialLinks: Object.fromEntries(
					Object.entries({
						linkedin: socialLinks.linkedin,
						github: socialLinks.github,
						twitter: socialLinks.twitter
					}).filter(([, v]) => v)
				)
			};

			const response = await userServiceApi.updateProfile(updateData);
			if (response.success) {
				toast.success('Profile updated successfully');
				await loadCompleteness(); // Refresh completeness score
			} else {
				toast.error(response.error || 'Failed to update profile');
			}
		} catch (error) {
			console.error('Failed to save profile:', error);
			toast.error('Failed to save profile');
		} finally {
			isSaving = false;
		}
	}

	async function saveVisibility() {
		isSaving = true;
		try {
			const response = await userServiceApi.updateVisibility(visibility);
			if (response.success) {
				toast.success('Visibility settings updated');
			} else {
				toast.error(response.error || 'Failed to update visibility');
			}
		} catch (error) {
			console.error('Failed to save visibility:', error);
			toast.error('Failed to save visibility settings');
		} finally {
			isSaving = false;
		}
	}

	async function handleAvatarUpload(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		// Validate file size (max 5MB)
		if (file.size > 5 * 1024 * 1024) {
			toast.error('Avatar file must be less than 5MB');
			return;
		}

		// Validate file type
		if (!file.type.startsWith('image/')) {
			toast.error('Please select an image file');
			return;
		}

		isUploadingAvatar = true;
		try {
			const response = await userServiceApi.uploadAvatar(file);
			if (response.success && response.data) {
				if (profile) {
					profile.avatarUrl = response.data.avatarUrl;
				}
				toast.success('Avatar uploaded successfully');
			} else {
				toast.error(response.error || 'Failed to upload avatar');
			}
		} catch (error) {
			console.error('Failed to upload avatar:', error);
			toast.error('Failed to upload avatar');
		} finally {
			isUploadingAvatar = false;
		}
	}

	function getInitials(name: string): string {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('')
			.toUpperCase()
			.slice(0, 2);
	}

	async function saveNotifications() {
		// TODO: Implement API endpoint for notification preferences when backend supports it
		toast.info('Notification preferences saved locally (backend integration pending)');
	}
</script>

<svelte:head>
	<title>Settings - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-bold">Settings</h1>
		<p class="text-muted-foreground">Manage your account and preferences</p>
	</div>

	<div class="flex gap-6">
		<!-- Sidebar -->
		<div class="w-48 space-y-1">
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'profile'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'profile')}
			>
				Profile
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'visibility'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'visibility')}
			>
				Privacy & Visibility
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'notifications'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'notifications')}
			>
				Notifications
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'preferences'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'preferences')}
			>
				Preferences
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'security'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'security')}
			>
				Security
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 space-y-6">
			<!-- Profile Completeness Widget -->
			{#if !isLoadingCompleteness && completeness && completeness.score < 100}
				<Card class="border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950">
					<CardContent class="pt-6">
						<div class="mb-2 flex items-center justify-between">
							<span class="text-sm font-medium">Profile Completeness</span>
							<span class="text-sm font-bold text-blue-600">{completeness.score}%</span>
						</div>
						<Progress value={completeness.score} class="h-2" />
						{#if completeness.recommendations.length > 0}
							<p class="mt-2 text-sm text-muted-foreground">
								{completeness.recommendations[0]}
							</p>
						{/if}
					</CardContent>
				</Card>
			{/if}

			{#if activeSection === 'profile'}
				<!-- Avatar Section -->
				<Card>
					<CardHeader>
						<CardTitle>Profile Picture</CardTitle>
						<CardDescription>Upload a photo to personalize your account</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="flex items-center gap-6">
							<Avatar class="h-24 w-24">
								<AvatarImage src={profile?.avatarUrl} alt={profileForm.fullName} />
								<AvatarFallback class="text-2xl">
									{getInitials(profileForm.fullName || 'U')}
								</AvatarFallback>
							</Avatar>
							<div class="space-y-2">
								<input
									bind:this={avatarInput}
									type="file"
									accept="image/*"
									class="hidden"
									onchange={handleAvatarUpload}
								/>
								<Button
									variant="outline"
									disabled={isUploadingAvatar}
									onclick={() => avatarInput?.click()}
								>
									{isUploadingAvatar ? 'Uploading...' : 'Change Avatar'}
								</Button>
								<p class="text-xs text-muted-foreground">JPG, PNG or GIF. Max size 5MB.</p>
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Basic Info -->
				<Card>
					<CardHeader>
						<CardTitle>Basic Information</CardTitle>
						<CardDescription>Update your personal details</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						{#if isLoadingProfile}
							<div class="animate-pulse space-y-4">
								<div class="h-10 rounded bg-muted"></div>
								<div class="h-10 rounded bg-muted"></div>
								<div class="h-24 rounded bg-muted"></div>
							</div>
						{:else}
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="fullName">Full Name</Label>
									<Input id="fullName" bind:value={profileForm.fullName} placeholder="Your name" />
								</div>
								<div class="space-y-2">
									<Label for="email">Email</Label>
									<Input
										id="email"
										type="email"
										value={profile?.email || ''}
										disabled
										class="bg-muted"
									/>
									<p class="text-xs text-muted-foreground">Email cannot be changed</p>
								</div>
							</div>
							<div class="space-y-2">
								<Label for="bio">Bio</Label>
								<Textarea
									id="bio"
									bind:value={profileForm.bio}
									placeholder="Tell us about yourself..."
									rows={3}
								/>
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="title">Job Title</Label>
									<Input
										id="title"
										bind:value={profileForm.title}
										placeholder="e.g. Product Manager"
									/>
								</div>
								<div class="space-y-2">
									<Label for="department">Department</Label>
									<Input
										id="department"
										bind:value={profileForm.department}
										placeholder="e.g. Engineering"
									/>
								</div>
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-2">
									<Label for="location">Location</Label>
									<Input
										id="location"
										bind:value={profileForm.location}
										placeholder="e.g. Ho Chi Minh City"
									/>
								</div>
								<div class="space-y-2">
									<Label for="website">Website</Label>
									<Input
										id="website"
										bind:value={profileForm.websiteUrl}
										placeholder="https://your-site.com"
									/>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Social Links -->
				<Card>
					<CardHeader>
						<CardTitle>Social Links</CardTitle>
						<CardDescription>Connect your social profiles</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-3 gap-4">
							<div class="space-y-2">
								<Label for="linkedin">LinkedIn</Label>
								<Input
									id="linkedin"
									bind:value={socialLinks.linkedin}
									placeholder="linkedin.com/in/username"
								/>
							</div>
							<div class="space-y-2">
								<Label for="github">GitHub</Label>
								<Input
									id="github"
									bind:value={socialLinks.github}
									placeholder="github.com/username"
								/>
							</div>
							<div class="space-y-2">
								<Label for="twitter">Twitter</Label>
								<Input
									id="twitter"
									bind:value={socialLinks.twitter}
									placeholder="twitter.com/username"
								/>
							</div>
						</div>
					</CardContent>
				</Card>

				<div class="flex justify-end">
					<Button onclick={saveProfile} disabled={isSaving}>
						{isSaving ? 'Saving...' : 'Save Changes'}
					</Button>
				</div>
			{:else if activeSection === 'visibility'}
				<Card>
					<CardHeader>
						<CardTitle>Privacy & Visibility</CardTitle>
						<CardDescription>Control who can see your profile information</CardDescription>
					</CardHeader>
					<CardContent class="space-y-6">
						<div class="space-y-2">
							<Label for="profileVisibility">Profile Visibility</Label>
							<Select.Root
								type="single"
								name="profileVisibility"
								value={visibility.profileVisibility}
								onValueChange={(v) => (visibility.profileVisibility = v as ProfileVisibility)}
							>
								<Select.Trigger class="w-full">
									{visibilityOptions.find((o) => o.value === visibility.profileVisibility)?.label ||
										'Select visibility'}
								</Select.Trigger>
								<Select.Content>
									{#each visibilityOptions as option (option.value)}
										<Select.Item value={option.value}>{option.label}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>

						<Separator />

						<div class="space-y-4">
							<h4 class="font-medium">Field Visibility</h4>
							<div class="flex items-center justify-between">
								<div>
									<Label>Show Email Address</Label>
									<p class="text-sm text-muted-foreground">Allow others to see your email</p>
								</div>
								<Switch bind:checked={visibility.showEmail} />
							</div>
							<div class="flex items-center justify-between">
								<div>
									<Label>Show Phone Number</Label>
									<p class="text-sm text-muted-foreground">Allow others to see your phone</p>
								</div>
								<Switch bind:checked={visibility.showPhone} />
							</div>
						</div>
					</CardContent>
				</Card>

				<div class="flex justify-end">
					<Button onclick={saveVisibility} disabled={isSaving}>
						{isSaving ? 'Saving...' : 'Save Visibility Settings'}
					</Button>
				</div>
			{:else if activeSection === 'notifications'}
				<Card>
					<CardHeader>
						<CardTitle>Notification Preferences</CardTitle>
						<CardDescription>Choose how you want to be notified</CardDescription>
					</CardHeader>
					<CardContent class="space-y-6">
						<div class="space-y-4">
							<h4 class="font-medium">Email Notifications</h4>
							<div class="space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<Label>Order Updates</Label>
										<p class="text-sm text-muted-foreground">
											Get notified about order status changes
										</p>
									</div>
									<Switch bind:checked={notifications.emailOrders} />
								</div>
								<div class="flex items-center justify-between">
									<div>
										<Label>Low Stock Alerts</Label>
										<p class="text-sm text-muted-foreground">Get notified when inventory is low</p>
									</div>
									<Switch bind:checked={notifications.emailInventory} />
								</div>
								<div class="flex items-center justify-between">
									<div>
										<Label>Marketing Emails</Label>
										<p class="text-sm text-muted-foreground">Receive tips, news, and offers</p>
									</div>
									<Switch bind:checked={notifications.emailMarketing} />
								</div>
							</div>
						</div>

						<Separator />

						<div class="space-y-4">
							<h4 class="font-medium">Push Notifications</h4>
							<div class="space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<Label>Order Updates</Label>
										<p class="text-sm text-muted-foreground">Push notifications for orders</p>
									</div>
									<Switch bind:checked={notifications.pushOrders} />
								</div>
								<div class="flex items-center justify-between">
									<div>
										<Label>Low Stock Alerts</Label>
										<p class="text-sm text-muted-foreground">Push notifications for inventory</p>
									</div>
									<Switch bind:checked={notifications.pushInventory} />
								</div>
							</div>
						</div>
					</CardContent>
				</Card>

				<div class="flex justify-end">
					<Button onclick={saveNotifications}>Save Preferences</Button>
				</div>
			{:else if activeSection === 'preferences'}
				<Card>
					<CardHeader>
						<CardTitle>Display Preferences</CardTitle>
						<CardDescription>Customize your experience</CardDescription>
					</CardHeader>
					<CardContent class="space-y-6">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="timezone">Timezone</Label>
								<Select.Root
									type="single"
									name="timezone"
									value={profileForm.timezone}
									onValueChange={(v) => (profileForm.timezone = v)}
								>
									<Select.Trigger class="w-full">
										{timezones.find((t) => t.value === profileForm.timezone)?.label ||
											'Select timezone'}
									</Select.Trigger>
									<Select.Content>
										{#each timezones as tz (tz.value)}
											<Select.Item value={tz.value}>{tz.label}</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							</div>
							<div class="space-y-2">
								<Label for="language">Language</Label>
								<Select.Root
									type="single"
									name="language"
									value={profileForm.language}
									onValueChange={(v) => (profileForm.language = v)}
								>
									<Select.Trigger class="w-full">
										{languages.find((l) => l.value === profileForm.language)?.label ||
											'Select language'}
									</Select.Trigger>
									<Select.Content>
										{#each languages as lang (lang.value)}
											<Select.Item value={lang.value}>{lang.label}</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							</div>
						</div>
					</CardContent>
				</Card>

				<div class="flex justify-end">
					<Button onclick={saveProfile} disabled={isSaving}>
						{isSaving ? 'Saving...' : 'Save Preferences'}
					</Button>
				</div>
			{:else if activeSection === 'security'}
				<Card>
					<CardHeader>
						<CardTitle>Security Settings</CardTitle>
						<CardDescription>Manage your account security</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<div class="flex items-center gap-2">
								<Label>Change Password</Label>
								<span class="rounded bg-muted px-2 py-0.5 text-xs">Coming Soon</span>
							</div>
							<div class="grid gap-2">
								<Input type="password" placeholder="Current password" disabled />
								<Input type="password" placeholder="New password" disabled />
								<Input type="password" placeholder="Confirm new password" disabled />
							</div>
						</div>
						<div class="flex justify-end">
							<Button disabled>Update Password</Button>
						</div>

						<Separator />

						<div class="space-y-2">
							<div class="flex items-center gap-2">
								<h4 class="font-medium">Two-Factor Authentication</h4>
								<span class="rounded bg-muted px-2 py-0.5 text-xs">Coming Soon</span>
							</div>
							<p class="text-sm text-muted-foreground">
								Add an extra layer of security to your account
							</p>
							<Button variant="outline" disabled>Enable 2FA</Button>
						</div>
					</CardContent>
				</Card>
			{/if}
		</div>
	</div>
</div>
