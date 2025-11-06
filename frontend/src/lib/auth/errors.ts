import { redirect } from '@sveltejs/kit';

export class AuthError extends Error {
	constructor(
		message: string,
		public code: AuthErrorCode,
		public statusCode: number = 500
	) {
		super(message);
		this.name = 'AuthError';
	}
}

export enum AuthErrorCode {
	// OAuth2 errors
	INVALID_CODE = 'invalid_code',
	INVALID_STATE = 'invalid_state',
	MISSING_VERIFIER = 'missing_verifier',
	TOKEN_EXCHANGE_FAILED = 'token_exchange_failed',
	INVALID_TOKEN = 'invalid_token',
	TOKEN_EXPIRED = 'token_expired',
	REFRESH_FAILED = 'refresh_failed',

	// Permission errors
	UNAUTHORIZED = 'unauthorized',
	FORBIDDEN = 'forbidden',
	INSUFFICIENT_PERMISSIONS = 'insufficient_permissions',

	// Session errors
	SESSION_EXPIRED = 'session_expired',
	NO_SESSION = 'no_session',

	// Network errors
	NETWORK_ERROR = 'network_error',
	KANIDM_UNAVAILABLE = 'kanidm_unavailable',

	// Generic errors
	UNKNOWN_ERROR = 'unknown_error'
}

export function handleAuthError(error: unknown, redirectTo: string = '/login'): never {
	console.error('Auth error:', error);

	let errorCode = AuthErrorCode.UNKNOWN_ERROR;
	let errorMessage = 'An unexpected error occurred';
	let statusCode = 500;

	if (error instanceof AuthError) {
		errorCode = error.code;
		errorMessage = error.message;
		statusCode = error.statusCode;
	} else if (error instanceof Error) {
		// Map common error messages to auth error codes
		if (error.message.includes('fetch')) {
			errorCode = AuthErrorCode.NETWORK_ERROR;
			errorMessage = 'Network error occurred';
			statusCode = 503;
		} else if (error.message.includes('token')) {
			errorCode = AuthErrorCode.INVALID_TOKEN;
			errorMessage = 'Invalid or expired token';
			statusCode = 401;
		} else {
			errorMessage = error.message;
		}
	}

	// Build redirect URL with error parameters
	const redirectUrl = new URL(redirectTo, 'http://placeholder');
	redirectUrl.searchParams.set('error', errorCode);
	redirectUrl.searchParams.set('message', encodeURIComponent(errorMessage));

	// Use pathname and search params to build relative URL
	throw redirect(302, `${redirectUrl.pathname}${redirectUrl.search}`);
}

export function createAuthError(code: AuthErrorCode, message?: string): AuthError {
	const defaultMessages: Record<AuthErrorCode, string> = {
		[AuthErrorCode.INVALID_CODE]: 'Invalid authorization code',
		[AuthErrorCode.INVALID_STATE]: 'Invalid state parameter',
		[AuthErrorCode.MISSING_VERIFIER]: 'Missing code verifier',
		[AuthErrorCode.TOKEN_EXCHANGE_FAILED]: 'Failed to exchange authorization code for tokens',
		[AuthErrorCode.INVALID_TOKEN]: 'Invalid or malformed token',
		[AuthErrorCode.TOKEN_EXPIRED]: 'Token has expired',
		[AuthErrorCode.REFRESH_FAILED]: 'Failed to refresh access token',
		[AuthErrorCode.UNAUTHORIZED]: 'Authentication required',
		[AuthErrorCode.FORBIDDEN]: 'Access forbidden',
		[AuthErrorCode.INSUFFICIENT_PERMISSIONS]: 'Insufficient permissions',
		[AuthErrorCode.SESSION_EXPIRED]: 'Session has expired',
		[AuthErrorCode.NO_SESSION]: 'No active session',
		[AuthErrorCode.NETWORK_ERROR]: 'Network communication error',
		[AuthErrorCode.KANIDM_UNAVAILABLE]: 'Authentication service unavailable',
		[AuthErrorCode.UNKNOWN_ERROR]: 'Unknown authentication error'
	};

	const statusCodes: Record<AuthErrorCode, number> = {
		[AuthErrorCode.INVALID_CODE]: 400,
		[AuthErrorCode.INVALID_STATE]: 400,
		[AuthErrorCode.MISSING_VERIFIER]: 400,
		[AuthErrorCode.TOKEN_EXCHANGE_FAILED]: 502,
		[AuthErrorCode.INVALID_TOKEN]: 401,
		[AuthErrorCode.TOKEN_EXPIRED]: 401,
		[AuthErrorCode.REFRESH_FAILED]: 502,
		[AuthErrorCode.UNAUTHORIZED]: 401,
		[AuthErrorCode.FORBIDDEN]: 403,
		[AuthErrorCode.INSUFFICIENT_PERMISSIONS]: 403,
		[AuthErrorCode.SESSION_EXPIRED]: 401,
		[AuthErrorCode.NO_SESSION]: 401,
		[AuthErrorCode.NETWORK_ERROR]: 503,
		[AuthErrorCode.KANIDM_UNAVAILABLE]: 503,
		[AuthErrorCode.UNKNOWN_ERROR]: 500
	};

	return new AuthError(
		message || defaultMessages[code],
		code,
		statusCodes[code]
	);
}
