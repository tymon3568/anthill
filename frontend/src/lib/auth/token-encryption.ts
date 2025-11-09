// Browser Web Crypto API
const crypto = globalThis.crypto;
export function generateDeviceFingerprint(): string {
	const canvas = document.createElement('canvas');
	const ctx = canvas.getContext('2d');
	ctx?.fillText('fingerprint', 10, 10);

	const fingerprint = [
		navigator.userAgent,
		navigator.language,
		screen.width + 'x' + screen.height,
		new Date().getTimezoneOffset(),
		!!window.sessionStorage,
		!!window.localStorage,
		canvas.toDataURL()
	].join('|');

	return btoa(fingerprint);
}

// Derive encryption key from device fingerprint
export async function deriveKeyFromFingerprint(fingerprint: string): Promise<CryptoKey> {
	const encoder = new TextEncoder();
	const keyMaterial = await crypto.subtle.importKey(
		'raw',
		encoder.encode(fingerprint),
		'PBKDF2',
		false,
		['deriveKey']
	);

	return crypto.subtle.deriveKey(
		{
			name: 'PBKDF2',
			salt: encoder.encode('anthill-salt'),
			iterations: 100000,
			hash: 'SHA-256'
		},
		keyMaterial,
		{ name: 'AES-GCM', length: 256 },
		false,
		['encrypt', 'decrypt']
	);
}

// Encrypt data using AES-GCM
export async function encryptData(data: string, key: CryptoKey): Promise<string> {
	const encoder = new TextEncoder();
	const iv = crypto.getRandomValues(new Uint8Array(12));

	const encrypted = await crypto.subtle.encrypt(
		{ name: 'AES-GCM', iv },
		key,
		encoder.encode(data)
	);

	// Combine IV and encrypted data
	const combined = new Uint8Array(iv.length + encrypted.byteLength);
	combined.set(iv);
	combined.set(new Uint8Array(encrypted), iv.length);

	return btoa(String.fromCharCode(...combined));
}

// Decrypt data using AES-GCM
export async function decryptData(encryptedData: string, key: CryptoKey): Promise<string> {
	try {
		const combined = new Uint8Array(atob(encryptedData).split('').map(c => c.charCodeAt(0)));
		const iv = combined.slice(0, 12);
		const encrypted = combined.slice(12);

		const decrypted = await crypto.subtle.decrypt(
			{ name: 'AES-GCM', iv },
			key,
			encrypted
		);

		const decoder = new TextDecoder();
		return decoder.decode(decrypted);
	} catch (error) {
		throw new Error('Failed to decrypt data - invalid key or corrupted data');
	}
}

// Get or create encryption key for current device
let encryptionKey: CryptoKey | null = null;
export async function getEncryptionKey(): Promise<CryptoKey> {
	if (!encryptionKey) {
		const fingerprint = generateDeviceFingerprint();
		encryptionKey = await deriveKeyFromFingerprint(fingerprint);
	}
	return encryptionKey;
}

// Encrypt token for storage
export async function encryptToken(token: string): Promise<string> {
	const key = await getEncryptionKey();
	return encryptData(token, key);
}

// Decrypt token from storage
export async function decryptToken(encryptedToken: string): Promise<string> {
	const key = await getEncryptionKey();
	return decryptData(encryptedToken, key);
}
