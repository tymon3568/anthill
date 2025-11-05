import { browser } from "$app/environment";
import { writable } from "svelte/store";

  export const useLocalStorage = <T>(key: string, value: T) => {
	let storage = $state<{ value: T }>({ value });
	if (browser) {
		const item = localStorage.getItem(key);
		if (item) {
			try {
				storage.value = JSON.parse(item);
			} catch (error) {
				// Clear corrupted data and use default value
				console.error('Failed to parse stored value:', error);
				localStorage.removeItem(key);
			}
		}
	}
	return {
		get value() {
			return storage.value;
		},
		set value(newValue: T) {
			storage.value = newValue;
			if (browser) {
				localStorage.setItem(key, JSON.stringify(storage.value));
			}
		}
	};
};
