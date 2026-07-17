//! Socket handle hygiene and port probing for the daemon's HTTP listener.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);

/// What a port is actually doing.
///
/// "Is something listening?" is not a yes/no question. Answering it with a lone
/// `connect()` conflates [`Free`](PortState::Free) with
/// [`Blocked`](PortState::Blocked) — the state that bricks startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    /// Nothing holds the address; a daemon can bind it.
    Free,
    /// Bound, and completing handshakes — a real server is accepting.
    Serving,
    /// Bound, but refusing connections: the address is taken so `bind` fails,
    /// yet nothing answers.
    ///
    /// In the wild this means the socket's **owning process is dead** while a
    /// handle to it survives elsewhere — the OS keeps the address reserved but
    /// stops completing handshakes. That is precisely a task holding a dead
    /// daemon's inherited listener; see [`disable_handle_inheritance`].
    ///
    /// Note a *live* listener that simply never calls `accept` is [`Serving`],
    /// not this: the kernel finishes the handshake into the backlog on its
    /// behalf. Such a daemon is caught later by the `/api/health` check.
    ///
    /// [`Serving`]: PortState::Serving
    Blocked,
}

/// Classify a port from the two facts we can observe about it.
///
/// Split out from [`probe_port`] so the decision table is testable without
/// needing to manufacture a dead process holding a live socket.
fn classify(bindable: bool, connectable: bool) -> PortState {
    match (bindable, connectable) {
        (true, _) => PortState::Free,
        (false, true) => PortState::Serving,
        (false, false) => PortState::Blocked,
    }
}

/// Classify `port` on loopback.
///
/// `bind` is the only probe that separates `Free` from `Blocked`: a leaked
/// socket refuses connections while still owning the address. Neither Rust's
/// `TcpListener` nor tokio's sets `SO_REUSEADDR`, so a successful bind really
/// does mean the address is free.
///
/// Inherently racy — another process may take the port immediately after. This
/// is a diagnostic; the daemon's own `bind` stays authoritative.
pub fn probe_port(port: u16) -> PortState {
    let bindable = TcpListener::bind(("127.0.0.1", port)).is_ok();
    // Skip the connect probe (and its timeout) when the bind already settled it.
    let connectable = !bindable && is_connectable(port);
    classify(bindable, connectable)
}

/// Whether something completes a TCP handshake on `port`.
pub fn is_connectable(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).is_ok()
}

/// Clear the OS "inheritable" flag on the daemon's listening socket.
///
/// Windows-only, and load-bearing. mio builds the listener with the plain WinSock
/// `socket()` call, which yields an **inheritable** handle — unlike `std::net`,
/// which passes `WSA_FLAG_NO_HANDLE_INHERIT` to `WSASocketW` precisely to avoid
/// this. `std::process::Command` then spawns every task with
/// `CreateProcessW(bInheritHandles = TRUE)` and no handle allowlist, so each task
/// receives a duplicate of the listener.
///
/// That duplicate outlives the daemon: the port stays bound with nobody
/// accepting, so the next daemon cannot bind and the GUI can never reach
/// `/api/health`. Labalaba then refuses to start until the orphaned task is
/// killed by hand — which is exactly what a tray Quit with tasks still running
/// used to produce.
///
/// Must run before the first task is spawned.
#[cfg(windows)]
pub fn disable_handle_inheritance<S>(socket: &S) -> std::io::Result<()>
where
    S: std::os::windows::io::AsRawSocket,
{
    use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT};

    let handle = socket.as_raw_socket() as usize as HANDLE;
    // SAFETY: `handle` is borrowed from a live socket that outlives this call.
    if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod port_tests {
    use super::*;

    fn free_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    #[test]
    fn unbound_port_is_free() {
        assert_eq!(probe_port(free_port()), PortState::Free);
    }

    #[tokio::test]
    async fn port_with_an_accepting_server_is_serving() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            loop {
                let _ = listener.accept().await;
            }
        });
        assert_eq!(probe_port(port), PortState::Serving);
    }

    /// The startup-bricking state: taken, but refusing connections.
    ///
    /// Reproducing it for real needs a dead process still holding a live socket,
    /// so the decision table is checked directly. The old `connect()`-only probe
    /// classified this as free, which is why the GUI kept spawning a daemon that
    /// could not possibly bind.
    #[test]
    fn taken_but_unconnectable_is_blocked_not_free() {
        assert_eq!(classify(false, false), PortState::Blocked);
    }

    #[test]
    fn classify_covers_every_case() {
        assert_eq!(classify(true, false), PortState::Free);
        assert_eq!(classify(true, true), PortState::Free);
        assert_eq!(classify(false, true), PortState::Serving);
        assert_eq!(classify(false, false), PortState::Blocked);
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Foundation::{GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT};

    fn inherit_flag<S: AsRawSocket>(socket: &S) -> u32 {
        let mut flags: u32 = 0;
        let ok = unsafe { GetHandleInformation(socket.as_raw_socket() as usize as HANDLE, &mut flags) };
        assert_ne!(
            ok,
            0,
            "GetHandleInformation failed: {}",
            std::io::Error::last_os_error()
        );
        flags & HANDLE_FLAG_INHERIT
    }

    #[tokio::test]
    async fn tokio_listener_starts_inheritable_and_is_cleared() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

        // Guards the upstream precondition this workaround exists for. If this
        // fires, mio started creating non-inheritable sockets and
        // `disable_handle_inheritance` is probably redundant — verify, then drop it.
        assert_ne!(
            inherit_flag(&listener),
            0,
            "expected mio's listener to be inheritable"
        );

        disable_handle_inheritance(&listener).unwrap();
        assert_eq!(inherit_flag(&listener), 0);
    }

    #[test]
    fn is_idempotent() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        disable_handle_inheritance(&listener).unwrap();
        disable_handle_inheritance(&listener).unwrap();
        assert_eq!(inherit_flag(&listener), 0);
    }
}
