#[cfg(test)]
mod tests {
    use proton_wallet_common::library_version;

    #[test]
    fn test_library_version() {
        let version = library_version();
        assert!(!version.is_empty(), "Version should not be empty");
    }
}