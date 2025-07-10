import { PUBLIC_BACKEND_WS } from "$env/static/public";
import type { DBFile } from "./file";

export class UploadFile {
    private file: File;
    private path: string;
    private link_upload: boolean = false;

    private socket: WebSocket;
    private static readonly URL: string = `${PUBLIC_BACKEND_WS}`;

    public static readonly CHUNK_SIZE = 16 * 1024 * 1024; // 16 MB
    private chunk_index: number = 0;
    private total_chunks: number = 0;

    private upload_start_time: number = 0;
    private bytes_sent: number = 0;

    private _progress: UploadFile.UploadProgress = UploadFile.UploadProgress.NotStarted;
    private set progress(value: UploadFile.UploadProgress) {
        this._progress = value;
        dispatchEvent(new CustomEvent('upload-status', { detail: { progress: value, file: this.file } }));
    }
    private get progress(): UploadFile.UploadProgress {
        return this._progress;
    }

    constructor(file: File, endpoint: UploadFile.UploadEndpoint, path: string, query?: string) {
        this.file = file;
        this.path = path;
        if (endpoint === '/o/upload') this.link_upload = true;

        const url = `${UploadFile.URL}${endpoint}/${encodeURIComponent(path)}${query ? `?${query}` : ''}`;
        this.socket = new WebSocket(url);
        this.socket.binaryType = 'arraybuffer';

        this.total_chunks = Math.ceil(file.size / UploadFile.CHUNK_SIZE);

        this.setup_socket();
    }

    private setup_socket() {
        this.socket.onopen = () => {
            console.log('WebSocket connection established');
        };
        this.socket.onmessage = async (event: MessageEvent) => {
            await this.handle_message(event);
        };
        this.socket.onerror = (event: Event) => {
            console.error('WebSocket error:', event);
            // <notification>
        };
        this.socket.onclose = (event: CloseEvent) => {
            console.log('WebSocket closed:', event);
            // <notification>
        };
    }

    private close() {
        if (this.socket.readyState === WebSocket.OPEN) {
            this.socket.close();
        }
    }

    private send_initial_data() {
        this.progress = UploadFile.UploadProgress.Preparing;
        this.send_json<UploadFile.InitialUpload>({
            name: this.file.name,
            size: this.file.size,
            chunk_size: UploadFile.CHUNK_SIZE
        }, UploadFile.JsonDataType.InitializeUpload);
    }

    private send_message(data: ArrayBuffer, type: UploadFile.PacketType) {
        if (this.socket.readyState !== WebSocket.OPEN) throw new Error('WebSocket is not open');

        const type_buffer = new ArrayBuffer(1);
        new DataView(type_buffer).setUint8(0, type);

        const total_len = type_buffer.byteLength + data.byteLength;
        const message = new Uint8Array(total_len);
        message.set(new Uint8Array(type_buffer), 0);
        message.set(new Uint8Array(data), type_buffer.byteLength);

        this.socket.send(message.buffer);
    }

    private send_json<T>(data: T, data_type: UploadFile.JsonDataType) {
        const json = new TextEncoder().encode(JSON.stringify(data));

        const type_buffer = new ArrayBuffer(1);
        new DataView(type_buffer).setUint8(0, data_type);

        const total_len = type_buffer.byteLength + json.byteLength;
        const message = new Uint8Array(total_len);
        message.set(new Uint8Array(type_buffer), 0);
        message.set(json, type_buffer.byteLength);

        this.send_message(message.buffer, UploadFile.PacketType.Json);
    }

    private async begin_upload() {
        let startTime = Date.now();
        while (this.socket.readyState !== WebSocket.OPEN) {
            await new Promise(resolve => setTimeout(resolve, 100));
            if (Date.now() - startTime > 5000) { // Timeout after 5 seconds
                throw new Error('WebSocket connection timed out');
            }
        }

        this.upload_start_time = Date.now();
        this.progress = UploadFile.UploadProgress.Uploading;

        await this.next();
    }

