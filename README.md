# BatchTor: SOCKS5 Torrent Downloader with CLI

This project is a SOCKS5 proxy-based torrent downloader with a customizable command-line interface. It supports specifying paths to magnet links and SOCKS5 proxies files, and a directory to save downloaded torrents.

## Features

- **Custom CLI**: Choose your input files and download directory.
- **SOCKS5 Proxy Support**: Downloads torrents through SOCKS5 proxies.
- **Proxy Health Check**: Uses `curl` to test if a proxy is functional before attempting the download.
- **Progress Bar**: Displays progress as proxies are checked and torrents are downloaded.
- **Proxy List Management**: Automatically removes non-working proxies from the proxy list to avoid retrying them.

## Requirements

- **Rust**: This project is written in Rust. You need to have Rust installed to run it.
- **transmission-cli**:

 Ensure `transmission-cli` is installed.

## Setup

1. Install Rust and `transmission-cli`:
   ```bash
   sudo apt install transmission-cli
   ```

2. Clone the repository:
   ```bash
   git clone <repository-url>
   cd <repository-directory>
   ```

## Usage

```bash
cargo run -- --magnet-file="magnet_links.txt" --socks-file="socks.txt" --download-dir="/path/to/download"
```

The program will cycle through the SOCKS5 proxies listed in `socks.txt`, download torrents specified in `magnet_links.txt`, and save the files to `/path/to/download`.
