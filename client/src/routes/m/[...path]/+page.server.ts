import { get_files } from '$lib/file';
import { get_file_system, get_storage_limit } from '$lib/metadata';
import { error, redirect } from '@sveltejs/kit';

export async function load({ cookies, params }) {
    const token = cookies.get("token");
    if (!token) {
        throw error(500, "Token not found in cookies");
    }

    const path = params.path;
    if (!path) throw redirect(302, "/m");

    return {
        file_system: await get_file_system(token),
        files: await get_files(path, token, true),
        storage_limit: await get_storage_limit(token),
        path: path
    }
}
