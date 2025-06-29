import { goto } from "$app/navigation";
import { PUBLIC_BACKEND } from "$env/static/public";

export const GITHUB_URL = "https://github.com/VilleOlof/simply_files";

// TODO: Some kind of toasts for errors?

export async function login(token: string): Promise<void> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/authenticate`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({ token })
    });

    if (!response.ok) {
        throw new Error(`Login failed: ${response.statusText}`);
    }

    goto("/m", { replaceState: true });
}

export async function logout(): Promise<boolean> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/logout`, {
        credentials: 'include'
    });

    if (!response.ok) {
        throw new Error(`Logout failed: ${response.statusText}`);
    }

    return true;
}