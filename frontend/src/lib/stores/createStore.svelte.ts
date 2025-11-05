import { browser } from "$app/environment";
import { writable } from "svelte/store";
import CryptoJS from 'crypto-js';
const decryptCode = 'sdf123jfd@sadjf!dsf';
const decryptFunc = async (data: string) => {
	const response = await fetch(`/api/decrypt`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
		body: JSON.stringify(data)
	});
	return response.json();
};

  export const useLocalStorage = <T>(key: string, value: T) => {
	let storage = $state<{ value: T }>({ value });
	if (browser) {
		const item = localStorage.getItem(key);
		if (item) {
			let decryptedValue = CryptoJS.AES.decrypt(item, decryptCode).toString(CryptoJS.enc.Utf8);
			storage.value = JSON.parse(decryptedValue);
		} else {
			let encryptedValue = CryptoJS.AES.encrypt(
				JSON.stringify(storage.value),
				decryptCode
			).toString();
			localStorage.setItem(key, encryptedValue);
		}
	}
	return {
		get value() {
			return storage.value;
		},
		set value(newValue: T) {
			storage.value = newValue;
			if (browser) {
				let encryptedValue = CryptoJS.AES.encrypt(
					JSON.stringify(storage.value),
					decryptCode
				).toString();
				localStorage.setItem(key, encryptedValue);
				// localStorage.setItem(key, JSON.stringify(storage.value));
			}
		}
	};
};
