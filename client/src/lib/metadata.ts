import { PUBLIC_BACKEND } from "$env/static/public";
import { error } from "@sveltejs/kit";

export async function get_file_system(token: string): Promise<FileSystemInfo> {
    const res = await fetch(`${PUBLIC_BACKEND}/m/file_system`, {
        credentials: 'include',
        headers: {
            'Authorization': `Bearer ${token}`
        },
    });
    if (!res.ok) {
        throw error(res.status, "Failed to fetch file system");
    }

    const data: FileSystemInfo = await res.json();

    return data;
}

export async function get_storage_limit(token: string): Promise<StorageLimit> {
    const res = await fetch(`${PUBLIC_BACKEND}/m/storage_limit`, {
        credentials: 'include',
        headers: {
            'Authorization': `Bearer ${token}`
        }
    });
    if (!res.ok) {
        throw error(res.status, "Failed to fetch storage limit");
    }

    const data: StorageLimit = await res.json();

    return data;
}

export type StorageLimit = {
    used: number;
    max: number;
};

export type FileSystemInfo = {
    which: string,
    about: string,
}