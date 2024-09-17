extern crate bindgen;

fn create_bindings(
    apr_path: &std::path::Path,
    apu_path: &std::path::Path,
    out_path: &std::path::Path,
    apr_include_paths: &[&std::path::Path],
) {
    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(apr_path.join("apr.h").to_str().unwrap())
        .header(apr_path.join("apr_allocator.h").to_str().unwrap())
        .header(apr_path.join("apr_general.h").to_str().unwrap())
        .header(apr_path.join("apr_errno.h").to_str().unwrap())
        .header(apr_path.join("apr_pools.h").to_str().unwrap())
        .header(apr_path.join("apr_version.h").to_str().unwrap())
        .header(apr_path.join("apr_tables.h").to_str().unwrap())
        .header(apr_path.join("apr_hash.h").to_str().unwrap())
        .header(apr_path.join("apr_file_info.h").to_str().unwrap())
        .header(apr_path.join("apr_file_io.h").to_str().unwrap())
        .header(apr_path.join("apr_getopt.h").to_str().unwrap())
        .header(apu_path.join("apr_uri.h").to_str().unwrap())
        .header(apr_path.join("apr_time.h").to_str().unwrap())
        .header(apu_path.join("apr_date.h").to_str().unwrap())
        .header(apr_path.join("apr_version.h").to_str().unwrap())
        .header(apu_path.join("apu_version.h").to_str().unwrap())
        .header(apr_path.join("apr_strings.h").to_str().unwrap())
        .header(apr_path.join("apr_thread_proc.h").to_str().unwrap())
        .allowlist_file(".*/apr.h")
        .allowlist_file(".*/apr_general.h")
        .allowlist_file(".*/apr_allocator.h")
        .allowlist_file(".*/apr_version.h")
        .allowlist_file(".*/apr_errno.h")
        .allowlist_file(".*/apr_pools.h")
        .allowlist_file(".*/apr_tables.h")
        .allowlist_file(".*/apr_hash.h")
        .allowlist_file(".*/apr_file_info.h")
        .allowlist_file(".*/apr_file_io.h")
        .allowlist_file(".*/apr_getopt.h")
        .allowlist_file(".*/apr_uri.h")
        .allowlist_file(".*/apr_time.h")
        .allowlist_file(".*/apr_date.h")
        .allowlist_file(".*/apr_strings.h")
        .allowlist_file(".*/apr_version.h")
        .allowlist_file(".*/apu_version.h")
        .allowlist_file(".*/apr_thread_proc.h")
        .clang_args(
            apr_include_paths
                .iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_path.join("generated.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    let deps = system_deps::Config::new().probe().unwrap();

    let apr = deps.get_by_name("apr-1").unwrap();

    let apr_util = deps.get_by_name("apr-util-1").unwrap();

    let apr_path = apr
        .include_paths
        .iter()
        .find(|x| x.join("apr.h").exists())
        .expect("Failed to find apr.h");

    let apr_util_path = apr_util
        .include_paths
        .iter()
        .find(|x| x.join("apu.h").exists())
        .expect("Failed to find apu.h");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    create_bindings(
        apr_path.as_path(),
        apr_util_path.as_path(),
        out_path.as_path(),
        apr.include_paths
            .iter()
            .map(|x| x.as_path())
            .collect::<Vec<_>>()
            .as_slice(),
    );
}
