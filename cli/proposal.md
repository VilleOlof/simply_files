# simply_files_cli

A CLI to interact with a simply_files server.  
Should allow for most features that the web client has atleast.  

## Mockup commands

```bash
# dotfs = .file_system

# if the input isnt exactly 10 chars
# and has no / nor extension, we treat it has an id
# file_identifier = <file_id|file_path>

# All file paths are based from the root directory
# specified in the server config
# so (/) or () is the root directory.
# all "directory" arguments should be `Option<String>.unwrap_or("");`

# Uploads a file to the server.
# Returns the file data if successful.
# if no --access flag is provided, it defaults to private.
# If --id is provided, it will upload the file using it has a link_id. (consuming a one-time link)
# Flags could also be "-a public"
[X] dotfs upload <file_path> <directory> --access <public|private> --id <link_id?>
# Downloads a file from the server.
# If <output_path> is not provided, it will be saved in the current directory.
# With the same name as the file on the server.  
# If --metadata is provided, it will only return the metadata of the file.
# If --link is provided, it will return a download link and make the file public.  
[X] dotfs get <file_identifier> <output_path?> --metadata --link
# Removes a file or directory from the server.
# Only deletes empty directories.
[ ] dotfs rm <file_identifier>
# Renames a file or directory on the server.
[ ] dotfs mv <file_identifier> <new_name>
# Lists files in a directory on the server.
[ ] dotfs ls <directory>
# Creates a new directory on the server.
[ ] dotfs mkdir <directory>
# Changes a files access to public or private.
[X] dotfs access <file_identifier> <public|private>
# Creates a one-time upload link.
[ ] dotfs link create
# Deletes a one-time upload link.
[ ] dotfs link rm <link_id>
# Lists all one-time upload links.
[ ] dotfs link ls
# Sets the host for the web client.  
# Used for getting the actual one-time upload links.  
# Otherwise 'link ls' just returns the IDs
[ ] dotfs link sethost <web_host>
# Tests your upload and download speed to the server.
[ ] dotfs speedtest
# Adds a new server entry
[X] dotfs auth add <name> <url> <token>
# Lists all server entries
[X] dotfs auth ls
# Removes a server entry
[X] dotfs auth rm <name>
# Sets the current server entry to use for commands
[X] dotfs auth set <name>
# Syncs the local file system with the server database.
[ ] dotfs sync
# Returns the current server log file.  
[ ] dotfs log
# Returns the server configuration.
# Only fields like (addr, db, file_system, upload_limit, 
# storage_limit, upload_timeout, redacted ssh config)
[ ] dotfs config
# Returns the current server statistics.
# Total allocated storage, used storage, free storage, total files,  
# total bytes downloaded, total downloads
[ ] dotfs stats
# Removes files that havent fully been uploaded but still recorded in the database.  
[ ] dotfs cleanup
# dotfs help
[ ] dotfs help
```