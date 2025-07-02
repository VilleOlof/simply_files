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

export type UploadEndpoint = "/m/upload" | "/o/upload";

export function upload_file(file: File, endpoint: UploadEndpoint, path: string): void {
    try {
        let request = new XMLHttpRequest();

        request.open('POST', `${PUBLIC_BACKEND}${endpoint}/${path}`);
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
            if (request.status === 201) {
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

export type FileMetadata = {
    path: string,
    is_dir: boolean,
    size: number,
    modified: number,
}

export async function get_files(path: string, token?: string, server?: boolean): Promise<FileMetadata[]> {
    let dir_config = {
        method: 'GET',
        headers: {},
        credentials: 'include'
    };

    if (server && token) {
        dir_config.headers = {
            'Authorization': `Bearer ${token}`
        };
    }

    const response = await fetch(`${PUBLIC_BACKEND}/m/directory${path !== "" ? `/${path}` : ""}`, dir_config as RequestInit);
    if (!response.ok) {
        console.error("Failed to fetch files:", response.statusText);
        throw new Error(`Failed to fetch files: ${response.statusText}`);
    }

    const data: FileMetadata[] = await response.json();

    return data;
}

export function format_path(path: string): string {
    const MAX_NAME_LENGTH = 20;

    const name = path.split('.').slice(0, -1).join('.');
    const ext = path.split('.').pop();

    if (name.length > MAX_NAME_LENGTH) {
        return `${name.slice(0, MAX_NAME_LENGTH)}... ${ext ? `.${ext}` : ''}`;
    }

    return path;
}

export type StorageLimit = {
    used: number;
    max: number;
};

export type FileSystemInfo = {
    which: string,
    about: string,
}