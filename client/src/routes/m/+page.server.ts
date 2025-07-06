import { get_files } from '$lib/file';
import { get_file_system, get_storage_limit } from '$lib/metadata';
import { error } from '@sveltejs/kit';

export async function load({ cookies }) {
    const token = cookies.get("token");
    if (!token) {
        throw error(500, "Token not found in cookies");
    }

    return {
        file_system: await get_file_system(token),
        files: await get_files("", token, true),
        storage_limit: await get_storage_limit(token)
    }
}
