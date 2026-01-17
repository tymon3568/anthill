import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => {
	const token = url.searchParams.get('token');
	return {
		token
	};
};
