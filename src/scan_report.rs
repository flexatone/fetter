use std::collections::HashMap;

use crate::package::Package;
use crate::path_shared::PathShared;
use crate::table::ColumnFormat;
use crate::table::Rowable;
use crate::table::RowableContext;
use crate::table::Tableable;

#[derive(Debug, Clone)]
pub(crate) struct ScanRecord {
    package: Package,
    sites: Vec<PathShared>,
}

impl ScanRecord {
    pub(crate) fn new(package: Package, sites: Vec<PathShared>) -> Self {
        ScanRecord { package, sites }
    }
}

impl Rowable for ScanRecord {
    fn to_rows(&self, context: &RowableContext) -> Vec<Vec<String>> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let pkg_display = self.package.to_string();
        let is_tty = *context == RowableContext::Tty;

        for (i, path) in self.sites.iter().enumerate() {
            let p = if i > 0 && is_tty {
                "".to_string()
            } else {
                pkg_display.clone()
            };
            rows.push(vec![p, path.to_string()]);
        }
        rows
    }
}

#[derive(Debug)]
pub struct ScanReport {
    records: Vec<ScanRecord>,
}

impl ScanReport {
    pub(crate) fn from_package_to_sites(
        package_to_sites: &HashMap<Package, Vec<PathShared>>,
    ) -> Self {
        let mut records = Vec::new();
        for (package, sites) in package_to_sites {
            let record = ScanRecord::new(package.clone(), sites.clone());
            records.push(record);
        }
        records.sort_by_key(|item| item.package.clone());
        ScanReport { records }
    }

    // Alternative constructor when we want to report on a subset of all packages.
    pub(crate) fn from_packages(
        packages: &Vec<Package>,
        package_to_sites: &HashMap<Package, Vec<PathShared>>,
    ) -> Self {
        let mut records = Vec::new();
        for package in packages {
            let sites = package_to_sites.get(package).unwrap();
            let record = ScanRecord::new(package.clone(), sites.clone());
            records.push(record);
        }
        records.sort_by_key(|item| item.package.clone());
        ScanReport { records }
    }
}

impl Tableable<ScanRecord> for ScanReport {
    fn get_header(&self) -> Vec<ColumnFormat> {
        vec![
            ColumnFormat::new("Package".to_string(), false, "#666666".to_string()),
            ColumnFormat::new("Site".to_string(), true, "#666666".to_string()),
        ]
    }
    fn get_records(&self) -> &Vec<ScanRecord> {
        &self.records
    }
}

//------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan_fs::ScanFS;
    use std::fs::File;
    use std::io;
    use std::io::BufRead;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_to_file_a() {
        let exe = PathBuf::from("/usr/bin/python3");
        let site = PathBuf::from("/usr/lib/python3/site-packages");
        let packages = vec![
            Package::from_name_version_durl("numpy", "1.19.3", None).unwrap(),
            Package::from_name_version_durl("static-frame", "2.13.0", None).unwrap(),
            Package::from_name_version_durl("flask", "1.2", None).unwrap(),
            Package::from_name_version_durl("packaging", "24.1", None).unwrap(),
        ];
        let sfs = ScanFS::from_exe_site_packages(exe, site, packages).unwrap();

        let sr1 = sfs.to_scan_report();

        let dir = tempdir().unwrap();
        let fp = dir.path().join("scan.txt");
        let _ = sr1.to_file(&fp, '|');

        let file = File::open(&fp).unwrap();
        let mut lines = io::BufReader::new(file).lines();

        assert_eq!(lines.next().unwrap().unwrap(), "Package|Site");
        assert_eq!(
            lines.next().unwrap().unwrap(),
            "flask-1.2|/usr/lib/python3/site-packages"
        );
        assert_eq!(
            lines.next().unwrap().unwrap(),
            "numpy-1.19.3|/usr/lib/python3/site-packages"
        );
        assert_eq!(
            lines.next().unwrap().unwrap(),
            "packaging-24.1|/usr/lib/python3/site-packages"
        );
        assert_eq!(
            lines.next().unwrap().unwrap(),
            "static-frame-2.13.0|/usr/lib/python3/site-packages"
        );
        assert!(lines.next().is_none());
    }
}
