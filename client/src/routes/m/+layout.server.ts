import { PUBLIC_BACKEND } from '$env/static/public';
import { redirect } from '@sveltejs/kit';

export async function load({ cookies }) {
    const token = cookies.get("token");
    if (!token) {
        throw redirect(302, "/");
    }

    const response = await fetch(`${PUBLIC_BACKEND}/m/check`, {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${token}`
        },
        credentials: 'include'
    });
    if (!response.ok) {
        console.error("Failed to check authentication:", response.statusText);
        throw redirect(302, "/");
    }

    return { status: 200 };
}