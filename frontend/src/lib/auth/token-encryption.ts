/**
 * Token encryption utilities for secure storage
 * Note: This is a simplified implementation. In production,
 * use proper encryption with a secure key management system.
 */

// Simple XOR encryption for demonstration (NOT secure for production!)
const ENCRYPTION_KEY = 'anthill-secure-key-2024'; // In production, use environment variable

export function encryptToken(token: string): string {
	try {
		// Simple XOR encryption (for demo only - NOT secure!)
		let result = '';
		for (let i = 0; i < token.length; i++) {
			const charCode = token.charCodeAt(i) ^ ENCRYPTION_KEY.charCodeAt(i % ENCRYPTION_KEY.length);
			result += String.fromCharCode(charCode);
		}
		// Convert to base64 for safe storage
		return btoa(result);
	} catch (error) {
		console.error('Token encryption failed:', error);
		return token; // Fallback to unencrypted
	}
}

export function decryptToken(encryptedToken: string): string {
	try {
		// Decode from base64
		const encrypted = atob(encryptedToken);
		// XOR decryption
		let result = '';
		for (let i = 0; i < encrypted.length; i++) {
			const charCode =
				encrypted.charCodeAt(i) ^ ENCRYPTION_KEY.charCodeAt(i % ENCRYPTION_KEY.length);
			result += String.fromCharCode(charCode);
		}
		return result;
	} catch (error) {
		console.error('Token decryption failed:', error);
		return encryptedToken; // Fallback to encrypted value
	}
}
