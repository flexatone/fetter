use sha2::{Digest, Sha256};
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::SystemTime;

use std::os::unix::fs::PermissionsExt;

//------------------------------------------------------------------------------

pub(crate) type ResultDynError<T> = Result<T, Box<dyn std::error::Error>>;

pub(crate) const DURATION_0: Duration = Duration::from_secs(0);

//------------------------------------------------------------------------------

// Normalize all names
pub(crate) fn name_to_key(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}

/// Remove whitespace and a leading "@" if found. Note: this owns the passed String as this is appropriate for the context in which it is used.
pub(crate) fn url_trim(mut input: String) -> String {
    input = input.trim().to_string();
    if input.starts_with('@') {
        input.remove(0);
        input = input.trim().to_string();
    }
    input
}

pub(crate) fn url_strip_user(url: &String) -> String {
    if let Some(pos_protocol) = url.find("://") {
        let pos_start = pos_protocol + 3;
        // get span to first @ if it exists
        if let Some(pos_span) = url[pos_start..].find('@') {
            let pos_end = pos_start + pos_span + 1;
            if url[pos_start..pos_end].find('/').is_none() {
                return format!("{}{}", &url[..pos_start], &url[pos_end..]);
            }
        }
    }
    url.to_string()
}

const PY_SYS_EXE: &str = "import sys;print(sys.executable)";

