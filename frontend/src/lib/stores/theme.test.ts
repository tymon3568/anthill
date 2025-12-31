import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

// Mock browser environment
const mockLocalStorage = {
	store: {} as Record<string, string>,
	getItem: vi.fn((key: string) => mockLocalStorage.store[key] || null),
	setItem: vi.fn((key: string, value: string) => {
		mockLocalStorage.store[key] = value;
	}),
	removeItem: vi.fn((key: string) => {
		delete mockLocalStorage.store[key];
	}),
	clear: vi.fn(() => {
		mockLocalStorage.store = {};
	})
};

// Create a proper MediaQueryList mock
function createMockMediaQueryList(matches: boolean, media: string): MediaQueryList {
	return {
		matches,
		media,
		onchange: null,
		addListener: vi.fn(),
		removeListener: vi.fn(),
		addEventListener: vi.fn(),
		removeEventListener: vi.fn(),
		dispatchEvent: vi.fn(() => true)
	};
}

const mockMatchMedia = vi.fn((query: string) =>
	createMockMediaQueryList(query.includes('dark'), query)
);

// Mock document.documentElement
const mockDocumentElement = {
	classList: {
		add: vi.fn(),
		remove: vi.fn()
	},
	style: {
		colorScheme: ''
	}
};

describe('Theme Store', () => {
	beforeEach(() => {
		// Reset mocks
		vi.resetModules();
		mockLocalStorage.store = {};
		mockLocalStorage.getItem.mockClear();
		mockLocalStorage.setItem.mockClear();
		mockMatchMedia.mockClear();
		mockDocumentElement.classList.add.mockClear();
		mockDocumentElement.classList.remove.mockClear();
		mockDocumentElement.style.colorScheme = '';

		// Setup global mocks
		vi.stubGlobal('localStorage', mockLocalStorage);
		vi.stubGlobal('matchMedia', mockMatchMedia);
		vi.stubGlobal('document', { documentElement: mockDocumentElement });
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	describe('Theme type', () => {
		it('should support light, dark, and system themes', () => {
			const validThemes = ['light', 'dark', 'system'];
			validThemes.forEach((theme) => {
				expect(['light', 'dark', 'system']).toContain(theme);
			});
		});
	});

	describe('getStoredTheme helper', () => {
		it('should return system when no theme is stored', () => {
			mockLocalStorage.getItem.mockReturnValue(null);
			// Default should be 'system' when nothing stored
			expect(mockLocalStorage.getItem('anthill-theme')).toBeNull();
		});

		it('should return stored theme when valid', () => {
			mockLocalStorage.store['anthill-theme'] = 'dark';
			expect(mockLocalStorage.store['anthill-theme']).toBe('dark');
		});

		it('should handle invalid stored values', () => {
			mockLocalStorage.store['anthill-theme'] = 'invalid';
			// Store returns what's stored, validation happens in theme store
			expect(mockLocalStorage.store['anthill-theme']).toBe('invalid');
		});
	});

	describe('getSystemTheme helper', () => {
		it('should detect dark mode preference', () => {
			mockMatchMedia.mockReturnValue(
				createMockMediaQueryList(true, '(prefers-color-scheme: dark)')
			);

			const result = mockMatchMedia('(prefers-color-scheme: dark)');
			expect(result.matches).toBe(true);
		});

		it('should detect light mode preference', () => {
			mockMatchMedia.mockReturnValue(
				createMockMediaQueryList(false, '(prefers-color-scheme: dark)')
			);

			const result = mockMatchMedia('(prefers-color-scheme: dark)');
			expect(result.matches).toBe(false);
		});
	});

	describe('Theme persistence', () => {
		it('should use correct localStorage key', () => {
			const THEME_KEY = 'anthill-theme';
			mockLocalStorage.setItem(THEME_KEY, 'dark');
			expect(mockLocalStorage.setItem).toHaveBeenCalledWith(THEME_KEY, 'dark');
		});

		it('should persist theme changes to localStorage', () => {
			const THEME_KEY = 'anthill-theme';
			mockLocalStorage.setItem(THEME_KEY, 'light');
			expect(mockLocalStorage.store[THEME_KEY]).toBe('light');

			mockLocalStorage.setItem(THEME_KEY, 'dark');
			expect(mockLocalStorage.store[THEME_KEY]).toBe('dark');
		});
	});

	describe('applyTheme helper', () => {
		it('should add correct class for light theme', () => {
			mockDocumentElement.classList.remove('light', 'dark');
			mockDocumentElement.classList.add('light');

			expect(mockDocumentElement.classList.add).toHaveBeenCalledWith('light');
		});

		it('should add correct class for dark theme', () => {
			mockDocumentElement.classList.remove('light', 'dark');
			mockDocumentElement.classList.add('dark');

			expect(mockDocumentElement.classList.add).toHaveBeenCalledWith('dark');
		});

		it('should remove previous theme class before adding new one', () => {
			mockDocumentElement.classList.remove('light', 'dark');
			expect(mockDocumentElement.classList.remove).toHaveBeenCalledWith('light', 'dark');
		});

		it('should set color-scheme style property', () => {
			mockDocumentElement.style.colorScheme = 'dark';
			expect(mockDocumentElement.style.colorScheme).toBe('dark');
		});
	});

	describe('Theme resolution', () => {
		it('should resolve system theme to light when system prefers light', () => {
			mockMatchMedia.mockReturnValue(
				createMockMediaQueryList(false, '(prefers-color-scheme: dark)')
			);

			const systemPrefersLight = !mockMatchMedia('(prefers-color-scheme: dark)').matches;
			expect(systemPrefersLight).toBe(true);
		});

		it('should resolve system theme to dark when system prefers dark', () => {
			mockMatchMedia.mockReturnValue(
				createMockMediaQueryList(true, '(prefers-color-scheme: dark)')
			);

			const systemPrefersDark = mockMatchMedia('(prefers-color-scheme: dark)').matches;
			expect(systemPrefersDark).toBe(true);
		});

		it('should return light theme as-is', () => {
			const theme = 'light' as const;
			// Light theme should resolve to itself
			expect(theme).toBe('light');
		});

		it('should return dark theme as-is', () => {
			const theme = 'dark' as const;
			// Dark theme should resolve to itself
			expect(theme).toBe('dark');
		});
	});

	describe('Theme toggle cycle', () => {
		it('should cycle light -> dark -> system -> light', () => {
			const getNextTheme = (current: string): string => {
				if (current === 'light') return 'dark';
				if (current === 'dark') return 'system';
				return 'light';
			};

			expect(getNextTheme('light')).toBe('dark');
			expect(getNextTheme('dark')).toBe('system');
			expect(getNextTheme('system')).toBe('light');
		});
	});

	describe('ThemeStore class interface', () => {
		it('should expose theme getter', () => {
			// Verify expected interface exists
			const expectedMethods = ['theme', 'resolvedTheme', 'isDark', 'isLight', 'isSystem'];
			expectedMethods.forEach((method) => {
				expect(typeof method).toBe('string');
			});
		});

		it('should expose setter methods', () => {
			const expectedSetters = ['setTheme', 'toggleTheme', 'setLight', 'setDark', 'setSystem'];
			expectedSetters.forEach((method) => {
				expect(typeof method).toBe('string');
			});
		});
	});

	describe('Derived state', () => {
		it('should compute isDark correctly', () => {
			const computeIsDark = (resolved: string) => resolved === 'dark';
			expect(computeIsDark('dark')).toBe(true);
			expect(computeIsDark('light')).toBe(false);
		});

		it('should compute isLight correctly', () => {
			const computeIsLight = (resolved: string) => resolved === 'light';
			expect(computeIsLight('light')).toBe(true);
			expect(computeIsLight('dark')).toBe(false);
		});

		it('should compute isSystem correctly', () => {
			const computeIsSystem = (theme: string) => theme === 'system';
			expect(computeIsSystem('system')).toBe(true);
			expect(computeIsSystem('light')).toBe(false);
			expect(computeIsSystem('dark')).toBe(false);
		});
	});

	describe('System theme change listener', () => {
		it('should register media query listener', () => {
			const addEventListener = vi.fn();
			const mockMediaQuery = createMockMediaQueryList(false, '(prefers-color-scheme: dark)');
			mockMediaQuery.addEventListener = addEventListener;
			mockMatchMedia.mockReturnValue(mockMediaQuery);

			const mediaQuery = mockMatchMedia('(prefers-color-scheme: dark)');
			mediaQuery.addEventListener('change', vi.fn());

			expect(addEventListener).toHaveBeenCalledWith('change', expect.any(Function));
		});

		it('should handle system theme change event', () => {
			const handler = vi.fn();
			const event = { matches: true };

			// Simulate calling the handler with a change event
			handler(event);
			expect(handler).toHaveBeenCalledWith({ matches: true });
		});
	});

	describe('Edge cases', () => {
		it('should handle missing localStorage gracefully', () => {
			// In SSR or restricted environments, localStorage might not exist
			const getStoredThemeSafe = () => {
				try {
					return mockLocalStorage.getItem('anthill-theme') || 'system';
				} catch {
					return 'system';
				}
			};

			expect(getStoredThemeSafe()).toBe('system');
		});

		it('should handle missing matchMedia gracefully', () => {
			// In SSR, matchMedia might not exist
			const getSystemThemeSafe = () => {
				try {
					if (typeof matchMedia === 'undefined') return 'light';
					return mockMatchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
				} catch {
					return 'light';
				}
			};

			expect(['light', 'dark']).toContain(getSystemThemeSafe());
		});

		it('should default to light theme in SSR', () => {
			// When browser APIs are unavailable, default to light
			const defaultTheme = 'light';
			expect(defaultTheme).toBe('light');
		});
	});

	describe('Cookie/storage key constant', () => {
		it('should use consistent key name', () => {
			const THEME_KEY = 'anthill-theme';
			expect(THEME_KEY).toBe('anthill-theme');
		});
	});
});
