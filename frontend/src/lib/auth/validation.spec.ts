import { describe, it, expect } from 'vitest';
import { loginSchema, registerSchema, calculatePasswordStrength, validatePasswordConfirmation } from './validation';
import { safeParse } from 'valibot';

describe('Auth Validation', () => {
	describe('loginSchema', () => {
		it('should validate valid login data', () => {
			const validData = {
				email: 'user@example.com',
				password: 'password123'
			};

			const result = safeParse(loginSchema, validData);
			expect(result.success).toBe(true);
			if (result.success) {
				expect(result.output).toEqual(validData);
			}
		});

		it('should reject invalid email', () => {
			const invalidData = {
				email: 'invalid-email',
				password: 'password123'
			};

			const result = safeParse(loginSchema, invalidData);
			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues.some((issue: any) => issue.path?.[0]?.key === 'email')).toBe(true);
			}
		});

		it('should reject password too short', () => {
			const invalidData = {
				email: 'user@example.com',
				password: '123'
			};

			const result = safeParse(loginSchema, invalidData);
			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues.some((issue: any) => issue.path?.[0]?.key === 'password')).toBe(true);
			}
		});

		it('should reject empty email', () => {
			const invalidData = {
				email: '',
				password: 'password123'
			};

			const result = safeParse(loginSchema, invalidData);
			expect(result.success).toBe(false);
		});

		it('should reject empty password', () => {
			const invalidData = {
				email: 'user@example.com',
				password: ''
			};

			const result = safeParse(loginSchema, invalidData);
			expect(result.success).toBe(false);
		});
	});

	describe('registerSchema', () => {
		it('should validate valid registration data', () => {
			const validData = {
				name: 'John Doe',
				email: 'user@example.com',
				password: 'StrongPass123!',
				confirmPassword: 'StrongPass123!'
			};

			const result = safeParse(registerSchema, validData);
			expect(result.success).toBe(true);
			if (result.success) {
				expect(result.output).toEqual(validData);
			}
		});

		it('should reject invalid name', () => {
			const invalidData = {
				name: 'J',
				email: 'user@example.com',
				password: 'StrongPass123!',
				confirmPassword: 'StrongPass123!'
			};

			const result = safeParse(registerSchema, invalidData);
			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues.some((issue: any) => issue.path?.[0]?.key === 'name')).toBe(true);
			}
		});

		it('should reject name with invalid characters', () => {
			const invalidData = {
				name: 'John123',
				email: 'user@example.com',
				password: 'StrongPass123!',
				confirmPassword: 'StrongPass123!'
			};

			const result = safeParse(registerSchema, invalidData);
			expect(result.success).toBe(false);
		});

		it('should reject weak password', () => {
			const invalidData = {
				name: 'John Doe',
				email: 'user@example.com',
				password: 'weak',
				confirmPassword: 'weak'
			};

			const result = safeParse(registerSchema, invalidData);
			expect(result.success).toBe(false);
		});

		it('should accept strong password', () => {
			const validData = {
				name: 'John Doe',
				email: 'user@example.com',
				password: 'StrongPass123!',
				confirmPassword: 'StrongPass123!'
			};

			const result = safeParse(registerSchema, validData);
			expect(result.success).toBe(true);
		});
	});

	describe('validatePasswordConfirmation', () => {
		it('should return true when passwords match', () => {
			expect(validatePasswordConfirmation('password123', 'password123')).toBe(true);
		});

		it('should return false when passwords do not match', () => {
			expect(validatePasswordConfirmation('password123', 'different123')).toBe(false);
		});

		it('should return false when passwords are empty', () => {
			expect(validatePasswordConfirmation('', '')).toBe(true);
		});

		it('should be case sensitive', () => {
			expect(validatePasswordConfirmation('Password', 'password')).toBe(false);
		});
	});

	describe('calculatePasswordStrength', () => {
		it('should return score 0 for empty password', () => {
			const result = calculatePasswordStrength('');
			expect(result.score).toBe(0);
			expect(result.strength).toBe('Very Weak');
		});

		it('should return low strength for short password', () => {
			const result = calculatePasswordStrength('123');
			expect(result.score).toBeLessThan(3);
		});

		it('should return higher strength for longer password', () => {
			const shortResult = calculatePasswordStrength('12345678');
			const longResult = calculatePasswordStrength('12345678901234567890abc');
			expect(longResult.score).toBeGreaterThan(shortResult.score);
		});

		it('should return higher strength for password with mixed case', () => {
			const lowerOnly = calculatePasswordStrength('password');
			const mixedCase = calculatePasswordStrength('Password');
			expect(mixedCase.score).toBeGreaterThan(lowerOnly.score);
		});

		it('should return higher strength for password with numbers', () => {
			const noNumbers = calculatePasswordStrength('Password');
			const withNumbers = calculatePasswordStrength('Password123');
			expect(withNumbers.score).toBeGreaterThan(noNumbers.score);
		});

		it('should return higher strength for password with special characters', () => {
			const noSpecial = calculatePasswordStrength('Password123');
			const withSpecial = calculatePasswordStrength('Password123!');
			expect(withSpecial.score).toBeGreaterThan(noSpecial.score);
		});

		it('should return maximum strength for very strong password', () => {
			const result = calculatePasswordStrength('VeryStrongPassword123!@#');
			expect(result.score).toBe(5);
			expect(result.strength).toBe('Very Strong');
		});

		it('should return correct strength levels', () => {
			expect(calculatePasswordStrength('').strength).toBe('Very Weak');
			expect(calculatePasswordStrength('weak').strength).toBe('Very Weak');
			expect(calculatePasswordStrength('weakpass').strength).toBe('Weak');
			expect(calculatePasswordStrength('Weakpass').strength).toBe('Fair');
			expect(calculatePasswordStrength('Weakpass1').strength).toBe('Strong');
			expect(calculatePasswordStrength('Weakpass1!').strength).toBe('Very Strong');
		});
	});
});
