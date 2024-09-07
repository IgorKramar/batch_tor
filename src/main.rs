use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::thread;

/// Reads the given file and extracts magnet links from it.
/// Only lines starting with "magnet:" are considered valid links.
/// 
/// # Arguments
/// 
/// * `filename` - A string slice that holds the name of the file with magnet links.
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
/// Each line is considered to be a proxy address in the format `IP:PORT`.
/// 
/// # Arguments
/// 
/// * `filename` - A string slice that holds the name of the file with proxy addresses.
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

/// Saves the remaining valid proxies back to the original file, overwriting the old list.
/// This ensures that invalid proxies are no longer retried.
/// 
/// # Arguments
/// 
/// * `filename` - The name of the file to write the valid proxies back into.
/// * `proxies` - A vector of valid proxy addresses to be written to the file.
fn save_socks_proxies(filename: &str, proxies: &[String]) {
    let mut file = File::create(filename).expect("Failed to recreate the proxy list file");
    for proxy in proxies {
        writeln!(file, "{}", proxy).expect("Error writing proxy to file");
    }
}

/// Checks if a SOCKS5 proxy is functional by attempting to connect to rutracker.org via curl.
/// This is done by specifying the proxy with curl's `-x` option and attempting an HTTPS connection.
/// 
/// # Arguments
/// 
/// * `proxy` - A string slice representing the proxy in `IP:PORT` format.
/// 
/// # Returns
/// 
/// * `bool` - Returns true if the proxy passed the curl check, otherwise false.
fn check_proxy_with_curl(proxy: &str) -> bool {
    println!("Checking proxy via curl: {}", proxy);

    let status = Command::new("curl")
        .arg("-x")
        .arg(format!("socks5h://{}", proxy))
        .arg("https://rutracker.org")
        .arg("--max-time")
        .arg("10")  // Timeout set to 10 seconds
        .stdout(Stdio::null()) // Suppress output
        .stderr(Stdio::null()) // Suppress error output
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
/// If the proxy passes the curl check, it is added to the list of valid proxies and used for downloading.
/// The function processes stdout and stderr streams in real-time.
/// 
/// # Arguments
/// 
/// * `magnet_link` - A string slice containing the magnet link of the torrent to download.
/// * `download_dir` - A string slice representing the directory where files should be downloaded.
/// * `proxies` - A mutable reference to a vector of proxy addresses.
/// * `socks_file` - A string slice representing the proxy file that will be updated with valid proxies.
fn download_torrent_via_socks(magnet_link: &str, download_dir: &str, proxies: &mut Vec<String>, socks_file: &str) {
    println!("Starting download for magnet link: {}", magnet_link);
    io::stdout().flush().unwrap(); // Flush buffer to ensure the message is printed immediately

    let mut valid_proxies = Vec::new();

    for proxy in proxies.iter() {
        if !check_proxy_with_curl(proxy) {
            continue; // Skip proxies that fail the curl check
        }

        valid_proxies.push(proxy.clone()); // Store valid proxy

        println!("Proxy {} passed, starting transmission-cli", proxy);
        io::stdout().flush().unwrap(); // Ensure immediate printing

        let mut child = Command::new("transmission-cli")
            .arg(magnet_link)
            .arg("-w")
            .arg(download_dir)
            .arg("--no-incomplete")
            .arg("--debug")
            .env("ALL_PROXY", format!("socks5://{}", proxy))
            .stderr(Stdio::piped()) // Capture stderr
            .stdout(Stdio::piped()) // Capture stdout
            .spawn()
            .expect("Failed to start transmission-cli");

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Read stdout in real-time
        let stdout_reader = io::BufReader::new(stdout);
        let stderr_reader = io::BufReader::new(stderr);

        let stdout_handle = thread::spawn(move || {
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => {
                        println!("{}", line);
                        io::stdout().flush().unwrap(); // Flush after every line
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
                        io::stderr().flush().unwrap(); // Flush after every line
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
            break; // Stop once the download succeeds
        } else {
            println!("Download failed via proxy: {}", proxy);
        }
    }

    // Overwrite the proxy file with the valid proxies
    save_socks_proxies(socks_file, &valid_proxies);

    println!("Finished processing proxies.");
    io::stdout().flush().unwrap(); // Ensure all output is printed
}

fn main() {
    let txt_file = "magnet_links.txt"; // File with magnet links
    let socks_file = "socks.txt";      // File with SOCKS5 proxies
    let download_dir = "/home/hombre/Torrents"; // Directory for downloading torrents

    // Step 1: Extract magnet links from the file
    let magnet_links = extract_magnet_links(txt_file);
    if magnet_links.is_empty() {
        println!("No magnet links found.");
        return;
    }

    // Step 2: Extract the list of SOCKS5 proxies
    let mut proxies = extract_socks_proxies(socks_file);
    if proxies.is_empty() {
        println!("No proxies found.");
        return;
    }

    // Step 3: Attempt to download each magnet link using available proxies
    for link in magnet_links {
        download_torrent_via_socks(&link, download_dir, &mut proxies, socks_file);
    }

    println!("All files downloaded to directory: {}", download_dir);
}
