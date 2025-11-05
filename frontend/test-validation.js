import { object, string, nonEmpty, email, pipe, safeParse } from 'valibot';

const loginSchema = object({
email: pipe(
string('Email is required'),
nonEmpty('Email is required'),
email('Please enter a valid email address')
),
password: pipe(
string('Password is required'),
nonEmpty('Password is required')
)
});

const result = safeParse(loginSchema, { email: '', password: '' });
console.log('Success:', result.success);
if (!result.success) {
	console.log('Issues:', result.issues);
}
