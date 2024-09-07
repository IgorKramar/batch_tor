# BatchTor: SOCKS5 Torrent Downloader with Magnet Links

This project is a SOCKS5 proxy-based torrent downloader that leverages `transmission-cli` to download torrents using magnet links. It can cycle through a list of proxies, checking each for availability, and only using functional proxies. Additionally, proxies that fail are removed from the proxy list to avoid being retried in the future.

## Features

- **SOCKS5 Proxy Support**: Downloads torrents through SOCKS5 proxies.
- **Proxy Health Check**: Uses `curl` to test if a proxy is functional before attempting the download.
- **Magnet Links**: Supports downloading torrents using magnet links.
- **Proxy List Management**: Automatically removes non-working proxies from the proxy list to avoid retrying them.

## Requirements

- **Rust**: This project is written in Rust. You need to have Rust installed to run it.
- **transmission-cli**: The command-line tool `transmission-cli` must be installed on your system to handle the torrent download.

## Setup

1. Install Rust by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
2. Ensure `transmission-cli` is installed:
   ```bash
   sudo apt install transmission-cli
   ```
3. Clone the repository:
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
   cargo run
   ```

3. **Process**:
   - The program will attempt to download torrents using the proxies listed in `socks.txt`.
   - Non-working proxies will be removed from the list.

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

