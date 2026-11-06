use std::path::{Path, PathBuf};

use crate::error::RagError;

/// List of filenames or prefix/suffix patterns that contain credentials or secrets.
const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    ".key",
    ".pem",
    ".p12",
    ".pfx",
    ".pkcs12",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    "credentials.json",
    "service_account.json",
    "secret.json",
    "secrets.json",
    "auth_token",
    "access_token",
];

/// List of directory names that are always ignored from knowledge indexing.
const IGNORED_DIRECTORY_NAMES: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    ".idea",
    ".vscode",
    ".gemini",
    "node_modules",
    "target",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".venv",
    "venv",
    ".dart_tool",
    "build",
    "coverage",
];

/// Validates that `candidate_path` is safely located within `workspace_root` without
/// path traversal, parent directory escapes, or symlink dereferencing outside root.
pub fn validate_safe_workspace_path(
    candidate_path: &Path,
    workspace_root: &Path,
) -> Result<PathBuf, RagError> {
    let path_str = candidate_path.to_string_lossy();

    // Check for explicit traversal components
    for comp in candidate_path.components() {
        if comp.as_os_str() == ".." {
            return Err(RagError::PathTraversal(path_str.to_string()));
        }
    }

    if path_str.contains('\0') {
        return Err(RagError::PathTraversal(
            "Path contains null byte".to_string(),
        ));
    }

    let resolved_path = if candidate_path.is_absolute() {
        candidate_path.to_path_buf()
    } else {
        workspace_root.join(candidate_path)
    };

    // If file exists, ensure canonical path doesn't escape workspace root
    if resolved_path.exists() {
        let canonical_root = workspace_root
            .canonicalize()
            .map_err(|_| RagError::PathTraversal(workspace_root.to_string_lossy().to_string()))?;
        let canonical_file = resolved_path
            .canonicalize()
            .map_err(|_| RagError::PathTraversal(resolved_path.to_string_lossy().to_string()))?;

        if !canonical_file.starts_with(&canonical_root) {
            return Err(RagError::PathOutsideWorkspace {
                path: canonical_file,
                root: canonical_root,
            });
        }
        Ok(canonical_file)
    } else {
        // Normal path resolution
        Ok(resolved_path)
    }
}

/// Returns whether the path should be excluded based on sensitive patterns or build/cache dirs.
pub fn is_sensitive_or_ignored(path: &Path) -> bool {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let lower = path_str.to_ascii_lowercase();

    // Check directory components
    for segment in lower.split('/') {
        if IGNORED_DIRECTORY_NAMES.contains(&segment) {
            return true;
        }
        if segment.starts_with(".env") {
            return true;
        }
    }

    // Check file name
    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
        let fn_lower = file_name.to_ascii_lowercase();
        for pattern in SENSITIVE_PATTERNS {
            if fn_lower == *pattern || fn_lower.starts_with(pattern) || fn_lower.ends_with(pattern)
            {
                return true;
            }
        }
        // Exclude system junk
        if fn_lower == ".ds_store" || fn_lower == "thumbs.db" {
            return true;
        }
    }

    false
}

/// Inspects byte buffer to determine if content is binary rather than UTF-8 text.
///
/// Looks for null bytes (`\0`) and excessive control characters.
pub fn is_binary_content(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return false;
    }

    let inspect_len = buffer.len().min(1024);
    let sample = &buffer[..inspect_len];

    let mut control_count = 0;
    for &byte in sample {
        // Null byte immediately signals binary
        if byte == 0 {
            return true;
        }
        // Control bytes outside common whitespace (\t, \n, \r)
        if byte < 32 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
            control_count += 1;
        }
    }

    // If control character frequency exceeds 10%, treat as binary
    (control_count as f64 / inspect_len as f64) > 0.10
}

/// Strips the Windows verbatim UNC prefix (`\\?\`) if present.
pub fn strip_unc_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

/// Normalizes path to forward slashes relative to workspace root.
pub fn normalize_relative_path(path: &Path, workspace_root: &Path) -> String {
    let clean_path = strip_unc_prefix(path);
    let clean_root = strip_unc_prefix(workspace_root);

    let rel = clean_path.strip_prefix(&clean_root).unwrap_or(&clean_path);
    let mut normalized = rel.to_string_lossy().replace('\\', "/");
    while normalized.starts_with('/') {
        normalized.remove(0);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_or_ignored() {
        assert!(is_sensitive_or_ignored(Path::new(".env")));
        assert!(is_sensitive_or_ignored(Path::new(".env.local")));
        assert!(is_sensitive_or_ignored(Path::new("server.key")));
        assert!(is_sensitive_or_ignored(Path::new("cert.pem")));
        assert!(is_sensitive_or_ignored(Path::new("id_rsa")));
        assert!(is_sensitive_or_ignored(Path::new("id_ed25519.pub")));
        assert!(is_sensitive_or_ignored(Path::new("src/secrets.json")));
        assert!(is_sensitive_or_ignored(Path::new(
            "node_modules/pkg/index.js"
        )));
        assert!(is_sensitive_or_ignored(Path::new("target/debug/app")));
        assert!(is_sensitive_or_ignored(Path::new(".git/HEAD")));

        assert!(!is_sensitive_or_ignored(Path::new("src/main.rs")));
        assert!(!is_sensitive_or_ignored(Path::new("README.md")));
        assert!(!is_sensitive_or_ignored(Path::new("docs/architecture.txt")));
    }

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content(b"Hello world! This is plain text.\n"));
        assert!(!is_binary_content(b"# Markdown Title\n\nSome paragraph.\n"));

        // Null byte
        assert!(is_binary_content(&[b'H', b'e', b'l', 0, b'o']));

        // High control bytes
        let mut controls = vec![1u8; 50];
        controls.extend(vec![b'A'; 50]);
        assert!(is_binary_content(&controls));
    }

    #[test]
    fn test_validate_safe_workspace_path_traversal() {
        let root = Path::new("g:/miniAI");
        let result = validate_safe_workspace_path(Path::new("../escape.txt"), root);
        assert!(result.is_err());
    }
}
