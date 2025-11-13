import { object, string, minLength, email, pipe } from 'valibot';

// Login form validation schema
export const loginSchema = object({
	email: pipe(string('Email is required'), email('Please enter a valid email address')),
	password: pipe(string('Password is required'), minLength(1, 'Password is required'))
});

// Register form validation schema
export const registerSchema = object({
	email: pipe(string('Email is required'), email('Please enter a valid email address')),
	password: pipe(
		string('Password is required'),
		minLength(8, 'Password must be at least 8 characters long')
	),
	confirmPassword: pipe(
		string('Please confirm your password'),
		minLength(1, 'Please confirm your password')
	)
});

// Full registration schema including full_name and tenant_name
export const fullRegisterSchema = object({
	email: pipe(string('Email is required'), email('Please enter a valid email address')),
	password: pipe(
		string('Password is required'),
		minLength(8, 'Password must be at least 8 characters long')
	),
	confirmPassword: pipe(
		string('Please confirm your password'),
		minLength(1, 'Please confirm your password')
	),
	fullName: pipe(string('Full name is required'), minLength(1, 'Full name is required')),
	tenantName: pipe(
		string('Organization name is required'),
		minLength(1, 'Organization name is required')
	)
});

// Password strength validation (optional enhancement)
export const passwordStrengthSchema = pipe(
	string(),
	minLength(8, 'Password must be at least 8 characters')
	// Add more complex validation if needed
);

// Re-export form types from centralized types module
export type { LoginForm, RegisterForm } from '$lib/types';
