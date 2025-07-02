import { PUBLIC_BACKEND } from '$env/static/public';
import { get_files, type FileSystemInfo, type StorageLimit } from '$lib';
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

async function get_file_system(token: string): Promise<FileSystemInfo> {
    const res = await fetch(`${PUBLIC_BACKEND}/m/file_system`, {
        credentials: 'include',
        headers: {
            'Authorization': `Bearer ${token}`
        },
    });
    if (!res.ok) {
        console.error("Failed to fetch file system:", res.statusText);
        throw error(res.status, "Failed to fetch file system");
    }

    const data: FileSystemInfo = await res.json();

    return data;
}

async function get_storage_limit(token: string): Promise<StorageLimit> {
    const res = await fetch(`${PUBLIC_BACKEND}/m/storage_limit`, {
        credentials: 'include',
        headers: {
            'Authorization': `Bearer ${token}`
        }
    });
    if (!res.ok) {
        console.error("Failed to fetch storage limit:", res.statusText);
        throw error(res.status, "Failed to fetch storage limit");
    }

    const data: StorageLimit = await res.json();

    return data;
}