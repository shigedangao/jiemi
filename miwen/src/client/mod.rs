pub mod server;
pub mod crd;

// Constant
const REQUEST_TIMEOUT: u64 = 30;

/// Get the gRPC http address
fn get_rpc_addr() -> String {
    if let Some(mode) = std::env::var_os("MODE") {
        if mode == "release" {
            return "http://repository-svc:50208".to_owned()
        }
    }

    // use on local dev
    "http://127.0.0.1:50208".to_owned()
}