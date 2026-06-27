use std::process::Command;

const SERVICE: &str = "com.cindrallabs.atlas";

pub fn read_secret(account: &str) -> Option<String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", account, "-w"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let secret = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if secret.is_empty() {
        None
    } else {
        Some(secret)
    }
}

pub fn write_secret(account: &str, secret: &str) -> std::io::Result<()> {
    if secret.is_empty() {
        return Ok(());
    }

    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE, "-a", account])
        .output();

    let status = Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            SERVICE,
            "-a",
            account,
            "-w",
            secret,
            "-U",
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "failed to write secret to macOS Keychain",
        ))
    }
}
