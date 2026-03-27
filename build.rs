#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "PostureWatch");
    res.set("ProductName", "PostureWatch");
    res.set("CompanyName", "PostureWatch Contributors");
    res.set("LegalCopyright", "Copyright (c) PostureWatch Contributors");
    res.set("OriginalFilename", "PostureWatch.exe");

    if let Err(err) = res.compile() {
        panic!("failed to compile Windows resources: {err}");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
