/// <reference types="vitest" />

// Mock SvelteKit imports for server-side testing
import { vi } from 'vitest';

vi.mock('$app/environment', () => ({
	browser: false
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$app/stores', () => ({
	page: vi.fn()
}));

// Mock browser APIs for token encryption testing
global.window = global.window || {};

// Mock crypto with proper implementations for testing
const mockCryptoKey = {};
const mockEncryptedData = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]); // 16 bytes for IV + encrypted data

Object.defineProperty(window, 'crypto', {
	value: {
		subtle: {
			importKey: vi.fn().mockResolvedValue(mockCryptoKey),
			deriveKey: vi.fn().mockResolvedValue(mockCryptoKey),
			encrypt: vi.fn().mockImplementation(async (algorithm, key, data) => {
				// Return IV (12 bytes) + encrypted data
				const iv = algorithm.iv;
				const encrypted = new TextEncoder().encode('encrypted-');
				const combined = new Uint8Array(iv.length + encrypted.length);
				combined.set(iv);
				combined.set(encrypted, iv.length);
				return combined;
			}),
			decrypt: vi.fn().mockImplementation(async (algorithm, key, data) => {
				// Return the decrypted data
				return new TextEncoder().encode('decrypted-data');
			})
		},
		getRandomValues: vi.fn((array) => {
			// Fill with predictable values for testing
			for (let i = 0; i < array.length; i++) {
				array[i] = i % 256;
			}
			return array;
		})
	}
});

Object.defineProperty(window, 'localStorage', {
	value: {
		getItem: vi.fn(),
		setItem: vi.fn(),
		removeItem: vi.fn(),
		clear: vi.fn()
	}
});

Object.defineProperty(window, 'sessionStorage', {
	value: {
		getItem: vi.fn(),
		setItem: vi.fn(),
		removeItem: vi.fn(),
		clear: vi.fn()
	}
});

// Also define global sessionStorage for direct access
global.sessionStorage = window.sessionStorage;

// Mock document and navigator
Object.defineProperty(window, 'document', {
	value: {
		createElement: vi.fn(() => ({
			getContext: vi.fn(() => ({
				fillText: vi.fn()
			})),
			toDataURL: vi.fn(() => 'mock-canvas-data')
		}))
	}
});

Object.defineProperty(window, 'navigator', {
	value: {
		userAgent: 'test-user-agent',
		language: 'en-US'
	}
});

Object.defineProperty(window, 'screen', {
	value: {
		width: 1920,
		height: 1080
	}
});
