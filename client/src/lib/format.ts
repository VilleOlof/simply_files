import type { FileMetadata } from "./file";

// try and make this css only?
export function format_path(path: string): string {
    return path;
    const MAX_NAME_LENGTH = 30;

    const name = path.split('.').slice(0, -1).join('.');
    const ext = path.split('.').pop();

    if (name.length > MAX_NAME_LENGTH) {
        return `${name.slice(0, MAX_NAME_LENGTH)}... ${ext ? `.${ext}` : ''}`;
    }

    return path;
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

export function fuckery_rust_time_to_date(time: number[]): Date {
    // [year, days, hour, minute, second, _, _, _, _];
    if (time.length != 9) {
        throw new Error("Invalid time array length");
    }

    const [year, days, hour, minute, second] = time;
    const date = new Date(Date.UTC(year, 0, days, hour, minute, second));
    date.setMinutes(date.getMinutes() + date.getTimezoneOffset());

    return date;
}