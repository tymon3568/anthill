interface RateLimitConfig {
	maxAttempts: number;
	windowMs: number; // Time window in milliseconds
	blockDurationMs: number; // How long to block after limit exceeded
}

class RateLimiter {
	private attempts: Map<string, { count: number; resetTime: number; blockedUntil?: number }> =
		new Map();

	private configs: Map<string, RateLimitConfig> = new Map();

	constructor() {
		// Default config for registration
		this.configs.set('register', {
			maxAttempts: 5,
			windowMs: 15 * 60 * 1000, // 15 minutes
			blockDurationMs: 30 * 60 * 1000 // 30 minutes
		});

		// Default config for login
		this.configs.set('login', {
			maxAttempts: 5,
			windowMs: 15 * 60 * 1000, // 15 minutes
			blockDurationMs: 15 * 60 * 1000 // 15 minutes
		});
	}

	isAllowed(key: string): boolean {
		const config = this.configs.get(key) || this.configs.get('register')!;
		const now = Date.now();
		const record = this.attempts.get(key);

		if (!record) {
			return true;
		}

		// Check if window has expired - if so, reset
		if (now > record.resetTime) {
			this.attempts.delete(key);
			return true;
		}

		// Check if currently blocked
		if (record.blockedUntil && now < record.blockedUntil) {
			return false;
		}

		// Check if under limit
		return record.count < config.maxAttempts;
	}

	recordAttempt(key: string): void {
		const config = this.configs.get(key) || this.configs.get('register')!;
		const now = Date.now();
		const record = this.attempts.get(key);

		if (!record || now > record.resetTime) {
			// Create new record
			this.attempts.set(key, {
				count: 1,
				resetTime: now + config.windowMs
			});
		} else {
			// Increment existing record
			record.count++;

			// Check if limit exceeded
			if (record.count >= config.maxAttempts) {
				record.blockedUntil = now + config.blockDurationMs;
			}
		}
	}

	getRemainingAttempts(key: string): number {
		const config = this.configs.get(key) || this.configs.get('register')!;
		const record = this.attempts.get(key);

		if (!record) {
			return config.maxAttempts;
		}

		const now = Date.now();

		// If blocked, no attempts remaining
		if (record.blockedUntil && now < record.blockedUntil) {
			return 0;
		}

		// If window expired, reset
		if (now > record.resetTime) {
			return config.maxAttempts;
		}

		return Math.max(0, config.maxAttempts - record.count);
	}

	getBlockedTimeRemaining(key: string): number {
		const record = this.attempts.get(key);

		if (!record || !record.blockedUntil) {
			return 0;
		}

		const remaining = record.blockedUntil - Date.now();
		return Math.max(0, remaining);
	}

	reset(key: string): void {
		this.attempts.delete(key);
	}
}

// Singleton instance
export const rateLimiter = new RateLimiter();

export function formatBlockedTime(ms: number): string {
	if (ms <= 0) return '0 seconds';

	const seconds = Math.ceil(ms / 1000);
	const minutes = Math.floor(seconds / 60);
	const remainingSeconds = seconds % 60;

	if (minutes > 0) {
		return `${minutes} minute${minutes > 1 ? 's' : ''}${remainingSeconds > 0 ? ` ${remainingSeconds} second${remainingSeconds > 1 ? 's' : ''}` : ''}`;
	}

	return `${remainingSeconds} second${remainingSeconds > 1 ? 's' : ''}`;
}
