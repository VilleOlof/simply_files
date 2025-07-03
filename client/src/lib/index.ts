import { goto } from "$app/navigation";
import { PUBLIC_BACKEND } from "$env/static/public";
import { error } from "@sveltejs/kit";

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

export async function get_file_from_path(path: string): Promise<DBFile> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/translate_path/${path}`, {
        credentials: 'include'
    });

    if (!response.ok) {
        console.error("Failed to fetch file:", response.statusText);
        throw new Error(`Failed to fetch file: ${response.statusText}`);
    }

    const data: DBFile = await response.json();
    return data;
}


export async function get_file_system(token: string): Promise<FileSystemInfo> {
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

export async function get_storage_limit(token: string): Promise<StorageLimit> {
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

export async function add_directory(path: string): Promise<void> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/directory/${path}`, {
        method: 'POST',
        credentials: 'include'
    });

    if (!response.ok) {
        console.error("Failed to create directory:", response.statusText);
        throw new Error(`Failed to create directory: ${response.statusText}`);
    }
}

export async function change_access(file: FileMetadata, access: number): Promise<void> {
    let cleaned = get_good_path(file);

    const response = await fetch(`${PUBLIC_BACKEND}/m/access/${cleaned}?access=${access}`, {
        method: 'POST',
        credentials: 'include',
    });

    if (!response.ok) {
        console.error("Failed to change access:", response.statusText);
        throw new Error(`Failed to change access: ${response.statusText}`);
    }
}

export function clean_path(path: string): string {
    if (path.startsWith('/m/') || path.startsWith('/m')) path = path.slice(3);
    if (path.startsWith('/o/') || path.startsWith('/o')) path = path.slice(3);
    if (path.startsWith('/')) path = path.slice(1);
    return path;
}

export const get_good_path = (file: FileMetadata): string => {
    let path = clean_path(window.location.pathname);
    path = path + (path.endsWith('/') ? '' : '/') + file.path;
    if (path.startsWith('/')) path = path.slice(1); // remove leading slash if exists
    return path;
};

export type StorageLimit = {
    used: number;
    max: number;
};

export type FileSystemInfo = {
    which: string,
    about: string,
}

export type DBFile = {
    id: string,
    path: string,
    size: number,
    download_count: number,
    last_downloaded_at?: Date,
    created_at: Date,
    updated_at: Date,
    access: number,
}

export type FileMetadata = {
    path: string,
    is_dir: boolean,
    size: number,
    modified: number,
    id: string,
    access: number,
}
