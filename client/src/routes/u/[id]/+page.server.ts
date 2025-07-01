import { PUBLIC_BACKEND } from '$env/static/public';

export async function load({ params }) {
    const id = params.id;
    if (!id) return { is_valid: false };

    const res = await fetch(`${PUBLIC_BACKEND}/verify_link/${id}`, {
        credentials: 'include',
        method: 'POST',
    });
    if (!res.ok) {
        return { is_valid: false };
    }

    return { is_valid: true, id };
}