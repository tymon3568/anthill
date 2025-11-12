# Task: Form Validation with Valibot

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.02_form_validation.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-11-12
**Last Updated:** 2025-11-12

## Detailed Description:
Implement comprehensive form validation for login and registration forms using Valibot. Create validation schemas that ensure data integrity, provide clear error messages, and integrate seamlessly with Svelte 5 reactive forms.

## Acceptance Criteria:
- [ ] Login form validation schema with email and password rules
- [ ] Registration form validation schema with email, password, and confirm password
- [ ] Password strength requirements (minimum length, complexity)
- [ ] Email format validation
- [ ] Confirm password matching validation
- [ ] Clear, user-friendly error messages in English
- [ ] Real-time validation feedback
- [ ] Form submission blocked when validation fails
- [ ] Validation integrates with Svelte 5 runes reactivity
- [ ] Code compiles without errors: `bun run check`
- [ ] Validation logic is reusable and well-tested

## Specific Sub-tasks:
- [ ] 1. Set up Valibot validation schemas
    - [ ] 1.1. Install valibot dependency if not already installed
    - [ ] 1.2. Create validation utilities file (`src/lib/validation/auth.ts`)
    - [ ] 1.3. Define login schema (email + password)
    - [ ] 1.4. Define registration schema (email + password + confirmPassword)

- [ ] 2. Implement password validation rules
    - [ ] 2.1. Minimum length requirement (8+ characters)
    - [ ] 2.2. At least one uppercase letter
    - [ ] 2.3. At least one lowercase letter
    - [ ] 2.4. At least one number
    - [ ] 2.5. Optional: special character requirement

- [ ] 3. Create form validation helpers
    - [ ] 3.1. Function to validate individual fields
    - [ ] 3.2. Function to validate entire form
    - [ ] 3.3. Error message formatting utilities
    - [ ] 3.4. Integration with Svelte 5 reactive state

- [ ] 4. Integrate validation with authentication pages
    - [ ] 4.1. Update login page to use validation
    - [ ] 4.2. Update registration page to use validation
    - [ ] 4.3. Display validation errors in UI
    - [ ] 4.4. Prevent form submission on validation errors

## Dependencies:
*   Task: `task_08.02.01_authentication_pages.md` (Status: Done)

## Files to Create/Modify:
*   `src/lib/validation/auth.ts` - Valibot validation schemas and helpers
*   `src/routes/login/+page.svelte` - Add validation integration
*   `src/routes/register/+page.svelte` - Add validation integration
*   `package.json` - Add valibot dependency (if needed)

## Code Examples:
```typescript
// src/lib/validation/auth.ts
import * as v from 'valibot';

export const loginSchema = v.object({
  email: v.pipe(
    v.string('Email is required'),
    v.email('Please enter a valid email address')
  ),
  password: v.pipe(
    v.string('Password is required'),
    v.minLength(1, 'Password is required')
  )
});

export const registerSchema = v.object({
  email: v.pipe(
    v.string('Email is required'),
    v.email('Please enter a valid email address')
  ),
  password: v.pipe(
    v.string('Password is required'),
    v.minLength(8, 'Password must be at least 8 characters'),
    v.regex(/[A-Z]/, 'Password must contain at least one uppercase letter'),
    v.regex(/[a-z]/, 'Password must contain at least one lowercase letter'),
    v.regex(/[0-9]/, 'Password must contain at least one number')
  ),
  confirmPassword: v.string('Please confirm your password')
}).refine(
  (data) => data.password === data.confirmPassword,
  {
    message: 'Passwords do not match',
    path: ['confirmPassword']
  }
);

export type LoginData = v.InferOutput<typeof loginSchema>;
export type RegisterData = v.InferOutput<typeof registerSchema>;
```

```svelte
<!-- Integration example in login page -->
<script>
  import { loginSchema, type LoginData } from '$lib/validation/auth';
  import * as v from 'valibot';

  let formData = $state<LoginData>({ email: '', password: '' });
  let errors = $state<Record<string, string>>({});
  let isSubmitting = $state(false);

  function validateField(field: keyof LoginData) {
    try {
      const fieldSchema = v.pick(loginSchema, [field]);
      v.parse(fieldSchema, { [field]: formData[field] });
      errors[field] = '';
    } catch (error) {
      if (error instanceof v.ValiError) {
        errors[field] = error.message;
      }
    }
  }

  function validateForm(): boolean {
    try {
      v.parse(loginSchema, formData);
      errors = {};
      return true;
    } catch (error) {
      if (error instanceof v.ValiError) {
        errors = {};
        for (const issue of error.issues) {
          const field = issue.path?.[0]?.key as string;
          if (field) {
            errors[field] = issue.message;
          }
        }
      }
      return false;
    }
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    isSubmitting = true;
    try {
      // API call will be implemented in next task
      console.log('Login data:', formData);
    } finally {
      isSubmitting = false;
    }
  }
</script>
```

## Testing Steps:
- [ ] Test login validation with valid/invalid emails
- [ ] Test password requirements on registration form
- [ ] Test confirm password matching
- [ ] Verify error messages are clear and helpful
- [ ] Test form submission is blocked with validation errors
- [ ] Test real-time validation feedback

## References:
*   Valibot documentation (research via Context7)
*   `frontend/.svelte-instructions.md` - Svelte 5 guidelines
*   Project validation patterns

## Notes / Discussion:
---
*   Use English error messages throughout
*   Password requirements should balance security with usability
*   Validation should be real-time but not overly aggressive
*   Consider accessibility when displaying error messages

## AI Agent Log:
---
*   2025-11-12 10:15: Task created by Claude
    - Set up comprehensive valibot validation for auth forms
    - Included password strength requirements
    - Added form integration examples
    - Ready for implementation</content>
<parameter name="filePath">/home/arch/Project/test/anthill/PROJECT_TRACKING/V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.02_form_validation.md
