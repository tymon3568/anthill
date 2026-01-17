import { object, string, minLength, email, pipe, regex, forward, partialCheck } from 'valibot';

// Login form validation schema
export const loginSchema = object({
	email: pipe(string('Email is required'), email('Please enter a valid email address')),
	password: pipe(string('Password is required'), minLength(1, 'Password is required'))
});

// Password validation with strength requirements
const passwordSchema = pipe(
	string('Password is required'),
	minLength(8, 'Password must be at least 8 characters long'),
	regex(/[A-Z]/, 'Password must contain at least one uppercase letter'),
	regex(/[a-z]/, 'Password must contain at least one lowercase letter'),
	regex(/[0-9]/, 'Password must contain at least one number')
);

// Register form validation schema (basic)
export const registerSchema = object({
	email: pipe(string('Email is required'), email('Please enter a valid email address')),
	password: passwordSchema,
	confirmPassword: pipe(
		string('Please confirm your password'),
		minLength(1, 'Please confirm your password')
	)
});

// Full registration schema including full_name and tenant_name
// Using forward + partialCheck for password confirmation matching
export const fullRegisterSchema = pipe(
	object({
		email: pipe(string('Email is required'), email('Please enter a valid email address')),
		password: passwordSchema,
		confirmPassword: pipe(
			string('Please confirm your password'),
			minLength(1, 'Please confirm your password')
		),
		fullName: pipe(string('Full name is required'), minLength(1, 'Full name is required')),
		tenantName: pipe(
			string('Organization name is required'),
			minLength(1, 'Organization name is required')
		)
	}),
	forward(
		partialCheck(
			[['password'], ['confirmPassword']],
			(input) => input.password === input.confirmPassword,
			'Passwords do not match'
		),
		['confirmPassword']
	)
);

// Password strength validation (standalone for real-time feedback)
export const passwordStrengthSchema = passwordSchema;

// Re-export form types from centralized types module
export type { LoginForm, RegisterForm } from '$lib/types';
