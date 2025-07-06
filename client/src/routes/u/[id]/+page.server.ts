import { PUBLIC_BACKEND } from '$env/static/public';

export async function load({ params, url }) {
    const id = params.id;
    if (!id) return { is_valid: false };

    const query = url.searchParams.get('f');
    if (query === "t") {
        const file_id = url.searchParams.get('id');
        if (!file_id) return { is_valid: false };
        // this is after a upload has been completed
        return { is_valid: true, id, done: true, file_id };
    }

    const res = await fetch(`${PUBLIC_BACKEND}/verify_link/${id}`, {
        credentials: 'include',
        method: 'POST',
    });
    if (!res.ok) {
        return { is_valid: false };
    }

    return { is_valid: true, id };
}