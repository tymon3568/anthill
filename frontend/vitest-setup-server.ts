/// <reference types="vitest" />

// Mock SvelteKit imports for server-side testing
import { vi } from 'vitest';

vi.mock('$app/environment', () => ({
	browser: false
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$app/stores', () => ({
	page: vi.fn()
}));
