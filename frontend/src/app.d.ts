// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			user?: import('$lib/auth/jwt').UserInfo;
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
		PUBLIC_KANIDM_CLIENT_ID?: string;
		PUBLIC_KANIDM_ISSUER_URL?: string;
		PUBLIC_KANIDM_REDIRECT_URI?: string;
	};
}

declare module '$env/static/public' {
	export const PUBLIC_API_BASE_URL: string;
	export const PUBLIC_APP_ENV: string;
	export const PUBLIC_KANIDM_CLIENT_ID: string;
	export const PUBLIC_KANIDM_ISSUER_URL: string;
	export const PUBLIC_KANIDM_REDIRECT_URI: string;
}

export {};
