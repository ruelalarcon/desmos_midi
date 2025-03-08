fn main() {
    #[cfg(windows)]
    {
        // Only run if we're on Windows
        println!("cargo:rerun-if-changed=build/windows/icon.rc");
        println!("cargo:rerun-if-changed=assets/icon.ico");

        // Create a Windows resource for the main binary
        {
            let mut res = winres::WindowsResource::new();
            res.set_resource_file("build/windows/icon.rc");

            if let Err(e) = res.compile() {
                eprintln!("Error compiling resources: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Packaging-related instructions (these don't affect the build process)
    println!("cargo:rerun-if-changed=config.toml");
    println!("cargo:rerun-if-changed=soundfonts");
}