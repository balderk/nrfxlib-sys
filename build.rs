//! Build Script for nrfxlib-sys
//!
//! Calls out to bindgen to generate a Rust crate from the Nordic header
//! files.

use std::io::Write;
use ihex::Record;
use std::env;
use std::path::{Path, PathBuf};

fn serialize_firmware(firmware_hex: &str, firmware_name: &str) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join(format!("{}.rs", firmware_name));
    let mut out_file = std::fs::OpenOptions::new().append(false).write(true).create(true).open(out_path).unwrap();
    let mut addr_offset: u32 = 0;
    write!(out_file, "pub const {}: [Firmware; {}_SIZE] = [\n", firmware_name, firmware_name).unwrap();
    let mut firmware_len = 0;
    let mut buffer = Vec::new();
    let data_target_len = 1000;
    let mut addr = 0;
    for v in ihex::Reader::new(firmware_hex).into_iter() {
        if let Ok(v) = v {
            match v.clone() {
                Record::Data { offset, value } => {
                    let offset_addr = addr_offset + offset as u32;
                    if addr == 0 {
                        addr = offset_addr;
                    }
                    buffer.extend(value);
                    if buffer.len() >= data_target_len {
                        firmware_len += 1;
                        write!(out_file, "    Firmware {{ addr: {}, data: &{:?} }},\n", addr, buffer.as_slice()).unwrap();
                        addr = 0;
                        buffer.clear();
                    }
                }
                Record::EndOfFile => {}
                Record::ExtendedSegmentAddress(_) => { panic!("not implemented") }
                Record::StartSegmentAddress { .. } => { panic!("not implemented") }
                Record::ExtendedLinearAddress(offset) => {
                    addr_offset = (offset as u32) << 16;
                }
                Record::StartLinearAddress(_) => { panic!("not implemented") }
            }
        }
    }
    if buffer.len() > 0 {
        firmware_len += 1;
        write!(out_file, "    Firmware {{ addr: {}, data: &{:?} }},\n", addr, buffer.as_slice()).unwrap()
    }
    write!(out_file, "];\npub const {}_SIZE: usize = {};\n", firmware_name, firmware_len).unwrap();
}

fn main() {
    let nrfxlib_path = "./third_party/nordic/nrfxlib";
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        // Set the target
        .clang_arg("-target")
        .clang_arg("arm")
        .clang_arg("-mcpu=cortex-m33")
        // Use softfp
        .clang_arg("-mfloat-abi=hard")
        .header("wrapper.h")
        // Point to Nordic headers
        .clang_arg(format!("-I{}", nrfxlib_path))
        // Point to our special local headers
        .clang_arg("-I./third_party/newlib/include")
        // Add extra paths that the C files assume are searched
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_cc310_platform/include")
        .clang_arg("-I./third_party/nordic/nrfxlib/crypto/nrf_oberon")
        // Disable standard includes (they belong to the host)
        // .clang_arg("-nostdinc")
        // We're no_std
        .use_core()
        // Use our own ctypes to save using libc
        .ctypes_prefix("ctypes")
        // Include only the useful stuff
        .allowlist_function("nrf_.*")
        .allowlist_function("bsd_.*")
        .allowlist_type("nrf_.*")
        .allowlist_var("NRF_.*")
        .allowlist_var("BSD_.*")
        // Format the output
        .rustfmt_bindings(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let mut rust_source = bindings.to_string();

    // Munge Doxygen comments into something Rustdoc can handle
    rust_source = rust_source.replace("#[doc = \"@{*/\"]", "");
    let re = regex::Regex::new("\"   \\s+- ").unwrap();
    rust_source = re.replace_all(&rust_source, "\" * ").into();
    let re = regex::Regex::new(r"\s*@param\s+(?P<var>[A-Za-z0-9_]+)\s+").unwrap();
    rust_source = re.replace_all(&rust_source, " * `$var` - ").into();
    let re =
        regex::Regex::new(r"\s*@param\[(out|in|inout|in,out)\](\\t|\s+)(?P<var>[A-Za-z0-9_]+)\s+")
            .unwrap();
    rust_source = re.replace_all(&rust_source, " * `$var` - ").into();
    let re = regex::Regex::new(r"@[cp]\s+(?P<var>[A-Za-z0-9_\(\)]+)").unwrap();
    rust_source = re.replace_all(&rust_source, " * `$var` - ").into();
    let re = regex::Regex::new(r"\\\\[cp]\s+(?P<var>[A-Za-z0-9_\(\)]+)").unwrap();
    rust_source = re.replace_all(&rust_source, "`$var`").into();
    let re = regex::Regex::new(r"\\\\ref\s+(?P<var>[A-Za-z0-9_\(\)]+)").unwrap();
    rust_source = re.replace_all(&rust_source, "`$var`").into();
    rust_source = rust_source.replace("\" @remark", "\" NB: ");
    rust_source = rust_source.replace("\"@brief", "\"");
    rust_source = rust_source.replace("\" @brief", "\" ");
    rust_source = rust_source.replace("\"@detail", "\"");
    rust_source = rust_source.replace("\" @detail", "\" ");
    rust_source = rust_source.replace("@name ", "# ");
    rust_source = rust_source.replace("@return ", "Returns ");
    rust_source = rust_source.replace("@retval ", "Returns ");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    std::fs::write(out_path, rust_source).expect("Couldn't write updated bindgen output");

    // Make sure we link against the libraries
    println!(
        "cargo:rustc-link-search={}",
        Path::new(&nrfxlib_path)
            .join("nrf_modem/lib/cortex-m33/hard-float")
            .display()
    );
    println!(
        "cargo:rustc-link-search={}",
        Path::new(&nrfxlib_path)
            .join("crypto/nrf_oberon/lib/cortex-m33/hard-float")
            .display()
    );
    println!("cargo:rustc-link-lib=static=modem");
    println!("cargo:rustc-link-lib=static=oberon_3.0.11");

    serialize_firmware(include_str!("third_party/firmware/firmware.update.image.segments.0.hex"), "FIRMWARE0");
    serialize_firmware(include_str!("third_party/firmware/firmware.update.image.segments.1.hex"), "FIRMWARE1");
}
