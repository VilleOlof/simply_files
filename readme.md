# simply_files

File sharing done easily.

## Features
- One-time links for others to upload files
- Easy to use interface
- Secure & fast, backend in Rust
- Folders to help you organize
- QR code generation to easily share
- Video/image/audio/text file preview
- File resumability & streaming
- No ads, no payment, 100% free
- Store your files locally or via SFTP
- No AI bullshit

## Installation

This repository contains 2 parts:
1. The backend, this exposes an API to interact with the files.  
2. The client, which is a web interace to interact with the backend.

The backend can run just fine on it's own.

> [!IMPORTANT]  
> Uploading uses websockets, make sure your server supports it.  
> A frame/body size of at least 16.8MB is needed for uploads.  

### Backend

> [!NOTE]  
> The backend requires Rust (V1.85+, 2024 edition) to be installed.  
> You can install it via [rustup](https://rustup.rs/).  

Begin with cloning or downloading the repository.  
Then navigate to the `backend` directory and run the following command to build it:

```bash
cd backend

cargo build --release
# binary can then be located at `target/release/simply_files`
```

Before we can run it tho, we will need to configure it so you can access it.

### Configuration

There is an example config file at `backend/config.example.toml`  
which you can copy to `backend/config.toml` and begin to edit to your liking.  

But we are gonna go through the important parts here.  

#### File System

Currently you can choose to store your files in 2 ways:
- **Local**  
    This will store them on the same host as the backend is running on.  
    The only argument to this is the root directory on where to store the files.  
    ```toml
    file_system = "local"
    [local]
    root = "/home/user/simply_files/data
    ```
- **SFTP**
    This stores the files on a remote server via SFTP.  
    You can authenticate via either password or a public key.  
    ```toml
    file_system = "ssh"
    [ssh]
    host = "example.com"
    port = 22
    username = "user"

    # Choose either
    [ssh.password]
    password = "******"
    # or
    [ssh.public_key]
    private_key = "/home/user/.ssh/id_rsa"
    # These are optional 
    # pass_phrase = "******"
    # public_key = "/home/user/.ssh/id_rsa.pub"
    ```
Then you will need to configure the token that you will use to login and manage your files.  
The backend only controls a single main account.  

```toml
token = "**********"
```

All other configs have default values and documentation in the example config file if you want to configure them.  

### Running the backend

After you've setup the configuration.  

```bash
# Just run the binary
./simply_files

# You can optionally specify a log level as the first argument
./simply_files debug
# [trace, debug, info, warn, error, off] are valid log levels
# Defaults to `info` if not specified.
```

### Client

The client is an easy to use interface that allows full interaction with the backend.  

> [!NOTE]  
> The client requires NPM & Node (V20.6+) to be installed.  
> You can install it [here](https://nodejs.org/en/download)  

Start with cloning or downloading the repository.  
And add a `.env` in the `client` directory with the following content:

```sh
PUBLIC_BACKEND = "https://backend_url.com"
PUBLIC_BACKEND_WS = "wss://backend_url.com"
```

This URL should point to the backend you configured earlier.  
In this `.env` you could also specify the `port` & `host`.  

Then run the following:

```bash
cd client
# Install dependencies
npm i
# Build the client
npm run build
# Run the client
node --env-file=.env build
```
