/**
 * Auth validation schemas using Valibot
 *
 * Production standards:
 * - Email format validation
 * - Password min 8 characters
 * - Password strength requirements
 * - Confirmation matching
 * - Comprehensive error messages
 */

import * as v from 'valibot';

/**
 * Login form validation schema
 *
 * Input requirements:
 * - email: valid email format, max 255 chars
 * - password: min 8 characters, max 128 chars
 */
export const LoginSchema = v.object({
	email: v.pipe(
		v.string('Email is required'),
		v.trim(),
		v.nonEmpty('Email cannot be empty'),
		v.email('Please enter a valid email address'),
		v.maxLength(255, 'Email is too long')
	),
	password: v.pipe(
		v.string('Password is required'),
		v.nonEmpty('Password cannot be empty'),
		v.minLength(8, 'Password must be at least 8 characters'),
		v.maxLength(128, 'Password is too long')
	)
});

export type LoginInput = v.InferOutput<typeof LoginSchema>;

/**
 * Registration form validation schema
 *
 * Input requirements:
 * - full_name: 2-100 characters
 * - email: valid email format
 * - password: min 8 chars with strength requirements
 * - confirmPassword: must match password
 * - tenant_name: optional, 2-50 characters
 */
export const RegisterSchema = v.pipe(
	v.object({
		full_name: v.pipe(
			v.string('Full name is required'),
			v.trim(),
			v.nonEmpty('Full name cannot be empty'),
			v.minLength(2, 'Full name must be at least 2 characters'),
			v.maxLength(100, 'Full name is too long')
		),
		email: v.pipe(
			v.string('Email is required'),
			v.trim(),
			v.nonEmpty('Email cannot be empty'),
			v.email('Please enter a valid email address'),
			v.maxLength(255, 'Email is too long')
		),
		password: v.pipe(
			v.string('Password is required'),
			v.nonEmpty('Password cannot be empty'),
			v.minLength(8, 'Password must be at least 8 characters'),
			v.maxLength(128, 'Password is too long'),
			v.regex(/[A-Z]/, 'Password must contain at least one uppercase letter'),
			v.regex(/[a-z]/, 'Password must contain at least one lowercase letter'),
			v.regex(/[0-9]/, 'Password must contain at least one number')
		),
		confirmPassword: v.pipe(
			v.string('Please confirm your password'),
			v.nonEmpty('Password confirmation cannot be empty')
		),
		tenant_name: v.optional(
			v.pipe(
				v.string(),
				v.trim(),
				v.minLength(2, 'Organization name must be at least 2 characters'),
				v.maxLength(50, 'Organization name is too long')
			)
		)
	}),
	v.forward(
		v.partialCheck(
			[['password'], ['confirmPassword']],
			(input) => input.password === input.confirmPassword,
			'Passwords do not match'
		),
		['confirmPassword']
	)
);

export type RegisterInput = v.InferOutput<typeof RegisterSchema>;

/**
 * Helper function to extract field-specific errors from ValiError
 */
export function extractFieldErrors<T extends Record<string, unknown>>(
	error: v.ValiError<v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>>
): Partial<Record<keyof T, string>> {
	const fieldErrors: Partial<Record<keyof T, string>> = {};

	for (const issue of error.issues) {
		const path = issue.path?.[0]?.key as keyof T;
		if (path && !fieldErrors[path]) {
			// Only take the first error per field
			fieldErrors[path] = issue.message;
		}
	}

	return fieldErrors;
}

/**
 * Helper function to validate and return typed result
 */
export function validateLogin(data: unknown):
	| { success: true; data: LoginInput }
	| { success: false; errors: Partial<Record<keyof LoginInput, string>> }
{
	try {
		const validated = v.parse(LoginSchema, data);
		return { success: true, data: validated };
	} catch (error) {
		if (error && typeof error === 'object' && 'issues' in error) {
			return { success: false, errors: extractFieldErrors<LoginInput>(error as v.ValiError<typeof LoginSchema>) };
		}
		throw error;
	}
}

/**
 * Helper function to validate registration
 */
export function validateRegister(data: unknown):
	| { success: true; data: RegisterInput }
	| { success: false; errors: Partial<Record<keyof RegisterInput, string>> }
{
	try {
		const validated = v.parse(RegisterSchema, data);
		return { success: true, data: validated };
	} catch (error) {
		if (error && typeof error === 'object' && 'issues' in error) {
			return { success: false, errors: extractFieldErrors<RegisterInput>(error as v.ValiError<typeof RegisterSchema>) };
		}
		throw error;
	}
}
