import { browser } from '$app/environment';

export type Theme = 'light' | 'dark' | 'system';

const THEME_KEY = 'anthill-theme';

function getSystemTheme(): 'light' | 'dark' {
	if (!browser) return 'light';
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function getStoredTheme(): Theme {
	if (!browser) return 'system';
	const stored = localStorage.getItem(THEME_KEY);
	if (stored === 'light' || stored === 'dark' || stored === 'system') {
		return stored;
	}
	return 'system';
}

function applyTheme(theme: Theme) {
	if (!browser) return;

	const root = document.documentElement;
	const resolvedTheme = theme === 'system' ? getSystemTheme() : theme;

	root.classList.remove('light', 'dark');
	root.classList.add(resolvedTheme);

	// Also set color-scheme for native elements
	root.style.colorScheme = resolvedTheme;
}

class ThemeStore {
	#theme = $state<Theme>(getStoredTheme());
	#resolvedTheme = $state<'light' | 'dark'>(
		this.#theme === 'system' ? getSystemTheme() : this.#theme
	);

	constructor() {
		if (browser) {
			// Apply initial theme
			applyTheme(this.#theme);

			// Listen for system theme changes
			const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
			mediaQuery.addEventListener('change', (e) => {
				if (this.#theme === 'system') {
					this.#resolvedTheme = e.matches ? 'dark' : 'light';
					applyTheme(this.#theme);
				}
			});
		}
	}

	get theme(): Theme {
		return this.#theme;
	}

	get resolvedTheme(): 'light' | 'dark' {
		return this.#resolvedTheme;
	}

	get isDark(): boolean {
		return this.#resolvedTheme === 'dark';
	}

	get isLight(): boolean {
		return this.#resolvedTheme === 'light';
	}

	get isSystem(): boolean {
		return this.#theme === 'system';
	}

	setTheme(theme: Theme) {
		this.#theme = theme;
		this.#resolvedTheme = theme === 'system' ? getSystemTheme() : theme;

		if (browser) {
			localStorage.setItem(THEME_KEY, theme);
			applyTheme(theme);
		}
	}

	toggleTheme() {
		// Cycle through: light -> dark -> system -> light
		const nextTheme: Theme =
			this.#theme === 'light' ? 'dark' : this.#theme === 'dark' ? 'system' : 'light';
		this.setTheme(nextTheme);
	}

	setLight() {
		this.setTheme('light');
	}

	setDark() {
		this.setTheme('dark');
	}

	setSystem() {
		this.setTheme('system');
	}
}

export const themeStore = new ThemeStore();
