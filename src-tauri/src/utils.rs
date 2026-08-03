use semver::Version;

pub fn is_newer_version(local: &str, remote: &str) -> bool {
    let local_ver = Version::parse(local).unwrap_or_else(|_| Version::new(0, 0, 0));
    let remote_ver =
        Version::parse(remote.trim_start_matches('v')).unwrap_or_else(|_| Version::new(0, 0, 0));
    remote_ver > local_ver
}
