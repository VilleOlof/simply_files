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

export function upload_file(file: File, endpoint: "/m/upload" | "/one-time/upload"): void {
    try {
        let request = new XMLHttpRequest();

        request.open('POST', `${PUBLIC_BACKEND}${endpoint}/${file.name}`);
        request.withCredentials = true;
        request.setRequestHeader('Content-Type', 'application/octet-stream');
        request.setRequestHeader('X-Filename', file.name);

        request.upload.onprogress = (e) => {
            let percent = Math.round((e.loaded / file.size) * 100);
            if (e.total === e.loaded || percent >= 100) {
                percent = 100;

                dispatchEvent(new CustomEvent('upload-complete', {
                    detail: {
                        percent,
                    }
                }));
            }

            dispatchEvent(new CustomEvent('upload-progress', {
                detail: {
                    percent,
                }
            }));
        }

        request.onload = () => {
            if (request.status === 200) {
                console.log("File uploaded successfully.");
            } else {
                console.error("Failed to upload file:", request.statusText);
            }
        }

        request.send(file);
    }
    catch (e) {
        console.error("Error during file upload:", e);

    }
}