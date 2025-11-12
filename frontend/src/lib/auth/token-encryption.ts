/**
 * Token encryption utilities
 *
 * SECURITY WARNING: This module is DEPRECATED and should not be used.
 *
 * Tokens should NEVER be stored client-side with any form of "encryption":
 * - XOR encryption is trivially reversible
 * - Client-side encryption keys are visible in source code
 * - Any client-side storage (localStorage/sessionStorage) is vulnerable to XSS
 *
 * PROPER APPROACH:
 * - Tokens should be stored in httpOnly cookies set by the server
 * - This prevents JavaScript access and XSS attacks
 * - The browser automatically includes httpOnly cookies with requests
 *
 * This file is kept for backwards compatibility only.
 */

/**
 * @deprecated Do not use. Tokens should be in httpOnly cookies, not encrypted client-side.
 */
export function encryptToken(token: string): string {
	console.error('encryptToken is deprecated: Store tokens in httpOnly cookies instead');
	// Return as-is, no fake security
	return token;
}

/**
 * @deprecated Do not use. Tokens should be in httpOnly cookies, not encrypted client-side.
 */
export function decryptToken(encryptedToken: string): string {
	console.error('decryptToken is deprecated: Store tokens in httpOnly cookies instead');
	// Return as-is, no fake security
	return encryptedToken;
}
