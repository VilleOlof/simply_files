import { PUBLIC_BACKEND } from "$env/static/public";
import { notification } from "./toast";

export type FileLink = {
    id: string,
    uploaded_file?: string,
    uploaded_at?: number[],
    created_at: number[],
}

export async function get_unused_links(token?: string): Promise<FileLink[]> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/links`, {
        credentials: 'include',
        headers: token ? { 'Authorization': `Bearer ${token}` } : {}
    });

    if (!response.ok) {
        notification.error(`Failed to fetch unused links: ${response.statusText}`);
        return [];
    }
    else {
        const data: FileLink[] = await response.json();
        return data;
    }
}

export async function delete_link(id: string): Promise<void> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/link/${id}`, {
        method: 'DELETE',
        credentials: 'include'
    });

    if (!response.ok) {
        notification.error(`Failed to delete link: ${response.statusText}`);
    }
    notification.success('Link deleted successfully!');
}

export async function create_link(): Promise<FileLink> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/new_link`, {
        method: 'POST',
        credentials: 'include'
    });

    if (!response.ok) {
        throw new Error(`Failed to create link: ${response.statusText}`);
    }

    const data: FileLink = await response.json();

    notification.success('Link created successfully!');

    return data;
}

export function copy_link(link: FileLink) {
    navigator.clipboard.writeText(`${location.origin}/u/${link.id}`);
}