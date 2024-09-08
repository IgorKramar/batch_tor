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


## How to Use

1. **Prepare the Files**:

   - `magnet_links.txt`: A file containing magnet links (one per line).
   - `socks.txt`: A file containing a list of SOCKS5 proxies in the format `IP:PORT` (one per line).

2. **Run the Program**:

```bash
cargo run -- --magnet-file="magnet_links.txt" --socks-file="socks.txt" --download-dir="/path/to/download"
```

3. **Process**:

   The program will cycle through the SOCKS5 proxies listed in `socks.txt`, download torrents specified in `magnet_links.txt`, and save the files to `/path/to/download`.

## Example `magnet_links.txt`

```txt
magnet:?xt=urn:btih:EXAMPLEHASH&dn=Example+Torrent+Name
magnet:?xt=urn:btih:ANOTHERHASH&dn=Another+Torrent+Name
```

## Example `socks.txt`

```txt
192.168.1.100:1080
123.456.789.10:8080
```

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests for improvements.

## License

This project is licensed under the MIT License.