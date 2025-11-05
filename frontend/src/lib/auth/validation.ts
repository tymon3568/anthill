import { object, string, minLength, email, regex, pipe, parse, safeParse, nonEmpty } from 'valibot';

// Password validation rules
const passwordSchema = pipe(
	string('Password must be a string'),
	nonEmpty('Password is required'),
	minLength(8, 'Password must be at least 8 characters'),
	regex(/[a-z]/, 'Password must contain at least one lowercase letter'),
	regex(/[A-Z]/, 'Password must contain at least one uppercase letter'),
	regex(/[0-9]/, 'Password must contain at least one number'),
	regex(/[^A-Za-z0-9]/, 'Password must contain at least one special character')
);

// Login form validation schema
export const loginSchema = object({
	email: pipe(
		string('Email is required'),
		nonEmpty('Email is required'),
		email('Please enter a valid email address')
	),
	password: pipe(
		string('Password is required'),
		nonEmpty('Password is required'),
		minLength(6, 'Password must be at least 6 characters')
	)
});

// Registration form validation schema
export const registerSchema = object({
	name: pipe(
		string('Name is required'),
		nonEmpty('Name is required'),
		minLength(2, 'Name must be at least 2 characters'),
		regex(/^[a-zA-Z\s]+$/, 'Name can only contain letters and spaces')
	),
	email: pipe(
		string('Email is required'),
		nonEmpty('Email is required'),
		email('Please enter a valid email address')
	),
	password: passwordSchema,
	confirmPassword: pipe(
		string('Please confirm your password'),
		nonEmpty('Please confirm your password')
	)
});

// Custom validation function for password confirmation
export function validatePasswordConfirmation(password: string, confirmPassword: string): boolean {
	return password === confirmPassword;
}

// Password strength calculation (for UI feedback)
export function calculatePasswordStrength(password: string): {
	score: number;
	strength: 'Very Weak' | 'Weak' | 'Fair' | 'Good' | 'Strong' | 'Very Strong';
	color: string;
} {
	let score = 0;

	if (password.length >= 8) score++;
	if (/[a-z]/.test(password)) score++;
	if (/[A-Z]/.test(password)) score++;
	if (/[0-9]/.test(password)) score++;
	if (/[^A-Za-z0-9]/.test(password)) score++;

	let strength: 'Very Weak' | 'Weak' | 'Fair' | 'Good' | 'Strong' | 'Very Strong';
	let color: string;

	switch (score) {
		case 0:
		case 1:
			strength = 'Very Weak';
			color = 'text-red-600';
			break;
		case 2:
			strength = 'Weak';
			color = 'text-red-600';
			break;
		case 3:
			strength = 'Fair';
			color = 'text-orange-600';
			break;
		case 4:
			strength = 'Strong';
			color = 'text-green-600';
			break;
		case 5:
			strength = 'Very Strong';
			color = 'text-green-700';
			break;
		default:
			strength = 'Very Weak';
			color = 'text-red-600';
	}

	return { score, strength, color };
}

// Type exports for TypeScript
export type LoginForm = {
	email: string;
	password: string;
};

export type RegisterForm = {
	name: string;
	email: string;
	password: string;
	confirmPassword: string;
};
