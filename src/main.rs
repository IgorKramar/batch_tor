use clap::Parser;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::thread;

/// CLI arguments structure
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the magnet links file
    #[arg(short, long, default_value = "magnet_links.txt")]
    magnet_file: String,

    /// Path to the SOCKS5 proxies file
    #[arg(short, long, default_value = "socks.txt")]
    socks_file: String,

    /// Directory to save downloaded torrents
    #[arg(short, long, default_value = "/home/hombre/Torrents")]
    download_dir: String,
}

/// Reads the given file and extracts magnet links.
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

/// Saves the remaining valid proxies back to the original file.
fn save_socks_proxies(filename: &str, proxies: &[String]) {
    let mut file = File::create(filename).expect("Failed to recreate the proxy list file");
    for proxy in proxies {
        writeln!(file, "{}", proxy).expect("Error writing proxy to file");
    }
}

/// Checks if a SOCKS5 proxy is functional via curl.
fn check_proxy_with_curl(proxy: &str) -> bool {
    println!("Checking proxy via curl: {}", proxy);

    let status = Command::new("curl")
        .arg("-x")
        .arg(format!("socks5h://{}", proxy))
        .arg("https://rutracker.org")
        .arg("--max-time")
        .arg("10")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                println!("Proxy {} passed curl check.", proxy);
                true
            } else {
                println!("Proxy {} failed curl check.", proxy);
                false
            }
        }
        Err(e) => {
            println!("Error while checking proxy {}: {:?}", proxy, e);
            false
        }
    }
}

/// Attempts to download a torrent file through a SOCKS5 proxy using transmission-cli.
fn download_torrent_via_socks(
    magnet_link: &str,
    download_dir: &str,
    proxies: &mut Vec<String>,
    socks_file: &str,
    bar: &ProgressBar,
) {
    println!("Starting download for magnet link: {}", magnet_link);
    io::stdout().flush().unwrap();

    let mut valid_proxies = Vec::new();

    for proxy in proxies.iter() {
        if !check_proxy_with_curl(proxy) {
            bar.inc(1); // Update progress for failed proxy
            continue; // Skip proxies that fail the curl check
        }

        valid_proxies.push(proxy.clone());

        println!("Proxy {} passed, starting transmission-cli", proxy);
        io::stdout().flush().unwrap();

        let mut child = Command::new("transmission-cli")
            .arg(magnet_link)
            .arg("-w")
            .arg(download_dir)
            .arg("--no-incomplete")
            .arg("--debug")
            .env("ALL_PROXY", format!("socks5://{}", proxy))
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start transmission-cli");

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_reader = io::BufReader::new(stdout);
        let stderr_reader = io::BufReader::new(stderr);

        let stdout_handle = thread::spawn(move || {
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => {
                        println!("{}", line);
                        io::stdout().flush().unwrap();
                    }
                    Err(err) => eprintln!("Error reading stdout: {}", err),
                }
            }
        });

        let stderr_handle = thread::spawn(move || {
            for line in stderr_reader.lines() {
                match line {
                    Ok(line) => {
                        eprintln!("{}", line);
                        io::stderr().flush().unwrap();
                    }
                    Err(err) => eprintln!("Error reading stderr: {}", err),
                }
            }
        });

        let status = child.wait().expect("Failed to wait for transmission-cli to finish");
        stdout_handle.join().expect("Failed to join stdout thread");
        stderr_handle.join().expect("Failed to join stderr thread");

        if status.success() {
            println!("Download succeeded via proxy: {}", proxy);
            break;
        } else {
            println!("Download failed via proxy: {}", proxy);
        }
        bar.inc(1); // Update progress after each proxy attempt
    }

    save_socks_proxies(socks_file, &valid_proxies);
    println!("Finished processing proxies.");
    io::stdout().flush().unwrap();
}

fn main() {
    let args = Args::parse(); // Parse CLI arguments

    // Extract magnet links
    let magnet_links = extract_magnet_links(&args.magnet_file);
    if magnet_links.is_empty() {
        println!("No magnet links found.");
        return;
    }

    // Extract SOCKS5 proxies
    let mut proxies = extract_socks_proxies(&args.socks_file);
    if proxies.is_empty() {
        println!("No proxies found.");
        return;
    }

    // Set up progress bar
    let bar = ProgressBar::new(proxies.len() as u64);

    // Attempt to download each magnet link
    for link in magnet_links {
        download_torrent_via_socks(&link, &args.download_dir, &mut proxies, &args.socks_file, &bar);
    }

    bar.finish();
    println!("All files downloaded to directory: {}", args.download_dir);
}
