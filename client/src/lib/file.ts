import { goto, invalidateAll } from "$app/navigation";
import { PUBLIC_BACKEND } from "$env/static/public";
import prettyBytes from "pretty-bytes";
import { get_good_path } from "./format";
import { notification } from "./toast";
import { UploadFile } from "./upload";

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

export type FilePreviewData = {
    size: number,
    file_name: string,
    id: string,
    created_at: number[],
    mime_type: string,
    access: number,
    path?: string
}

export type UploadEndpoint = "/m/upload" | "/o/upload";

export function upload_button(one_time: boolean = false) {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = !one_time; // allow multiple files if not one-time upload
    input.accept = '*';
    input.style.display = 'none';

    input.onchange = (e) => {
        // this moves the job to the DropFile component
        const files = (e.target as HTMLInputElement).files;
        dispatchEvent(new CustomEvent('manual-upload', { detail: { files } }));

        document.body.removeChild(input); // remove the input after use
    };

    document.body.appendChild(input);
    input.click();
}

export function upload_file(file: File, endpoint: UploadEndpoint, path: string, query?: string): void {
    new UploadFile(file, endpoint, path, query);
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
        notification.error(`Failed to fetch files: ${response.statusText}`);
        return [];
    }
    else {
        const data: FileMetadata[] = await response.json();

        return data;
    }
}

export async function rename_file(file: FileMetadata, new_name: string): Promise<void> {
    let cleaned = get_good_path(file.path);
    let new_path = cleaned.split('/').slice(0, -1).join('/') + '/' + new_name;
    if (new_path.startsWith('/')) new_path = new_path.slice(1); // remove leading slash if exists

    const response = await fetch(`${PUBLIC_BACKEND}/m/rename_file/${cleaned}?to=${new_path}`, {
        method: 'POST',
        credentials: 'include',
    });

    if (!response.ok) {
        notification.error(`Failed to rename file: ${response.statusText}`);
    }
}

export async function change_access(file: FileMetadata, access: number): Promise<void> {
    let cleaned = get_good_path(file.path);

    const response = await fetch(`${PUBLIC_BACKEND}/m/access/${cleaned}?access=${access}`, {
        method: 'POST',
        credentials: 'include',
    });

    if (!response.ok) {
        notification.error(`Failed to change access: ${response.statusText}`);
    }
}

export async function change_access_with_id(id: string, access: number): Promise<void> {
    const response = await fetch(`${PUBLIC_BACKEND}/m/access/${id}?access=${access}&id=true`, {
        method: 'POST',
        credentials: 'include',
    });

    if (!response.ok) {
        notification.error(`Failed to change access: ${response.statusText}`);
    }
}

export function get_download_link(file_id: string): string {
    return `${PUBLIC_BACKEND}/d/${file_id}`;
}

export function get_preview_link(file_id: string): string {
    return `${location.origin}/d/${file_id}`;
}