# simply_files

http://simply.localhost:5173/  


## TODO
- [X] Abstract the stream upload handler to be reusable for one-time uploads.
- [ ] Fix all file related endpoints for /m
- [ ] Fix a sqlite DB to sync file ids to their paths & statistics.  
- [ ] One-time links (https://simply.lifelike.dev/u/<link_id>) for file uploads.
- [X] Proper handler for downloading files (https://simply.lifelike.dev/d/<file_id>)
- [ ] Update so the client speed test adjusts to the current upload/download speed.  
- [ ] Run db backups every so often onto the file_system.  

how a raw file upload url may look like: *(aside from headers n stuff)*  
POST https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv

## DB stuff
```sql
-- Table for one time links
CREATE TABLE IF NOT EXISTS links (
    id TEXT PRIMARY KEY, -- the unique id for the link, used in the url  
    uploaded_file TEXT, -- the file id that was uploaded  
    uploaded_at TIMESTAMP, -- when that file was uploaded  
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- when the link was created  
)
```
```sql
-- Table for file ids
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY, -- the unique id for the file, used in the url
    path TEXT NOT NULL, -- (maybe just the full path on the server incase root is changed?). Used to locate the file  
    size INTEGER DEFAULT 0, -- the size of the file in bytes, mostly used for stats and for fast access
    download_count INTEGER DEFAULT 0, -- download count
    last_downloaded_at TIMESTAMP, -- when the file was last downloaded
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- when the file was uploaded  
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- when anything about the file was updated  
)
```
```sql
CREATE INDEX IF NOT EXISTS idx_files_path ON files (path);
```
