# simply_files

http://simply.localhost:5173/  


## TODO
- [X] Abstract the stream upload handler to be reusable for one-time uploads.
- [ ] Fix all file related endpoints for /m
- [ ] Fix a sqlite DB to sync file ids to their paths & statistics.  
- [/] One-time links (https://simply.lifelike.dev/u/<link_id>) for file uploads.
- [X] Proper handler for downloading files (https://simply.lifelike.dev/d/<file_id>)
- [ ] Update so the client speed test adjusts to the current upload/download speed.  
- [ ] Run db backups every so often onto the file_system.  

how a raw file upload url may look like: *(aside from headers n stuff)*  
POST https://simply-backend.lifelike.dev/m/upload/media/content/2025-05-23%2024-52.mkv

## DB stuff
```sql
CREATE INDEX IF NOT EXISTS idx_files_path ON files (path);
```
