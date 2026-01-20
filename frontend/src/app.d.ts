// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			user?: import('$lib/auth/jwt').UserInfo;
			/** Tenant slug detected from subdomain or X-Tenant-ID header */
			tenantSlug?: string | null;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

// Environment variables
declare module '$env/dynamic/public' {
	export const env: {
		PUBLIC_API_BASE_URL?: string;
		PUBLIC_APP_ENV?: string;
		PUBLIC_USER_SERVICE_URL?: string;
	};
}

declare module '$env/static/public' {
	export const PUBLIC_API_BASE_URL: string;
	export const PUBLIC_APP_ENV: string;
	export const PUBLIC_USER_SERVICE_URL: string;
}

export {};
