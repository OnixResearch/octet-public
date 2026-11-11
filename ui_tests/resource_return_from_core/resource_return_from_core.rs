fn open_resource_in_core() -> std::fs::File {
    loop {}
}

fn spawn_handle_in_core() -> std::thread::JoinHandle<()> {
    loop {}
}

// Resource handles nested in wrappers, behind aliases, or next to
// description-like names are still concrete resources.
struct ResourceRequest;

type PortFile = std::fs::File;

fn open_with_request_in_core() -> (ResourceRequest, std::fs::File) {
    loop {}
}

fn open_port_file_in_core() -> PortFile {
    loop {}
}

fn connect_in_core() -> std::io::Result<std::net::TcpStream> {
    loop {}
}

fn build_process_in_core() -> std::process::Command {
    loop {}
}

fn subscribe_in_core() -> std::sync::mpsc::Receiver<u8> {
    loop {}
}

async fn open_async_in_core() -> std::fs::File {
    loop {}
}

// Domain values whose names merely contain resource words stay clean.
struct FileId(u32);
struct ClientId(u32);
struct ChildId(u32);

enum Command {
    Start,
    Stop,
}

struct CommandSpec {
    program: &'static str,
}

fn listen_address_in_core() -> std::net::SocketAddr {
    loop {}
}

fn file_id_in_core() -> FileId {
    loop {}
}

fn client_id_in_core() -> ClientId {
    loop {}
}

fn child_id_in_core() -> ChildId {
    loop {}
}

fn next_command_in_core() -> Command {
    loop {}
}

fn command_spec_in_core() -> CommandSpec {
    loop {}
}

fn main() {}
