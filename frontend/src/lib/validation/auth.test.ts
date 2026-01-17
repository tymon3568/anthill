import { describe, it, expect } from 'vitest';
import { safeParse } from 'valibot';
import { loginSchema, registerSchema, fullRegisterSchema, passwordStrengthSchema } from './auth';

describe('Auth Validation Schemas', () => {
	describe('loginSchema', () => {
		it('should validate correct login data', () => {
			const result = safeParse(loginSchema, {
				email: 'user@example.com',
				password: 'password123'
			});

			expect(result.success).toBe(true);
		});

		it('should reject invalid email', () => {
			const result = safeParse(loginSchema, {
				email: 'invalid-email',
				password: 'password123'
			});

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe('Please enter a valid email address');
			}
		});

		it('should reject empty email', () => {
			const result = safeParse(loginSchema, {
				email: '',
				password: 'password123'
			});

			expect(result.success).toBe(false);
		});

		it('should reject empty password', () => {
			const result = safeParse(loginSchema, {
				email: 'user@example.com',
				password: ''
			});

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe('Password is required');
			}
		});

		it('should reject missing fields', () => {
			const result = safeParse(loginSchema, {});

			expect(result.success).toBe(false);
		});
	});

	describe('passwordStrengthSchema', () => {
		it('should validate strong password', () => {
			const result = safeParse(passwordStrengthSchema, 'Password123');

			expect(result.success).toBe(true);
		});

		it('should reject password shorter than 8 characters', () => {
			const result = safeParse(passwordStrengthSchema, 'Pass1');

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe('Password must be at least 8 characters long');
			}
		});

		it('should reject password without uppercase letter', () => {
			const result = safeParse(passwordStrengthSchema, 'password123');

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe(
					'Password must contain at least one uppercase letter'
				);
			}
		});

		it('should reject password without lowercase letter', () => {
			const result = safeParse(passwordStrengthSchema, 'PASSWORD123');

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe(
					'Password must contain at least one lowercase letter'
				);
			}
		});

		it('should reject password without number', () => {
			const result = safeParse(passwordStrengthSchema, 'PasswordABC');

			expect(result.success).toBe(false);
			if (!result.success) {
				expect(result.issues[0].message).toBe('Password must contain at least one number');
			}
		});
	});

	describe('registerSchema', () => {
		it('should validate correct registration data', () => {
			const result = safeParse(registerSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: 'Password123'
			});

			expect(result.success).toBe(true);
		});

		it('should reject weak password', () => {
			const result = safeParse(registerSchema, {
				email: 'user@example.com',
				password: 'weak',
				confirmPassword: 'weak'
			});

			expect(result.success).toBe(false);
		});

		it('should reject invalid email', () => {
			const result = safeParse(registerSchema, {
				email: 'not-an-email',
				password: 'Password123',
				confirmPassword: 'Password123'
			});

			expect(result.success).toBe(false);
		});

		it('should reject empty confirmPassword', () => {
			const result = safeParse(registerSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: ''
			});

			expect(result.success).toBe(false);
		});
	});

	describe('fullRegisterSchema', () => {
		it('should validate correct full registration data', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: 'Password123',
				fullName: 'John Doe',
				tenantName: 'My Organization'
			});

			expect(result.success).toBe(true);
		});

		it('should reject mismatched passwords', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: 'DifferentPassword123',
				fullName: 'John Doe',
				tenantName: 'My Organization'
			});

			expect(result.success).toBe(false);
			if (!result.success) {
				const confirmError = result.issues.find((issue) =>
					issue.path?.some((p) => p.key === 'confirmPassword')
				);
				expect(confirmError?.message).toBe('Passwords do not match');
			}
		});

		it('should reject empty fullName', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: 'Password123',
				fullName: '',
				tenantName: 'My Organization'
			});

			expect(result.success).toBe(false);
		});

		it('should reject empty tenantName', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123',
				confirmPassword: 'Password123',
				fullName: 'John Doe',
				tenantName: ''
			});

			expect(result.success).toBe(false);
		});

		it('should reject missing fields', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123'
			});

			expect(result.success).toBe(false);
		});

		it('should validate complex email formats', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user.name+tag@subdomain.example.com',
				password: 'Password123',
				confirmPassword: 'Password123',
				fullName: 'John Doe',
				tenantName: 'My Organization'
			});

			expect(result.success).toBe(true);
		});

		it('should accept password with special characters', () => {
			const result = safeParse(fullRegisterSchema, {
				email: 'user@example.com',
				password: 'Password123!@#',
				confirmPassword: 'Password123!@#',
				fullName: 'John Doe',
				tenantName: 'My Organization'
			});

			expect(result.success).toBe(true);
		});
	});
});
