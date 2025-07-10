import { UploadFile } from "./upload"

export type QueueItem = {
    file: File,
    endpoint: UploadFile.UploadEndpoint,
    path: string,
    query?: string
};

export type QueueChanged = {
    queue_length: number,
    current_item: QueueItem | null,
    queue: { name: string, size: number }[],
    new_item?: QueueItem
};

let queue: QueueItem[] = [];

export async function add(item: QueueItem): Promise<void> {
    queue.push(item);

    // this triggers off the upload since its the only one
    // once this completes it will trigger any next one if they exist.
    if (queue.length === 1) {
        upload(item);
        changed(item);
    } else {
        changed(queue[0], item);
    }
}

export function file_complete_handler(e: Event) {
    if (queue.length === 0) {
        console.warn('Upload complete event received but queue is empty.');
        changed(null);
        return;
    }

    const data = (e as CustomEvent).detail as UploadFile.UploadFileComplete;
    const completed_item = queue.shift()!;
    console.log('File completed:', data.file.name, 'Queue length:', queue.length);

    if (!is_same_file(completed_item.file, data.file)) {
        console.warn('Completed file does not match the queued file.');
        return;
    }

    if (queue.length > 0) {
        const next_item = queue[0];
        changed(next_item);
        upload(next_item);
    }
    else {
        changed(null)
    }
}

function is_same_file(file_1: File, file_2: File): boolean {
    return file_1.name === file_2.name && file_1.size === file_2.size && file_1.lastModified === file_2.lastModified;
}

function upload(item: QueueItem): void {
    console.log('Starting upload for:', item.file.name);
    new UploadFile(item.file, item.endpoint, item.path, item.query);
}

export function changed(current: QueueItem | null, new_item?: QueueItem): void {
    dispatchEvent(new CustomEvent('queue-changed', {
        detail: {
            queue_length: queue.length,
            current_item: current,
            queue: queue.map(q => ({
                name: q.file.name,
                size: q.file.size
            })),
            new_item: new_item
        } as QueueChanged
    }));
}