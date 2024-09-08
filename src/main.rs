use clap::Parser;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

/// The `Args` struct defines the CLI interface for the program using the `clap` library.
/// It allows users to specify paths to the magnet links file, the SOCKS5 proxies file,
/// and the directory where the downloaded torrents will be saved.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file containing magnet links. Each line is expected to be a valid magnet link.
    #[arg(short, long, default_value = "magnet_links.txt")]
    magnet_file: String,

    /// The file containing SOCKS5 proxies in the format `IP:PORT`. Each line represents one proxy.
    #[arg(short, long, default_value = "socks.txt")]
    socks_file: String,

    /// The directory where the downloaded torrents will be saved.
    #[arg(short, long, default_value = "/home/hombre/Torrents")]
    download_dir: String,
}

/// Reads the given file and extracts magnet links.
/// The function expects each line in the file to represent a valid magnet link starting with "magnet:".
///
/// # Arguments
///
/// * `filename` - The path to the file containing magnet links.
///
/// # Returns
///
/// A vector of magnet links extracted from the file.
fn extract_magnet_links(filename: &str) -> Vec<String> {
    let file = File::open(filename).expect("Failed to open magnet links file");
    let reader = io::BufReader::new(file);

    let mut links = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line from file");
        if line.starts_with("magnet:") {
            links.push(line);
        }
    }
    links
}

/// Reads the list of SOCKS5 proxies from a file.
/// The function expects each line to be in the format `IP:PORT`, with each line representing one proxy.
///
/// # Arguments
///
/// * `filename` - The path to the file containing proxy addresses.
///
/// # Returns
///
/// A vector of SOCKS5 proxy addresses.
fn extract_socks_proxies(filename: &str) -> Vec<String> {
    let file = File::open(filename).expect("Failed to open proxy list file");
    let reader = io::BufReader::new(file);

    let mut proxies = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line from file");
        if !line.trim().is_empty() {
            proxies.push(line);
        }
    }
    proxies
}

/// Asynchronously checks if a SOCKS5 proxy is functional using `curl`.
/// It attempts to establish a connection to `rutracker.org` through the proxy.
/// The function returns `Some(proxy)` if the proxy is valid, otherwise returns `None`.
///
/// # Arguments
///
/// * `proxy` - A `String` representing the SOCKS5 proxy in the format `IP:PORT`.
///
/// # Returns
///
/// An `Option<String>`, where `Some(proxy)` indicates a valid proxy and `None` indicates a failure.
async fn check_proxy_with_curl(proxy: String) -> Option<String> {
    println!("Checking proxy via curl: {}", proxy);

    let status = Command::new("curl")
        .arg("-x")
        .arg(format!("socks5h://{}", proxy))
        .arg("https://rutracker.org")
        .arg("--max-time")
        .arg("10")  // Sets a 10-second timeout for the connection attempt
        .stdout(Stdio::null())  // Suppresses the output from curl
        .stderr(Stdio::null())  // Suppresses any error messages from curl
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                println!("Proxy {} passed curl check.", proxy);
                Some(proxy)
            } else {
                println!("Proxy {} failed curl check.", proxy);
                None
            }
        }
        Err(e) => {
            println!("Error while checking proxy {}: {:?}", proxy, e);
            None
        }
    }
}

/// Runs a race to find the first valid SOCKS5 proxy from the list.
/// Multiple proxy checks are spawned concurrently, and the function returns the first valid proxy found.
/// If no valid proxy is found, the function returns `None`.
///
/// # Arguments
///
/// * `proxies` - A vector of proxies to be checked.
/// * `bar` - A reference to a progress bar (indicatif) to track the progress of the proxy checks.
///
/// # Returns
///
/// An `Option<String>` containing the first valid proxy, or `None` if no valid proxies are found.
async fn find_valid_proxy(proxies: Vec<String>, bar: Arc<ProgressBar>) -> Option<String> {
    let (tx, mut rx) = mpsc::channel(1);  // A channel to send the result back to the main thread.

    for proxy in proxies {
        let tx = tx.clone();
        let bar = Arc::clone(&bar);
        task::spawn(async move {
            let result = check_proxy_with_curl(proxy.clone()).await;
            if result.is_some() {
                let _ = tx.send(result).await;  // Sends the first valid proxy back through the channel.
            }
            bar.inc(1);  // Increments the progress bar after each proxy check.
        });
    }

    drop(tx);  // Closes the sending side of the channel when all tasks are spawned.

    rx.recv().await.flatten()  // Receives and returns the first valid proxy, flattening nested Option<Option<T>>.
}

