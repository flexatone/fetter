use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use rayon::prelude::*;

use crate::util::get_absolute_path_from_exe;
use crate::util::is_python_exe;
use crate::util::path_home;

//------------------------------------------------------------------------------
// Provide absolute paths for directories that should be excluded from executable search.
fn get_search_exclude_paths() -> HashSet<PathBuf> {
    let mut paths: HashSet<PathBuf> = HashSet::new();
    match path_home() {
        Some(home) => {
            paths.insert(home.clone().join(".cache"));
            paths.insert(home.clone().join(".npm"));

            if env::consts::OS == "macos" {
                paths.insert(home.clone().join("Library"));
                paths.insert(home.clone().join("Photos"));
                paths.insert(home.clone().join("Downloads"));
                paths.insert(home.clone().join(".Trash"));
            } else if env::consts::OS == "linux" {
                paths.insert(home.clone().join(".local/share/Trash"));
            }
        }
        None => {
            eprintln!("Error getting HOME");
        }
    }
    paths
}

// Provide directories that should be used as origins for searching for executables. Returns a vector of PathBuf, bool, where the bool indicates if the directory should be recursively searched.
fn get_search_origins() -> HashSet<(PathBuf, bool)> {
    let mut paths: HashSet<(PathBuf, bool)> = HashSet::new();

    // get all paths on PATH
    if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(':') {
            paths.insert((PathBuf::from(path), false));
        }
    }
    match path_home() {
        Some(home) => {
            paths.insert((home.clone(), false));
            // collect all directories in the user's home directory
            match fs::read_dir(home) {
                Ok(entries) => {
                    for entry in entries {
                        let path = entry.unwrap().path();
                        if path.is_dir() {
                            paths.insert((path, true));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading home: {}", e);
                }
            }
        }
        None => {
            eprintln!("Error getting HOME");
        }
    }
    paths.insert((PathBuf::from("/bin"), false));
    paths.insert((PathBuf::from("/sbin"), false));
    paths.insert((PathBuf::from("/usr/bin"), false));
    paths.insert((PathBuf::from("/usr/sbin"), false));
    paths.insert((PathBuf::from("/usr/local/bin"), false));
    paths.insert((PathBuf::from("/usr/local/sbin"), false));
    if env::consts::OS == "macos" {
        paths.insert((PathBuf::from("/opt/homebrew/bin"), false));
    }
    paths
}

fn is_symlink(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Try to find all Python executables given a starting directory. This will recursively search all directories that are not symlinks. All exe should be returned as absolute paths.
fn find_exe_inner(
    path: &Path,
    exclude_paths: &HashSet<PathBuf>,
    recurse: bool,
) -> Vec<PathBuf> {
    if exclude_paths.contains(path) {
        return Vec::with_capacity(0);
    }
    // NOTE: not sensible for this to be a HashSet as, due to recursion, this is only a partial search
    let mut paths = Vec::new();

    if path.is_dir() {
        // if we find "fpdir/pyvenv.cfg", we can always get fpdir/bin/python3
        let path_cfg = path.to_path_buf().join("pyvenv.cfg");
        if path_cfg.exists() {
            let path_exe = path.to_path_buf().join("bin/python3");
            if path_exe.exists() && is_python_exe(&path_exe) {
                paths.push(path_exe)
            }
        } else {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        let path = entry.unwrap().path();
                        if recurse && path.is_dir() && !is_symlink(&path) {
                            // recurse
                            paths.extend(find_exe_inner(&path, exclude_paths, recurse));
                        } else if is_python_exe(&path) {
                            paths.push(path);
                        }
                    }
                }
                Err(e) => {
                    // log this?
                    eprintln!("Error reading {:?}: {}", path, e);
                }
            }
        }
    }
    paths
}

// After collecting origins, find all executables
pub(crate) fn find_exe() -> HashSet<PathBuf> {
    let exclude = get_search_exclude_paths();
    let origins = get_search_origins();

    let mut paths: HashSet<PathBuf> = origins
        .par_iter()
        .flat_map(|(path, recurse)| find_exe_inner(path, &exclude, *recurse))
        .collect();
    if let Some(exe_def) = get_absolute_path_from_exe("python3") {
        paths.insert(exe_def);
    }
    paths
}

//------------------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::os::unix::fs::symlink;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_get_search_exclude_paths_a() {
        let post = get_search_exclude_paths();
        assert_eq!(post.len() > 2, true);
    }

    #[test]
    fn test_get_search_origins_a() {
        let post = get_search_origins();
        assert_eq!(post.len() > 6, true);
    }

    #[test]
    fn test_is_exe_a() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("test.sh");
        let _ = File::create(fp.clone()).unwrap();
        let mut perms = fs::metadata(fp.clone()).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x (755) for an executable script
        fs::set_permissions(fp.clone(), perms).unwrap();
        assert_eq!(is_python_exe(&fp), false);
    }

    #[test]
    fn test_is_exe_b() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("python");
        let _ = File::create(fp.clone()).unwrap();
        let mut perms = fs::metadata(fp.clone()).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x (755) for an executable script
        fs::set_permissions(fp.clone(), perms).unwrap();
        assert_eq!(is_python_exe(&fp), true);
    }

    #[test]
    fn test_is_exe_c() {
        let temp_dir = tempdir().unwrap();
        let fp = temp_dir.path().join("python10.100");
        let _ = File::create(fp.clone()).unwrap();
        let mut perms = fs::metadata(fp.clone()).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x (755) for an executable script
        fs::set_permissions(fp.clone(), perms).unwrap();
        assert_eq!(is_python_exe(&fp), true);
    }

    #[test]
    fn test_is_symlink_a() {
        let temp_dir = tempdir().unwrap();
        let fp1 = temp_dir.path().join("test.txt");
        let _ = File::create(fp1.clone()).unwrap();
        let fp2 = temp_dir.path().join("link.txt");
        let _ = symlink(fp1.clone(), fp2.clone());
        assert_eq!(is_symlink(&fp1), false);
        assert_eq!(is_symlink(&fp2), true);
    }

    #[test]
    fn test_scan_executable_inner_a() {
        let temp_dir = tempdir().unwrap();
        let fpd1 = temp_dir.path();
        let fpf1 = fpd1.join("pyvenv.cfg");
        let _ = File::create(fpf1).unwrap();

        let fpd2 = fpd1.join("bin");
        fs::create_dir(fpd2.clone()).unwrap();

        let fpf2 = fpd2.join("python3");
        let _ = File::create(fpf2.clone()).unwrap();
        let mut perms = fs::metadata(fpf2.clone()).unwrap().permissions();
        perms.set_mode(0o755); // rwxr-xr-x (755) for an executable script
        fs::set_permissions(fpf2.clone(), perms).unwrap();

        let exclude_paths = HashSet::with_capacity(0);
        let mut result = find_exe_inner(fpd1, &exclude_paths, true);
        assert_eq!(result.len(), 1);

        let fp_found: PathBuf = result.pop().unwrap();
        let pcv = fp_found.into_iter().rev().take(2).collect::<Vec<_>>();
        let pcp = pcv.iter().rev().collect::<PathBuf>();
        assert_eq!(pcp, PathBuf::from("bin/python3"));
    }
}
