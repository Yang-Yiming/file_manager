#[cfg(windows)]
extern crate winres;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        
        // 只有当图标文件存在时才设置图标
        if std::path::Path::new("res/icon.ico").exists() {
            res.set_icon("res/icon.ico");
        }
        
        // 只有当清单文件存在时才设置清单
        if std::path::Path::new("res/app.manifest").exists() {
            res.set_manifest_file("res/app.manifest");
        }
        
        res.set("FileDescription", "文件快速访问管理器");
        res.set("ProductName", "文件快速访问器");
        res.set("CompanyName", "File Manager Team");
        res.set("LegalCopyright", "Copyright (C) 2024");
        res.set("FileVersion", "1.0.0.0");
        res.set("ProductVersion", "1.0.0.0");
        
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}