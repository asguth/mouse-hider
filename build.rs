//! Embute o icone e as informacoes de versao no MouseHider.exe.

fn main() {
    println!("cargo:rerun-if-changed=assets/MouseHider.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut recurso = winresource::WindowsResource::new();
    recurso
        .set_icon("assets/MouseHider.ico")
        .set("ProductName", "Mouse Hider")
        .set("FileDescription", "Mouse Hider")
        .set("CompanyName", "asguth")
        .set("LegalCopyright", "MIT");

    // Compilar do Linux para checar tipos nao tem rc.exe/windres: vira aviso, nao erro.
    if let Err(erro) = recurso.compile() {
        println!("cargo:warning=icone nao embutido no exe: {erro}");
    }
}
