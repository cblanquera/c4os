fn main() {
    println!("cargo:rerun-if-changed=../frontend/index.html");
    println!("cargo:rerun-if-changed=../frontend/app.js");
    println!("cargo:rerun-if-changed=../frontend/styles.css");
    println!("cargo:rerun-if-changed=../frontend/connectors.js");
    println!("cargo:rerun-if-changed=../frontend/connector-contract.js");
    println!("cargo:rerun-if-changed=../frontend/vendor/xterm/xterm.js");
    println!("cargo:rerun-if-changed=../frontend/vendor/xterm/xterm.css");
    tauri_build::build();
}
