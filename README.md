# BatchTor: Asynchronous Torrent Downloader via SOCKS5 Proxies with CLI Support

BatchTor is a SOCKS5 proxy-based torrent downloader that allows you to download torrents using magnet links through a customizable command-line interface (CLI). The program asynchronously checks proxies and utilizes the first valid proxy for downloading torrents. It supports concurrency for proxy checking and provides a progress bar to track the status of proxy verification.

## Key Features

- **SOCKS5 Proxy Support**: Downloads torrents via SOCKS5 proxies.
- **Asynchronous Proxy Checking**: Proxies are checked in parallel, and the first valid proxy is used for downloading.
- **CLI Interface**: Allows specifying input files for magnet links and proxies, as well as the directory to save downloaded torrents.
- **Proxy Race**: Once a valid proxy is found, further proxy checks are stopped.
- **Progress Bar**: A visual progress bar tracks the proxy verification process.

## Requirements

- **Rust**: This project is written in Rust. You need to have Rust installed to run it. You can install Rust by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).
- **transmission-cli**: Ensure that `transmission-cli` is installed. This is required for torrent downloading.
  
  ```bash
  sudo apt install transmission-cli
  ```

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd <repository-directory>
   ```

2. Build the project using Cargo:
   ```bash
   cargo build
   ```

3. Prepare the input files:
   - `magnet_links.txt`: A file containing magnet links (one per line).
   - `socks.txt`: A file containing a list of SOCKS5 proxies in the format `IP:PORT` (one per line).

## Usage

You can specify the magnet links file, SOCKS5 proxies file, and download directory via the command-line options.

```bash
cargo run -- --magnet-file="magnet_links.txt" --socks-file="socks.txt" --download-dir="/path/to/download"
```

### Example `magnet_links.txt`

```txt
magnet:?xt=urn:btih:EXAMPLEHASH&dn=Example+Torrent+Name
magnet:?xt=urn:btih:ANOTHERHASH&dn=Another+Torrent+Name
```

### Example `socks.txt`

```txt
192.168.1.100:1080
123.456.789.10:8080
```

### CLI Options

- `--magnet-file`: The path to the file containing magnet links. Default is `magnet_links.txt`.
- `--socks-file`: The path to the file containing SOCKS5 proxies. Default is `socks.txt`.
- `--download-dir`: The directory where the torrents will be downloaded. Default is `/home/hombre/Torrents`.

## How It Works

1. **Proxy Check**: The program asynchronously checks all the proxies listed in the `socks.txt` file using `curl`. The first valid proxy that successfully connects to `rutracker.org` is used for torrent downloading.
2. **Torrent Download**: Once a valid proxy is found, the program uses `transmission-cli` to download torrents from the provided magnet links using the specified proxy.
3. **Progress Bar**: The progress of the proxy checking process is displayed using a progress bar, giving visual feedback as proxies are checked.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests with improvements or bug fixes.

## License

This project is licensed under the MIT License.