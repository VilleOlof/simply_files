import { goto } from "$app/navigation";
import { PUBLIC_BACKEND } from "$env/static/public";
import { notification } from "./toast";

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
        notification.error(`Login failed. (${response.statusText})`);
    }
    else {
        goto("/m", { replaceState: true });
    }
}

export async function logout(): Promise<boolean> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/logout`, {
        credentials: 'include'
    });

    if (!response.ok) {
        notification.error(`Logout failed. (${response.statusText})`);
        return false;
    }
    else {
        return true;
    }
}