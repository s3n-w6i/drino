use std::io;
use std::process::Command;
use static_files::resource_dir;

fn main() -> io::Result<()> {
    build_frontend()
}

fn build_frontend() -> io::Result<()> {
    // Build the frontend as static files
    Command::new("pnpm")
        .current_dir("./ui")
        .arg("build")
        .status()?;
    
    resource_dir("./ui/build/client").build()?;

    // The frontend files themselves are included in the source code using include!
    Ok(())
}