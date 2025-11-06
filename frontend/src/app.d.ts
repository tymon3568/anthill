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
		VITE_API_BASE_URL?: string;
		VITE_APP_ENV?: string;
		VITE_KANIDM_CLIENT_ID?: string;
		VITE_KANIDM_ISSUER_URL?: string;
		VITE_KANIDM_REDIRECT_URI?: string;
	};
}

declare module '$env/static/public' {
	export const VITE_API_BASE_URL: string;
	export const VITE_APP_ENV: string;
	export const VITE_KANIDM_CLIENT_ID: string;
	export const VITE_KANIDM_ISSUER_URL: string;
	export const VITE_KANIDM_REDIRECT_URI: string;
}

export {};