/// Downloads a torrent file using the first valid SOCKS5 proxy found.
/// It runs `transmission-cli` to download the torrent and logs both `stdout` and `stderr` to the console in real-time.
///
/// # Arguments
///
/// * `magnet_link` - The magnet link of the torrent to be downloaded.
/// * `download_dir` - The directory where the downloaded files will be saved.
/// * `proxy` - The SOCKS5 proxy to use for the download.
fn download_torrent_via_socks(magnet_link: &str, download_dir: &str, proxy: &str) {
    println!("Starting download for magnet link: {}", magnet_link);
    io::stdout().flush().unwrap();  // Flushes the stdout buffer to ensure the message is printed immediately.

    println!("Using proxy: {} for download", proxy);

    let mut child = Command::new("transmission-cli")
        .arg(magnet_link)
        .arg("-w")
        .arg(download_dir)
        .arg("--no-incomplete")  // Ensures that incomplete downloads are not kept.
        .arg("--debug")  // Enables debug logging for transmission-cli.
        .env("ALL_PROXY", format!("socks5://{}", proxy))  // Sets the proxy environment variable for transmission-cli.
        .stderr(Stdio::piped())  // Captures the stderr stream.
        .stdout(Stdio::piped())  // Captures the stdout stream.
        .spawn()
        .expect("Failed to start transmission-cli");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let stdout_reader = io::BufReader::new(stdout);
    let stderr_reader = io::BufReader::new(stderr);

    // Processes and prints stdout in real-time.
    for line in stdout_reader.lines() {
        match line {
            Ok(line) => {
                println!("{}", line);
                io::stdout().flush().unwrap();  // Flushes after each line to ensure immediate output.
            }
            Err(err) => eprintln!("Error reading stdout: {}", err),
        }
    }

    // Processes and prints stderr in real-time.
    for line in stderr_reader.lines() {
        match line {
            Ok(line) => {
                eprintln!("{}", line);
                io::stderr().flush().unwrap();  // Flushes after each line to ensure immediate output.
            }
            Err(err) => eprintln!("Error reading stderr: {}", err),
        }
    }

    child.wait().expect("Failed to wait for transmission-cli to finish");  // Waits for the transmission-cli process to finish.
}

/// The main entry point of the program. It parses the CLI arguments, extracts magnet links and proxies,
/// runs the race to find a valid proxy, and starts the torrent download using the first valid proxy found.
///
/// # Asynchronous Execution
///
/// The function is marked with `#[tokio::main]` to run asynchronous tasks using the `tokio` runtime.
#[tokio::main]
async fn main() {
    let args = Args::parse();  // Parses CLI arguments using `clap`.

    // Extracts magnet links from the specified file.
    let magnet_links = extract_magnet_links(&args.magnet_file);
    if magnet_links.is_empty() {
        println!("No magnet links found.");
        return;
    }

    // Extracts SOCKS5 proxies from the specified file.
    let proxies = extract_socks_proxies(&args.socks_file);
    if proxies.is_empty() {
        println!("No proxies found.");
        return;
    }

    // Sets up a progress bar to track the proxy-checking process.
    let bar = Arc::new(ProgressBar::new(proxies.len() as u64));

    // Runs the race to find the first valid proxy.
    if let Some(valid_proxy) = find_valid_proxy(proxies, Arc::clone(&bar)).await {
        bar.finish();  // Finishes the progress bar once a valid proxy is found or all proxies are checked.
        for link in magnet_links {
            download_torrent_via_socks(&link, &args.download_dir, &valid_proxy);
        }
    } else {
        println!("No valid proxies found.");
    }
}
