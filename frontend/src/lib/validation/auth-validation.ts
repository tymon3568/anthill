export type RegisterInput = {
	full_name: string;
	email: string;
	password: string;
	confirmPassword: string;
	tenant_name?: string;
};

export function validateRegister(data: RegisterInput) {
	const errors: Partial<Record<keyof RegisterInput, string>> = {};

	// Validate full name
	if (!data.full_name || data.full_name.trim().length === 0) {
		errors.full_name = 'Full name is required';
	}

	// Validate email
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	if (!data.email || !emailRegex.test(data.email)) {
		errors.email = 'Please enter a valid email address';
	}

	// Validate password
	if (!data.password || data.password.length < 8) {
		errors.password = 'Password must be at least 8 characters';
	} else {
		const passwordErrors: string[] = [];
		if (!/[A-Z]/.test(data.password)) {
			passwordErrors.push('at least one uppercase letter');
		}
		if (!/[a-z]/.test(data.password)) {
			passwordErrors.push('at least one lowercase letter');
		}
		if (!/[0-9]/.test(data.password)) {
			passwordErrors.push('at least one number');
		}
		if (passwordErrors.length > 0) {
			errors.password = `Password must contain ${passwordErrors.join(', ')}`;
		}
	}

	// Validate confirm password
	if (!data.confirmPassword) {
		errors.confirmPassword = 'Please confirm your password';
	} else if (data.password !== data.confirmPassword) {
		errors.confirmPassword = 'Passwords do not match';
	}

	const hasErrors = Object.keys(errors).length > 0;

	return {
		success: !hasErrors,
		data: hasErrors ? undefined : data,
		errors: hasErrors ? errors : undefined
	};
}
