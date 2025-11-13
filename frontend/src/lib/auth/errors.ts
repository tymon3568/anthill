import { error } from '@sveltejs/kit';
import { redirect } from '@sveltejs/kit';

export enum AuthErrorCode {
	INVALID_TOKEN = 'INVALID_TOKEN',
	TOKEN_EXPIRED = 'TOKEN_EXPIRED',
	SESSION_EXPIRED = 'SESSION_EXPIRED',
	REFRESH_FAILED = 'REFRESH_FAILED',
	NO_SESSION = 'NO_SESSION',
	UNAUTHORIZED = 'UNAUTHORIZED',
	FORBIDDEN = 'FORBIDDEN',
	INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',
	USER_NOT_FOUND = 'USER_NOT_FOUND',
	TENANT_NOT_FOUND = 'TENANT_NOT_FOUND',
	PERMISSION_DENIED = 'PERMISSION_DENIED',
	PROFILE_FETCH_FAILED = 'PROFILE_FETCH_FAILED',
	PROFILE_UPDATE_FAILED = 'PROFILE_UPDATE_FAILED',
	NETWORK_ERROR = 'NETWORK_ERROR'
}

export class AuthError extends Error {
	public readonly code: AuthErrorCode;
	public readonly status: number;

	constructor(code: AuthErrorCode, message: string, status: number = 401) {
		super(message);
		this.name = 'AuthError';
		this.code = code;
		this.status = status;
	}
}

export function createAuthError(code: AuthErrorCode, message?: string): AuthError {
	const defaultMessages: Record<AuthErrorCode, string> = {
		[AuthErrorCode.INVALID_TOKEN]: 'Invalid authentication token',
		[AuthErrorCode.TOKEN_EXPIRED]: 'Authentication token has expired',
		[AuthErrorCode.SESSION_EXPIRED]: 'Session has expired',
		[AuthErrorCode.REFRESH_FAILED]: 'Failed to refresh authentication token',
		[AuthErrorCode.NO_SESSION]: 'No active session found',
		[AuthErrorCode.UNAUTHORIZED]: 'Authentication required',
		[AuthErrorCode.FORBIDDEN]: 'Access forbidden',
		[AuthErrorCode.INVALID_CREDENTIALS]: 'Invalid credentials provided',
		[AuthErrorCode.USER_NOT_FOUND]: 'User not found',
		[AuthErrorCode.TENANT_NOT_FOUND]: 'Tenant not found',
		[AuthErrorCode.PERMISSION_DENIED]: 'Permission denied',
		[AuthErrorCode.PROFILE_FETCH_FAILED]: 'Failed to fetch user profile',
		[AuthErrorCode.PROFILE_UPDATE_FAILED]: 'Failed to update user profile',
		[AuthErrorCode.NETWORK_ERROR]: 'Network error occurred'
	};

	return new AuthError(code, message || defaultMessages[code]);
}

export function handleAuthError(authError: AuthError | Error | string, redirectTo?: string): never {
	let errorToHandle: AuthError;

	if (authError instanceof AuthError) {
		errorToHandle = authError;
	} else if (authError instanceof Error) {
		// Convert generic Error to AuthError
		errorToHandle = createAuthError(AuthErrorCode.NETWORK_ERROR, authError.message);
	} else if (typeof authError === 'string') {
		// Convert string to AuthError
		errorToHandle = createAuthError(AuthErrorCode.NETWORK_ERROR, authError);
	} else {
		// Unknown error type
		errorToHandle = createAuthError(AuthErrorCode.NETWORK_ERROR, 'An unexpected error occurred');
	}

	if (redirectTo) {
		// Add error details as query parameters
		// URLSearchParams automatically encodes, don't double-encode
		const url = new URL(redirectTo, 'http://localhost'); // Use dummy base for URL construction
		url.searchParams.set('error', errorToHandle.code.toLowerCase());
		url.searchParams.set('message', errorToHandle.message); // URLSearchParams handles encoding

		// Construct final redirect path (only use pathname + search, not full URL)
		const redirectPath = redirectTo.startsWith('http')
			? url.toString()
			: `${url.pathname}${url.search}`;

		throw redirect(302, redirectPath);
	}

	throw error(errorToHandle.status, errorToHandle.message);
}
