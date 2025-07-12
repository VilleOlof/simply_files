# simply_packet

A packet format for uploading files via websockets.  
This format is used by this project ('simply_files') to upload files.  
The format makes it easy to upload files that can be resumed at *any* point.  

- <a href="#packet-format">Packet Format</a>
- <a href="#binary-0u8">Binary Packet</a>
- <a href="#json-1u8">JSON Packet</a>
- <a href="#next-2u8">Next Packet</a>
- <a href="#client-and-server-communication">Client & Server Communication</a>
- <a href="#implementations">Implementations</a>

## Packet Format

This is a custom binary packet format with some JSON for instruction messages.  

When a file is uploaded, the client decided on how many chunks to split the file into.  
Once the client has communicated to the server some file metadata.  
And the server has acknowledged that, the client can start sending binary chunk packets.  


Types of packets:  
    - **Binary**: Contains raw file chunk data.  
    - **JSON**: Contains instructions that the client or server sends to each other.  
    - **Next**: Tells the client to send the next chunk of data.  

Each type corresponds to a byte:  
| Type   | Byte |
|--------|------|
| Binary | 0u8 |
| JSON   | 1u8 |
| Next   | 2u8 |

So a complete packet will look something like this:
```
┌─────────────┬──────────────────────┐  
│ packet type │ packet specific data │
│ 1 byte u8   │ [u8]                 │
└─────────────┴──────────────────────┘
```

## Binary *(0u8)*

Binary packets are only used for sending chunks of file data.  
A binary packet follows a structure like this:  

```
┌─────────────┬──────────────┬──────┐  
│ chunk index │ chunk length │ data │  
│ 8 bytes u64 │ 8 bytes u64  │ [u8] │
└─────────────┴──────────────┴──────┘
```

A file is split into `N` chunks, so the `chunk index` is whatever 0-based index the chunk is in.  
Then the `chunk length` is simply just the byte length of the following `data` field.  
And `data` is the actual file data for that chunk.  

## JSON *(1u8)*

JSON packets are used for sending instructions between the client and server.  
Like the server telling the client that the socket is open & ready for file metadata.  
Or telling the client that the file upload is complete.  

There is a few types of JSON packets that corresponds to different data being sent.  
- *(0)* **ConnectionAccepted**: Sent to the client when the server has accepted the connection & is ready to receive the file metadata.  
- *(1)* **InitializeUpload**: Sent to the server with the file name, size and bytes per chunk.  
- *(3)* **ReadyForUpload**: Sent to the client when the server is ready to receive file chunks, it also includes the starting chunk index that the client should start sending from.  
- *(4)* **SetChunkIndex**: Sent to the client if the server ever wants the client to start sending from a different chunk index, useful if the chunk order is not sequential.  
- *(5)* **UploadComplete**: Sent to the client once the upload is complete on the server.  

A JSON packet follows a structure like this:  
```
┌─────────────┬──────┐  
│ JSON type   │ data │
│ 1 byte u8   │ [u8] │
└─────────────┴──────┘
```

### JSON data types
- **ConnectionAccepted**  
    No extra data, just the type byte.  
    So a complete packet is simply just `[1, 0]` (`1` for JSON packet type and `0` for the JSON type). 
- **InitializeUpload**  
   Contains the file name, size and bytes per chunk.  
   ```json
   {
      "name": "example.png",
      "size": 558275815,
      "chunk_size": 8388608 
   }
   ```
- **ReadyForUpload**  
    Contains only the starting chunk index that the client should start from.  
    ```json
    {
      "chunk_index": 0
    }
    ```
- **SetChunkIndex**  
    Very similar to `ReadyForUpload` but can happen at any point during the upload.  
    Contains the chunk index that the client should start from.  
    ```json
    {
      "chunk_index": 5
    }
    ```
- **UploadComplete**  
    This contains the entire file structure from the server's perspective.  
    In our case, the data directly from the database.  
    ```json
    {
      "id": "j8CkWo1a6p",
      "path": "content/example.png",
      "size": 558275815,
      "download_count": 0,
      "last_downloaded_at": null,
      "created_at": [2025, 145, 14, 15, 0, 0, 0, 0],
      "updated_at": [2025, 145, 14, 15, 0, 0, 0, 0],
      "access": 0,
      "chunk_index": 67,
      "total_chunks": 67
    }
    ```

## Next *(2u8)*

Next packets are a simple 1 byte packet that just tells the client to send the next chunk of data.  
So it contains no extra data and just the type byte. `[2]`  

## Client and Server Communication

Now onto the ordering of packets and how an upload could look like.  

1. **Client**  
    The client begins by opening a websocket connection to the server.  
    And then waits until the server sends a `ConnectionAccepted` packet.  
2. **Server**  
    The server accepts the connection and sends a `ConnectionAccepted` packet to the client  
    once it is ready to continue the upload.  
3. **Client**  
    The client can now send a `InitializeUpload` packet that has some core metadata both  
    about the file and how the file will be split into chunks.  
    The server can calculate the total chunks based on the size and chunk size (size / chunk size).  
4. **Server**  
    Once the server has received the `InitializeUpload`, and done any validation it would need.  
    It can send a `ReadyForUpload` packet, indicating that the client can begin sending file chunks.  
    The server should then continue to listen for binary packets until the upload is complete.  
5. **Client**  
    The client can start sending binary chunk packets, startin from whatever chunk index  
    the server sent in the `ReadyForUpload` packet.  
    If the client ever receives a `SetChunkIndex` packet at this point, it should swap to that  
    chunk index and continue sending chunks from there.  
6. **Server**  
    At this point the server can handle the binary packets however it needs to.  
    Writing them to disk, storing them in memory, a cache etc.  
    If the server notices that the chunk_index is mismatched or out of order.  
    It can send a `SetChunkIndex` packet to the client to tell it to start sending from a different index.  
    Once the server has gotten all the chunks, it can send a `UploadComplete` packet to the client.  
7. **Client**  
    After the client has sent all chunks, it should not close the socket until it has received a   
    `CompleteUpload` packet that contains the final uploaded file's metadata.  
    The client can then close the socket and consider the upload complete.  

## Implementations

There is a Typescript client implementation at `client/src/lib/upload.ts`.  
*Showcases how a client upload to a server looks like*   

And a Rust websocket server implementation at `backend/src/upload/websocket.rs`.  
*Showcases how the server may receive and process the packets*  

And a Rust packet format implementation at `sf_core/src/simply_packet.rs`.  
*Showcases a pure Rust packet format implementation that is both from and to bytes*  