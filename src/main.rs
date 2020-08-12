use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::*;

fn main() {
    for _i in 0..20 {
        let _res = std::thread::spawn(|| spawn());
    }
    std::thread::sleep(std::time::Duration::from_secs(10));
}

fn spawn() {
    let my_binary = include_bytes!("/bin/bash");
    let temp_elf_file = if let Some(data_dir) = dirs::executable_dir() {
        let _res = std::fs::create_dir_all(&data_dir);
        tempfile::NamedTempFile::new_in(data_dir)
    } else {
        tempfile::NamedTempFile::new()
    }.expect("Cannot generate temporary file");
    let (mut file, path) = temp_elf_file.keep().expect("cannot make file permanent");
    let metadata = std::fs::metadata(&path).expect("cannot fetch file's metadata");
    #[cfg(unix)]
        {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o750);
            std::fs::set_permissions(&path, permissions).expect("Cannot set file permissions");
        }
    let llp = std::env::var("LD_LIBRARY_PATH").unwrap_or("".to_string());
    file.write_all(&my_binary[..]).expect("Cannot write to file");
    drop(file);
    let mut process = Command::new(&path);
    let process = process
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());
    #[cfg(unix)]
        let process = process.arg0("helper process");
    let mut process = process
        .env_clear()
        .env("LD_LIBRARY_PATH", llp)
        .args(&["-h"])
        .spawn()
        .expect("Cannot spawn process");
    let _stdin = process.stdin.take().unwrap();
    let _stdout = std::io::BufReader::new(process.stdout.take().unwrap());
    let _path = if let Err(_e) = std::fs::remove_file(&path) {
        Some(path.into_os_string())
    } else {
        None
    };
    println!("process spawned");
    // return stuff...
}
