/**
 * Client-side rate limiter for auth endpoints
 *
 * Production standards:
 * - Track attempts per IP/key (client-side uses identifier like 'login')
 * - Window: 15 minutes (900 seconds)
 * - Max attempts: 5
 * - Persist to sessionStorage for refresh persistence
 * - Show countdown when blocked
 */

import { browser } from '$app/environment';

interface RateLimitEntry {
	attempts: number;
	windowStart: number; // timestamp in ms
	blockedUntil?: number; // timestamp in ms
}

const MAX_ATTEMPTS = 5;
const WINDOW_MS = 15 * 60 * 1000; // 15 minutes
const BLOCK_DURATION_MS = 15 * 60 * 1000; // Block for 15 minutes after max attempts
const STORAGE_KEY_PREFIX = 'rate_limit_';

export class RateLimiter {
	private storage: Map<string, RateLimitEntry> = new Map();
	private cleanupInterval?: ReturnType<typeof setInterval>;

	constructor() {
		if (browser) {
			this.loadFromStorage();
			// Cleanup expired entries every 5 minutes
			this.cleanupInterval = setInterval(() => this.cleanup(), 5 * 60 * 1000);
		}
	}

	/**
	 * Load rate limit data from sessionStorage
	 */
	private loadFromStorage(): void {
		try {
			Object.keys(sessionStorage).forEach((key) => {
				if (key.startsWith(STORAGE_KEY_PREFIX)) {
					const identifier = key.replace(STORAGE_KEY_PREFIX, '');
					const data = sessionStorage.getItem(key);
					if (data) {
						const entry: RateLimitEntry = JSON.parse(data);
						this.storage.set(identifier, entry);
					}
				}
			});
		} catch (error) {
			console.error('Failed to load rate limit data:', error);
		}
	}

	/**
	 * Save rate limit data to sessionStorage
	 */
	private saveToStorage(identifier: string, entry: RateLimitEntry): void {
		if (!browser) return;

		try {
			sessionStorage.setItem(
				`${STORAGE_KEY_PREFIX}${identifier}`,
				JSON.stringify(entry)
			);
		} catch (error) {
			console.error('Failed to save rate limit data:', error);
		}
	}

	/**
	 * Remove expired entries from storage
	 */
	private cleanup(): void {
		const now = Date.now();

		this.storage.forEach((entry, identifier) => {
			// Remove if window expired and not blocked
			if (!entry.blockedUntil && now - entry.windowStart > WINDOW_MS) {
				this.storage.delete(identifier);
				if (browser) {
					sessionStorage.removeItem(`${STORAGE_KEY_PREFIX}${identifier}`);
				}
			}
			// Remove if block expired
			else if (entry.blockedUntil && now > entry.blockedUntil) {
				this.storage.delete(identifier);
				if (browser) {
					sessionStorage.removeItem(`${STORAGE_KEY_PREFIX}${identifier}`);
				}
			}
		});
	}

	/**
	 * Check if an attempt is allowed
	 * @returns true if allowed, false if rate limited
	 */
	isAllowed(identifier: string): boolean {
		const now = Date.now();
		const entry = this.storage.get(identifier);

		// No entry, first attempt
		if (!entry) {
			this.recordAttempt(identifier);
			return true;
		}

		// Check if currently blocked
		if (entry.blockedUntil && now < entry.blockedUntil) {
			return false;
		}

		// Check if window expired, reset
		if (now - entry.windowStart > WINDOW_MS) {
			this.reset(identifier);
			this.recordAttempt(identifier);
			return true;
		}

		// Check if max attempts reached
		if (entry.attempts >= MAX_ATTEMPTS) {
			// Block for additional time
			entry.blockedUntil = now + BLOCK_DURATION_MS;
			this.saveToStorage(identifier, entry);
			return false;
		}

		// Within window and under limit
		this.recordAttempt(identifier);
		return true;
	}

	/**
	 * Record an attempt
	 */
	private recordAttempt(identifier: string): void {
		const now = Date.now();
		const entry = this.storage.get(identifier);

		if (!entry) {
			const newEntry: RateLimitEntry = {
				attempts: 1,
				windowStart: now
			};
			this.storage.set(identifier, newEntry);
			this.saveToStorage(identifier, newEntry);
		} else {
			entry.attempts++;
			this.saveToStorage(identifier, entry);
		}
	}

	/**
	 * Get remaining time in seconds if blocked
	 * @returns seconds remaining, or 0 if not blocked
	 */
	getBlockedTimeRemaining(identifier: string): number {
		const entry = this.storage.get(identifier);
		if (!entry || !entry.blockedUntil) return 0;

		const now = Date.now();
		const remaining = Math.max(0, entry.blockedUntil - now);
		return Math.ceil(remaining / 1000); // Convert to seconds
	}

	/**
	 * Get number of attempts remaining before block
	 * @returns attempts remaining, or 0 if blocked
	 */
	getRemainingAttempts(identifier: string): number {
		const entry = this.storage.get(identifier);
		if (!entry) return MAX_ATTEMPTS;

		const now = Date.now();

		// If blocked
		if (entry.blockedUntil && now < entry.blockedUntil) {
			return 0;
		}

		// If window expired
		if (now - entry.windowStart > WINDOW_MS) {
			return MAX_ATTEMPTS;
		}

		return Math.max(0, MAX_ATTEMPTS - entry.attempts);
	}

	/**
	 * Reset rate limit for identifier
	 */
	reset(identifier: string): void {
		this.storage.delete(identifier);
		if (browser) {
			sessionStorage.removeItem(`${STORAGE_KEY_PREFIX}${identifier}`);
		}
	}

	/**
	 * Destroy cleanup interval on component unmount
	 */
	destroy(): void {
		if (this.cleanupInterval) {
			clearInterval(this.cleanupInterval);
		}
	}
}

// Singleton instance
export const rateLimiter = new RateLimiter();

// Format seconds into human-readable time
export function formatBlockedTime(seconds: number): string {
	if (seconds < 60) {
		return `${seconds} second${seconds !== 1 ? 's' : ''}`;
	}

	const minutes = Math.ceil(seconds / 60);
	return `${minutes} minute${minutes !== 1 ? 's' : ''}`;
}
