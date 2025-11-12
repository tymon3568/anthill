import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { rateLimiter, formatBlockedTime } from './rate-limiter';

describe('Rate Limiter', () => {
	beforeEach(() => {
		// Reset rate limiter state
		rateLimiter.reset('register');
		rateLimiter.reset('login');
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('isAllowed', () => {
		it('should allow requests within limit', () => {
			expect(rateLimiter.isAllowed('register')).toBe(true);
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.isAllowed('register')).toBe(true);
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.isAllowed('register')).toBe(true);
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.isAllowed('register')).toBe(true);
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.isAllowed('register')).toBe(true);
		});

		it('should block requests over limit', () => {
			// Make 5 attempts (the limit)
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			expect(rateLimiter.isAllowed('register')).toBe(false);
		});

		it('should allow requests after window expires', () => {
			// Make 5 attempts
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			expect(rateLimiter.isAllowed('register')).toBe(false);

			// Fast forward past the window (15 minutes + 1 second)
			vi.advanceTimersByTime(15 * 60 * 1000 + 1000);

			expect(rateLimiter.isAllowed('register')).toBe(true);
		});

		it('should use different configs for different keys', () => {
			// Login allows 5 attempts, register allows 5 attempts
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('login');
				rateLimiter.recordAttempt('register');
			}

			expect(rateLimiter.isAllowed('login')).toBe(false);
			expect(rateLimiter.isAllowed('register')).toBe(false);
		});
	});

	describe('recordAttempt', () => {
		it('should increment attempt count', () => {
			expect(rateLimiter.getRemainingAttempts('register')).toBe(5);

			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(4);

			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(3);
		});

		it('should reset window after time passes', () => {
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(4);

			// Fast forward past the window
			vi.advanceTimersByTime(15 * 60 * 1000 + 1000);

			expect(rateLimiter.getRemainingAttempts('register')).toBe(5);
		});
	});

	describe('getRemainingAttempts', () => {
		it('should return correct remaining attempts', () => {
			expect(rateLimiter.getRemainingAttempts('register')).toBe(5);

			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(4);

			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(3);
		});

		it('should return 0 when blocked', () => {
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			expect(rateLimiter.getRemainingAttempts('register')).toBe(0);
		});
	});

	describe('getBlockedTimeRemaining', () => {
		it('should return 0 when not blocked', () => {
			expect(rateLimiter.getBlockedTimeRemaining('register')).toBe(0);
		});

		it('should return remaining block time when blocked', () => {
			// Make 5 attempts to trigger blocking
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			const remaining = rateLimiter.getBlockedTimeRemaining('register');
			expect(remaining).toBeGreaterThan(0);
			expect(remaining).toBeLessThanOrEqual(30 * 60 * 1000); // 30 minutes
		});

		it('should decrease over time', () => {
			// Make 5 attempts to trigger blocking
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			const initialRemaining = rateLimiter.getBlockedTimeRemaining('register');
			expect(initialRemaining).toBeGreaterThan(0);

			// Advance time by 1 minute
			vi.advanceTimersByTime(60 * 1000);

			const laterRemaining = rateLimiter.getBlockedTimeRemaining('register');
			expect(laterRemaining).toBeLessThan(initialRemaining);
		});
	});

	describe('reset', () => {
		it('should reset attempt counter', () => {
			rateLimiter.recordAttempt('register');
			rateLimiter.recordAttempt('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(3);

			rateLimiter.reset('register');
			expect(rateLimiter.getRemainingAttempts('register')).toBe(5);
		});

		it('should reset block status', () => {
			// Make 5 attempts to trigger blocking
			for (let i = 0; i < 5; i++) {
				rateLimiter.recordAttempt('register');
			}

			expect(rateLimiter.isAllowed('register')).toBe(false);

			rateLimiter.reset('register');
			expect(rateLimiter.isAllowed('register')).toBe(true);
		});
	});
});

describe('formatBlockedTime', () => {
	it('should format seconds correctly', () => {
		expect(formatBlockedTime(0)).toBe('0 seconds');
		expect(formatBlockedTime(1000)).toBe('1 second');
		expect(formatBlockedTime(59000)).toBe('59 seconds');
	});

	it('should format minutes correctly', () => {
		expect(formatBlockedTime(60000)).toBe('1 minute');
		expect(formatBlockedTime(120000)).toBe('2 minutes');
		expect(formatBlockedTime(3599000)).toBe('59 minutes 59 seconds');
	});

	it('should handle edge cases', () => {
		expect(formatBlockedTime(-1000)).toBe('0 seconds');
		expect(formatBlockedTime(1000 * 60 * 60 * 24)).toBe('1440 minutes'); // 1 day
	});
});
