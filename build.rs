use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", "cspice/cspice/lib");
    
    // Tell cargo to tell rustc to link the system cspice library.
    // This is commented out for now since we're using WASM approach
    // println!("cargo:rustc-link-lib=cspice");

    // Tell cargo to invalidate the built crate whenever the C header changes
    println!("cargo:rerun-if-changed=cspice/cspice/include/SpiceUsr.h");

    // Check if we have CSPICE headers available and bindgen feature is enabled
    let cspice_header_path = PathBuf::from("cspice/cspice/include/SpiceUsr.h");
    
    #[cfg(feature = "bindgen")]
    {
        if cspice_header_path.exists() {
            // Try to generate bindings from CSPICE headers
            match try_generate_bindings() {
                Ok(_) => {
                    println!("cargo:rustc-cfg=feature=\"cspice_bindings_available\"");
                    println!("cargo:warning=Successfully generated CSPICE bindings");
                }
                Err(e) => {
                    println!("cargo:warning=Could not generate CSPICE bindings: {}", e);
                    println!("cargo:warning=Building without CSPICE bindings - install libclang if needed");
                }
            }
        } else {
            println!("cargo:warning=CSPICE headers not found at {:?}", cspice_header_path);
            println!("cargo:warning=Enable bindgen feature and run from project root with CSPICE source available");
        }
    }
    
    #[cfg(not(feature = "bindgen"))]
    {
        println!("cargo:warning=Building without automatic bindgen (use --features bindgen to enable)");
        println!("cargo:warning=Using manual CSPICE interface definitions");
    }
}

#[cfg(feature = "bindgen")]
fn try_generate_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // Generate bindings from CSPICE headers
    let bindings = bindgen::Builder::default()
        // Main CSPICE header
        .header("cspice/cspice/include/SpiceUsr.h")
        
        // Include paths
        .clang_arg("-Icspice/cspice/include")
        
        // Generate bindings for key CSPICE functions
        .allowlist_function("spkezr_c")
        .allowlist_function("spkezp_c")
        .allowlist_function("spkpos_c")
        .allowlist_function("furnsh_c")
        .allowlist_function("unload_c")
        .allowlist_function("kclear_c")
        .allowlist_function("failed_c")
        .allowlist_function("getmsg_c")
        .allowlist_function("reset_c")
        .allowlist_function("sigerr_c")
        .allowlist_function("setmsg_c")
        .allowlist_function("str2et_c")
        .allowlist_function("et2utc_c")
        .allowlist_function("utc2et_c")
        .allowlist_function("pxform_c")
        .allowlist_function("sxform_c")
        .allowlist_function("namfrm_c")
        .allowlist_function("frmnam_c")
        .allowlist_function("chkin_c")
        .allowlist_function("chkout_c")
        .allowlist_function("erract_c")
        .allowlist_function("errdev_c")
        .allowlist_function("exists_c")
        
        // Generate bindings for key CSPICE types
        .allowlist_type("SpiceDouble")
        .allowlist_type("SpiceInt")
        .allowlist_type("SpiceChar")
        .allowlist_type("SpiceBoolean")
        .allowlist_type("ConstSpiceDouble")
        .allowlist_type("ConstSpiceInt")
        .allowlist_type("ConstSpiceChar")
        .allowlist_type("ConstSpiceBoolean")
        
        // Generate bindings for constants
        .allowlist_var("SPICETRUE")
        .allowlist_var("SPICEFALSE")
        .allowlist_var("SPICESUCCESS")
        .allowlist_var("SPICEFAILURE")
        
        // Customize the generated code
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        
        // Generate the bindings
        .generate()?;

    // Write the bindings to the $OUT_DIR/cspice_bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("cspice_bindings.rs"))?;
    
    Ok(())
}
