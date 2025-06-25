// build.rs
fn main() {
    // 指定 WinPcap/Npcap 的库路径
    println!("cargo:rustc-link-search=C:/Program Files/Npcap/Lib/x64");
    // 链接静态库
    println!("cargo:rustc-link-lib=wpcap");
    println!("cargo:rustc-link-lib=Packet"); // Packet.lib
                                             // 可能需要链接 Windows 系统库
    println!("cargo:rustc-link-lib=Advapi32");
}
