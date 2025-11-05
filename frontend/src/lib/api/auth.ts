import { apiClient } from './client';
import type { ApiResponse, User, LoginForm } from '$lib/types';

export interface AuthResponse {
	user: User;
	access_token: string;
	refresh_token: string;
	expires_in: number;
}

export const authApi = {
	async login(credentials: LoginForm): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>('/auth/login', credentials);
	},

	async refreshToken(
		refreshToken: string
	): Promise<ApiResponse<{ access_token: string; expires_in: number }>> {
		return apiClient.post('/auth/refresh', { refresh_token: refreshToken });
	},

	async logout(): Promise<ApiResponse<void>> {
		return apiClient.post('/auth/logout');
	},

	async getProfile(): Promise<ApiResponse<User>> {
		return apiClient.get<User>('/auth/profile');
	},

	async updateProfile(userData: Partial<User>): Promise<ApiResponse<User>> {
		return apiClient.put<User>('/auth/profile', userData);
	}
};
