fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        // wix/icon.ico is a copy of assets/icons/icon.ico kept inside the crate
        // root so it ships in the published package (`cargo install` on Windows
        // needs it too, not just repo builds).
        res.set_icon("wix/icon.ico");
        res.set("FileDescription", "Spectatui - GitHub Spec-Kit TUI dashboard");
        res.set("ProductName", "Spectatui");
        res.set("LegalCopyright", "Copyright © 2026 Tine Kondo");
        res.compile().expect("Failed to compile Windows resources");
    }
}