// Use the default Python to get absolute path to the exe. Use "-S" to skip site configuration.
pub(crate) fn get_absolute_path_from_exe(executable: &str) -> Option<PathBuf> {
    match Command::new(executable)
        .arg("-S")
        .arg("-c")
        .arg(PY_SYS_EXE)
        .output()
    {
        Ok(output) => match std::str::from_utf8(&output.stdout) {
            Ok(s) => Some(PathBuf::from(s.trim())),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

//------------------------------------------------------------------------------

// Determine if the Path is an exe; must be an absolute path.
fn is_python_exe_file_name(path: &Path) -> bool {
    match path.file_name().and_then(|f| f.to_str()) {
        Some(name) if name.starts_with("python") => {
            let suffix = &name[6..];
            // NOTE: this will not work for windows .exe
            suffix.is_empty() || suffix.chars().all(|c| c.is_ascii_digit() || c == '.')
        }
        _ => false,
    }
}

// Return True if the absolute path points to a python executable. We assume this has already been proven to exist.
pub(crate) fn is_python_exe(path: &Path) -> bool {
    if is_python_exe_file_name(path) {
        match fs::metadata(path) {
            Ok(md) => md.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    } else {
        false
    }
}

pub(crate) fn path_home() -> Option<PathBuf> {
    if env::consts::OS == "windows" {
        env::var_os("USERPROFILE").map(PathBuf::from)
    } else {
        env::var_os("HOME").map(PathBuf::from)
    }
}

const IO_FETTER: &str = "io.fetter";

// TOOD: return error instead of option
pub(crate) fn path_cache(create: bool) -> Option<PathBuf> {
    let cache_path = if env::consts::OS == "windows" {
        env::var_os("LOCALAPPDATA").map(|local_app_data| {
            let mut path = PathBuf::from(local_app_data);
            path.push(IO_FETTER);
            path.push("Cache");
            path
        })
    } else if env::consts::OS == "macos" {
        path_home().map(|mut path| {
            path.push("Library");
            path.push("Caches");
            path.push(IO_FETTER);
            path
        })
    } else {
        path_home().map(|mut path| {
            path.push(".cache");
            path.push(IO_FETTER);
            path
        })
    };
    if create {
        if let Some(ref path) = cache_path {
            if let Err(e) = fs::create_dir_all(path) {
                eprintln!("Failed to create cache directory: {}", e);
                return None;
            }
        }
    }
    cache_path
}

/// Given a Path, make it absolute, either expanding `~` or prepending current working directory.
pub(crate) fn path_normalize(path: &Path) -> ResultDynError<PathBuf> {
    let mut fp = path.to_path_buf();
    if let Some(path_str) = fp.to_str() {
        if path_str.starts_with('~') {
            let home = path_home().ok_or("Cannot get home directory")?;
            let path_stripped =
                fp.strip_prefix("~").map_err(|_| "Failed to strip prefix")?;
            fp = home.join(path_stripped);
            println!("post conversion: {:?}", fp);
        }
    }
    if fp.is_relative() {
        let cwd = env::current_dir().map_err(|e| e.to_string())?;
        fp = cwd.join(fp);
    }
    Ok(fp)
}

/// Optimal routine to determine if a Path has only one component. A single component at the root directory ("/bin") has two components and will return false.
pub(crate) fn path_is_component(path: &Path) -> bool {
    let mut components = path.components();
    components.next().is_some() && components.next().is_none()
}

pub(crate) fn exe_path_normalize(path: &Path) -> ResultDynError<PathBuf> {
    let mut fp = path.to_path_buf();
    // if given a single-component path that is a Python name, call it to get the full path to the exe
    if is_python_exe_file_name(path) && path_is_component(path) {
        fp = match path.file_name().and_then(|f| f.to_str()) {
            Some(name) => get_absolute_path_from_exe(name).ok_or_else(|| {
                format!("cannot get absolute path from exe: {:?}", path)
            })?,
            None => {
                let msg = format!("cannot get absolute path from exe: {:?}", path);
                return Err(msg.into());
            }
        };
    }
    path_normalize(&fp)
}

pub(crate) fn path_within_duration<P: AsRef<Path>>(
    cache_path: P,
    max_dur: Duration,
) -> bool {
    if let Ok(metadata) = fs::metadata(&cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(dur) = SystemTime::now().duration_since(modified) {
                return dur <= max_dur;
            }
        }
    }
    false
}

/// Create a hash of an iterable of PathBuf plus an additional Boolean flag (used for the usite configuration option).
pub(crate) fn hash_paths(paths: &[PathBuf], flag: bool) -> String {
    let mut ps: Vec<PathBuf> = paths.to_owned();
    ps.sort();

    let concatenated = ps
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join("\n");

    let input = format!("{concatenated}\n{}", flag);
    // println!("hash_paths input: {:?}", input);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    hash.iter().fold(String::new(), |mut acc, byte| {
        write!(&mut acc, "{:02x}", byte).unwrap();
        acc
    })
}

// pub(crate) fn hash_paths<I, T>(paths: I, flag: bool) -> String
// where
//     I: IntoIterator<Item = T>,
//     T: AsRef<Path>,
// {
//     let mut ps: Vec<&Path> = paths.into_iter().map(|path| path.as_ref()).collect();
//     ps.sort();

//     let concatenated = ps
//         .iter()
//         .map(|path| path.to_string_lossy())
//         .collect::<Vec<_>>()
//         .join("\n");

//     let input = format!("{concatenated}\n{}", flag);
//     let mut hasher = Sha256::new();
//     hasher.update(input.as_bytes());
//     let hash = hasher.finalize();

//     hash.iter().fold(String::new(), |mut acc, byte| {
//         write!(&mut acc, "{:02x}", byte).unwrap();
//         acc
//     })
// }

//------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Component;

    use tempfile::tempdir;

    #[test]
    fn test_url_strip_user_a() {
        let s1 = "file:///localbuilds/pip-1.3.1-py33-none-any.whl".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s1, s2)
    }

    #[test]
    fn test_url_strip_user_b() {
        let s1 = "file://foo@/localbuilds/pip-1.3.1-py33-none-any.whl".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s2, "file:///localbuilds/pip-1.3.1-py33-none-any.whl")
    }

    #[test]
    fn test_url_strip_user_c() {
        let s1 = "https://github.com/pypa/pip/archive/1.3.1.zip#sha1=da9234ee9982d4bbb3c72346a6de940a148ea686".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s2, s1)
    }

    #[test]
    fn test_url_strip_user_d() {
        let s1 = "git+https://git.repo/some_pkg.git@1.3.1".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s2, s1)
    }

    #[test]
    fn test_url_strip_user_e() {
        let s1 = "git+ssh://git@github.com/uqfoundation/dill.git@0.3.8".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s2, "git+ssh://github.com/uqfoundation/dill.git@0.3.8")
    }

    #[test]
    fn test_url_strip_user_f() {
        let s1 = "git+https://foo@github.com/pypa/packaging.git@cf2cbe2aec28f87c6228a6fb136c27931c9af407".to_string();
        let s2 = url_strip_user(&s1);
        assert_eq!(s2, "git+https://github.com/pypa/packaging.git@cf2cbe2aec28f87c6228a6fb136c27931c9af407")
    }

    #[test]
    fn test_path_normalize_a() {
        let p1 = Path::new("~/foo/bar");
        let p2 = path_normalize(&p1).unwrap();
        let home = path_home().unwrap();
        assert!(p2.starts_with(home));
    }

    #[test]
    fn test_path_cache_a() {
        let p1 = path_cache(false).unwrap();
        let result = p1.components().any(|component| match component {
            Component::Normal(name) => name == IO_FETTER,
            _ => false,
        });
        assert_eq!(result, true);
    }

    #[test]
    fn test_path_within_duration_a() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("foo.txt");
        let _ = File::create(fp.clone()).unwrap();
        assert!(path_within_duration(&fp, Duration::from_secs(60)));
        assert!(!path_within_duration(&fp, Duration::from_nanos(1)));
    }

    #[test]
    fn test_is_python_exe_file_name_a() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("python3");
        assert!(is_python_exe_file_name(&fp));
    }

    #[test]
    fn test_is_python_exe_file_name_b() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("python--");
        assert!(!is_python_exe_file_name(&fp));
    }

    #[test]
    fn test_is_python_exe_file_name_c() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("python3.12.1000");
        assert!(is_python_exe_file_name(&fp));
    }

    #[test]
    fn test_path_is_component_a() {
        let fp = PathBuf::from("python3.12.1000");
        assert!(path_is_component(&fp));
    }

    #[test]
    fn test_path_is_component_b1() {
        let fp = PathBuf::from("/foo");
        assert!(!path_is_component(&fp));
    }

    #[test]
    fn test_path_is_component_b2() {
        let fp = PathBuf::from("/bin");
        assert!(!path_is_component(&fp));
    }

    #[test]
    fn test_path_is_component_c() {
        let fp = PathBuf::from("/foo/bar");
        assert!(!path_is_component(&fp));
    }

    #[test]
    fn test_hash_paths_a() {
        let paths = vec![
            Path::new("/a/foo/bar").to_path_buf(),
            Path::new("/b/foo/bar").to_path_buf(),
        ];
        let hashed = hash_paths(&paths, true);
        assert_eq!(
            hashed,
            "aa1e51b6cc2de01f6180c646bd9fe6e5c548bdee475a212747588edc5b0d741b"
        )
    }

    #[test]
    fn test_hash_paths_b() {
        let paths = vec![Path::new("*").to_path_buf()];
        let hashed = hash_paths(&paths, true);
        assert_eq!(
            hashed,
            "e55c287546ecb742e64cae60f41e128a082b290f663f2e03f734b1d82d2ad274"
        )
    }
}