    private async next() {
        const start = this.chunk_index * UploadFile.CHUNK_SIZE;
        const end = Math.min(this.file.size, start + UploadFile.CHUNK_SIZE);
        const chunk = await this.file.slice(start, end).arrayBuffer();

        await this.upload_chunk(chunk);

        dispatchEvent(new CustomEvent('upload-progress', {
            detail: {
                file: this.file,
                percent: Math.round(((this.chunk_index + 1) / this.total_chunks) * 100),
                total_bytes: this.file.size,
                bytes_sent: this.bytes_sent,
                chunk_index: this.chunk_index,
                chunk_size: chunk.byteLength,
                total_chunks: this.total_chunks,
                upload_start_time: this.upload_start_time,
            } as UploadFile.UploadFileEventDetail
        }));

        if (this.chunk_index >= this.total_chunks) {
            this.progress = UploadFile.UploadProgress.ClientCompleted;
        }
    }

    private async upload_chunk(chunk: ArrayBuffer) {
        let message = new ArrayBuffer(16 + chunk.byteLength);
        const data_view = new DataView(message);
        data_view.setBigUint64(0, BigInt(this.chunk_index), false);
        data_view.setBigUint64(8, BigInt(chunk.byteLength), false);

        const chunk_array = new Uint8Array(message);
        chunk_array.set(new Uint8Array(chunk), 16);

        this.send_message(message, UploadFile.PacketType.Binary);
        this.bytes_sent += chunk.byteLength;
        this.chunk_index++;
    }

    private async handle_message(event: MessageEvent) {
        const data_view = new DataView(event.data);
        const packet_type = data_view.getUint8(0);

        if (packet_type === UploadFile.PacketType.Binary) {
            // We should never receive binary data on the client
            console.error('Unexpected binary message received');
            return;
        }

        // next gets its own special type so its as fast as possible
        if (packet_type === UploadFile.PacketType.Next) {
            await this.next();
            return;
        }

        const json_data_type = data_view.getUint8(1);
        if (json_data_type === UploadFile.JsonDataType.ConnectionAccepted) {
            this.send_initial_data();
            return;
        }

        const data = new TextDecoder().decode(event.data.slice(2));
        let parsed_data: any;

        try {
            parsed_data = JSON.parse(data);
        }
        catch (e) {
            console.error('Failed to parse JSON:', e);
            return;
        }

        switch (json_data_type) {
            case UploadFile.JsonDataType.ReadyForUpload:
                const chunk_data = parsed_data as UploadFile.ChunkIndex;
                this.chunk_index = chunk_data.chunk_index;

                await this.begin_upload();

                break;
            case UploadFile.JsonDataType.SetChunkIndex:
                const set_chunk_data = parsed_data as UploadFile.ChunkIndex;
                this.chunk_index = set_chunk_data.chunk_index;
                break;
            case UploadFile.JsonDataType.UploadComplete:
                this.progress = UploadFile.UploadProgress.FullyCompleted;
                const db_file = parsed_data as DBFile;

                dispatchEvent(new CustomEvent('upload-complete', {
                    detail: {
                        file: this.file,
                        link_upload: this.link_upload || false,
                        db_file
                    } as UploadFile.UploadFileComplete
                }));

                this.close();
                break;
            default:
                console.error(`Unknown JSON data type: ${json_data_type}`);
                throw new Error(`Unknown JSON data type: ${json_data_type}`);
        }
    }
}

export namespace UploadFile {
    export const JsonDataType = {
        ConnectionAccepted: 0,
        InitializeUpload: 1,
        ReadyForUpload: 2,
        SetChunkIndex: 3,
        UploadComplete: 4,
    } as const;
    export type JsonDataType = typeof JsonDataType[keyof typeof JsonDataType];

    export const PacketType = {
        Binary: 0,
        Json: 1,
        Next: 2
    } as const;
    export type PacketType = typeof PacketType[keyof typeof PacketType];

    export const UploadProgress = {
        NotStarted: 0,
        Preparing: 1,
        Uploading: 2,
        ClientCompleted: 3,
        FullyCompleted: 4,
    } as const;
    export type UploadProgress = typeof UploadProgress[keyof typeof UploadProgress];

    export type ChunkIndex = {
        chunk_index: number
    }

    export type InitialUpload = {
        name: string,
        size: number,
        chunk_size: number,
    }

    export type UploadEndpoint = "/m/upload" | "/o/upload";

    export type UploadFileEventDetail = {
        file: File;
        percent: number;
        total_bytes: number;
        bytes_sent: number;
        chunk_index: number;
        total_chunks: number;
        chunk_size: number;
        upload_start_time: number;
        link_upload?: boolean;
    }

    export type UploadFileComplete = {
        file: File,
        link_upload: boolean,
        db_file: DBFile
    }
}