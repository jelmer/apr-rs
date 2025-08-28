extern crate bindgen;

fn create_bindings(
    apr_path: &std::path::Path,
    apu_path: &std::path::Path,
    out_path: &std::path::Path,
    apr_include_paths: &[&std::path::Path],
) {
    // Generate bindings using bindgen
    let mut builder = bindgen::Builder::default();
    // check if the pool-debug feature is present
    if std::env::var("CARGO_FEATURE_POOL_DEBUG").is_ok() {
        builder = builder.clang_arg("-DAPR_POOL_DEBUG");
    }
    let bindings = builder
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
        .header(apr_path.join("apr_thread_mutex.h").to_str().unwrap())
        .header(apr_path.join("apr_thread_cond.h").to_str().unwrap())
        .header(apr_path.join("apr_dso.h").to_str().unwrap())
        .header(apr_path.join("apr_env.h").to_str().unwrap())
        .header(apr_path.join("apr_network_io.h").to_str().unwrap())
        .header(apr_path.join("apr_mmap.h").to_str().unwrap())
        .header(apr_path.join("apr_user.h").to_str().unwrap())
        .header(apu_path.join("apr_md5.h").to_str().unwrap())
        .header(apu_path.join("apr_sha1.h").to_str().unwrap())
        .header(apu_path.join("apr_base64.h").to_str().unwrap())
        .header(apu_path.join("apr_uuid.h").to_str().unwrap())
        .header(apu_path.join("apr_strmatch.h").to_str().unwrap())
        .header(apu_path.join("apr_xlate.h").to_str().unwrap())
        .header(apu_path.join("apr_xml.h").to_str().unwrap())
        .header(apu_path.join("apr_crypto.h").to_str().unwrap())
        .header_contents("sys_socket.h", "#include <sys/socket.h>")
        .header_contents("sys_types.h", "#include <sys/types.h>")
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
        .allowlist_file(".*/apr_thread_mutex.h")
        .allowlist_file(".*/apr_thread_cond.h")
        .allowlist_file(".*/apr_dso.h")
        .allowlist_file(".*/apr_env.h")
        .allowlist_file(".*/apr_network_io.h")
        .allowlist_file(".*/apr_mmap.h")
        .allowlist_file(".*/apr_user.h")
        .allowlist_file(".*/apr_md5.h")
        .allowlist_file(".*/apr_sha1.h")
        .allowlist_file(".*/apr_base64.h")
        .allowlist_file(".*/apr_uuid.h")
        .allowlist_file(".*/apr_strmatch.h")
        .allowlist_file(".*/apr_xlate.h")
        .allowlist_file(".*/apr_xml.h")
        .allowlist_file(".*/apr_crypto.h")
        .allowlist_file(".*/apr_portable.h")
        .allowlist_file(".*/apr_support.h")
        .clang_args(
            apr_include_paths
                .iter()
                .map(|path| format!("-I{}", path.display())),
        )
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
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
