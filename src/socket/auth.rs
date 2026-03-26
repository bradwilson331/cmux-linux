/// Validate that the connecting process's UID matches the app owner's UID.
/// Uses SO_PEERCRED on Linux. Returns Ok(true) if uid matches, Ok(false) if rejected.
#[allow(dead_code)]
pub fn validate_peer_uid(_stream: &tokio::net::UnixStream) -> std::io::Result<bool> {
    todo!("SOCK-06: implement SO_PEERCRED uid check — Plan 02")
}

#[cfg(test)]
mod tests {
    /// SOCK-06: SO_PEERCRED must reject connections from UIDs != app owner UID.
    /// This test is a design stub — full implementation in Plan 02.
    #[test]
    #[ignore = "stub: implement in Plan 02"]
    fn test_peercred_rejection() {
        todo!("SOCK-06")
    }
}
