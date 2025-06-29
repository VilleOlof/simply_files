import { PUBLIC_BACKEND } from "$env/static/public";

export async function download(): Promise<number> {
    const start = performance.now();
    const response = await fetch(`${PUBLIC_BACKEND}/speed_test/download`);
    const reader = response.body?.getReader();

    let downloaded = 0;
    while (true) {
        const { done, value } = await reader!.read();
        if (done) break;
        downloaded += value.length;
    }

    const end = performance.now();
    const speed = calculate_speed(downloaded, [start, end]);
    localStorage.setItem("latest_download_speed", speed.toFixed(2));
    return speed;
}

const UPLOAD_SIZE = 100 * 1024 * 1024; // 100 MB
export async function upload(): Promise<number> {
    const data = new Uint8Array(UPLOAD_SIZE);
    const start = performance.now();

    await fetch(`${PUBLIC_BACKEND}/speed_test/upload`, {
        method: 'POST',
        body: data,
        headers: {
            'Content-Type': 'application/octet-stream'
        }
    });

    const end = performance.now();
    const speed = calculate_speed(UPLOAD_SIZE, [start, end]);
    localStorage.setItem("latest_upload_speed", speed.toFixed(2));
    return speed;
}

/**
 * Calculate the speed in Mbps based on the number of bytes transferred and the time taken.
 */
function calculate_speed(bytes: number, [start, end]: [number, number]): number {
    const duration_sec = (end - start) / 1000;
    return (bytes * 8) / (duration_sec * 1_000_000);
}