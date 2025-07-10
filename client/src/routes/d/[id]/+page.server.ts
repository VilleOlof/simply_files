import { PUBLIC_BACKEND } from '$env/static/public';
import { get_download_link, type FilePreviewData } from '$lib/file.js';
import { error } from '@sveltejs/kit';

export async function load({ params, cookies }) {
    const { id } = params;
    const token = cookies.get('token');
    if (!id) {
        throw error(400, 'ID parameter is required');
    }

    const response = await fetch(`${PUBLIC_BACKEND}/preview_data/${id}`, {
        headers: token ? {
            Authorization: `Bearer ${token}`
        } : {}
    });
    if (!response.ok) {
        throw error(response.status, response.statusText);
    }
    let data: FilePreviewData = await response.json();
    // just for the delete popup
    // @ts-ignore
    data.is_dir = false;

    return {
        id,
        meta: data,
        url: get_download_link(id) + "?r=t&p=t",
        raw_url: get_download_link(id),
        has_token: token ? true : false
    }
}