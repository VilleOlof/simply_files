import { PUBLIC_BACKEND } from "$env/static/public";
import { notification } from "./toast";

export async function add_directory(path: string): Promise<void> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/directory/${path}`, {
        method: 'POST',
        credentials: 'include'
    });

    if (!response.ok) {
        notification.error(`Failed to create directory: ${response.statusText}`);
    }
}