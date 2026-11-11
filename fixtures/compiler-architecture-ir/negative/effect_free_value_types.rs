use std::fs::Metadata;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::Ordering;
use std::thread::ThreadId;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::SystemTimeError;

pub fn same_host(left: IpAddr, right: IpAddr) -> bool {
    left == right
}

pub fn loopback(address: Ipv4Addr) -> bool {
    address.is_loopback()
}

pub fn port(address: SocketAddr) -> u16 {
    address.port()
}

pub fn strongest(order: Ordering) -> bool {
    order == Ordering::SeqCst
}

pub fn same_thread(left: ThreadId, right: ThreadId) -> bool {
    left == right
}

pub fn exited(status: ExitStatus) -> bool {
    status.success()
}

pub fn directory(metadata: &Metadata) -> bool {
    metadata.is_dir()
}

pub fn doubled(timeout: Duration) -> Duration {
    timeout * 2
}

pub fn joined(base: &Path) -> PathBuf {
    base.join("child")
}

pub fn connect(address: SocketAddr) -> std::io::Result<TcpStream> {
    TcpStream::connect(address)
}

pub fn read(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn instant_between(later: Instant, earlier: Instant) -> Duration {
    later.duration_since(earlier)
}

pub fn system_between(later: SystemTime, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
    later.duration_since(earlier)
}

pub fn file_name(path: &Path) -> bool {
    path.parent().and_then(Path::file_name).is_some() && path.extension().is_none() && path.starts_with("/")
}

pub fn waited(start: Instant) -> Duration {
    start.elapsed()
}

pub fn age(stamp: SystemTime) -> Result<Duration, SystemTimeError> {
    stamp.elapsed()
}

pub fn present(path: &Path) -> bool {
    path.exists()
}

pub fn present_buffer(path: &PathBuf) -> bool {
    path.is_file()
}

pub fn listing(path: &Path) -> std::io::Result<std::fs::ReadDir> {
    path.read_dir()
}
