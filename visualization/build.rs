use std::io;
use std::process::Command;
use std::time::SystemTime;
use static_files::resource_dir;

fn main() -> io::Result<()> {
    build_frontend()
}

fn build_frontend() -> io::Result<()> {
    let time = SystemTime::now();
    
    // Build the frontend as static files
    Command::new("pnpm")
        .current_dir("./ui")
        .arg("build")
        .status()?;
    
    println!("cargo:warning=Frontend build took {:?}", time.elapsed().unwrap());
    
    resource_dir("./ui/build/client").build()?;

    // Tell Cargo that only if anything important in the UI folder changes, to rerun this build script.
    println!("cargo::rerun-if-changed=ui/app");
    println!("cargo::rerun-if-changed=ui/assets");
    println!("cargo::rerun-if-changed=ui/pnpm-lock.yaml");
    println!("cargo::rerun-if-changed=ui/package.json");
    println!("cargo::rerun-if-changed=ui/react-router.config.ts");
    println!("cargo::rerun-if-changed=ui/tailwind.config.ts");
    println!("cargo::rerun-if-changed=ui/tsconfig.json");
    println!("cargo::rerun-if-changed=ui/vite.config.ts");

    // The frontend files themselves are included in the source code using include!
    Ok(())
}
