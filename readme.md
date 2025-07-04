# simply_files

http://simply.localhost:5173/  


## TODO
- [X] Abstract the stream upload handler to be reusable for one-time uploads.
- [/] Fix all file related endpoints for /m
- [X] Fix a sqlite DB to sync file ids to their paths & statistics.  
- [/] One-time links (https://simply.lifelike.dev/u/<link_id>) for file uploads.
- [X] Proper handler for downloading files (https://simply.lifelike.dev/d/<file_id>)
- [ ] Update so the client speed test adjusts to the current upload/download speed.  
- [ ] Run db backups every so often onto the file_system.  
- [X] Add privacy toggle (auto make it public when copying link)  
- [X] Remove directories (only when empty)  
- [ ] toasts on client  
- [ ] do a refresh on the entire files & db at some point  
- [X] fully fix rename (client and sync with db on server)  
- [X] notify that dir deletions only work on empty directories
- [X] add checks so a path before going into the file_system is NEVER begins with a /. because we dont wanna delete the root directory or even attempt it.  
- [ ] mobile support in css (upload, download, etc)  
- [ ] Fix some page or way to add new one-time links and that stuff
- [X] Fix UI on one-time upload links  
- [ ] remove unwraps  
- [X] Auto create Local/SSH root folder on "first run" &  ".public_uploads"
- [ ] Either upload queue or parallel multi-uploads
- [ ] Organize client "index.ts"
- [ ] add storage_limit config & check it on upload  