#[cfg(target_os = "windows")]
use winres;

#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
    match res.compile() {
        Err(error) => {
            println!("{}", error);
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}