use std::{env::var_os, path::Path, process::Command};

fn main() {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    build_molten_vk();
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn build_molten_vk() {
    match var_os("SIERRA_BUILD_MVK") {
        None => return,
        Some(value) => {
            if value != "yes" && value != "1" {
                return;
            }
        }
    }

    let target_dir = match var_os("CARGO_TARGET_DIR") {
        None => {
            let root = var_os("CARGO_MANIFEST_DIR").expect("Failed to find CARGO_MANIFEST_DIR");
            Path::new(&root).join("target")
        }
        Some(target_dir) => target_dir.into(),
    };

    let profile = var_os("PROFILE").expect("Failed to find PROFILE");
    let target_tripple = var_os("TARGET").expect("Failed to find TARGET");
    let host_tripple = var_os("HOST").expect("Failed to find HOST");

    let lib_dir = if host_tripple == target_tripple {
        target_dir.join(profile)
    } else {
        target_dir.join(target_tripple).join(profile)
    };

    let lib_path = lib_dir.join("libvulkan.dylib");

    if lib_path.exists() {
        return;
    }

    let out_dir = var_os("OUT_DIR").expect("Failed to find OUT_DIR");
    let mvk_checkout_dir = Path::new(&out_dir).join("mvk");

    let status = Command::new("git")
        .arg("clone")
        .args(["--depth", "1"])
        .arg("https://github.com/KhronosGroup/MoltenVK.git")
        .arg(&mvk_checkout_dir)
        .spawn()
        .expect("Failed to run git")
        .wait()
        .expect("Failed to clone MoltenVK repo");

    assert!(status.success(), "Failed to clone MoltenVK repo");

    let (target_name, dylib_dir) = match std::env::var("CARGO_CFG_TARGET_OS") {
        Ok(target) => match target.as_ref() {
            "macos" => ("macos", "macOS"),
            "ios" => ("ios", "iOS"),
            target => panic!("Unknown target '{}'", target),
        },
        Err(e) => panic!("Failed to determinte target os '{}'", e),
    };

    let status = Command::new("sh")
        .current_dir(&mvk_checkout_dir)
        .arg("fetchDependencies")
        .arg(format!("--{}", target_name))
        .spawn()
        .expect("Failed to run fetchDependencies script")
        .wait()
        .expect("Failed to fetch dependencies");

    assert!(status.success(), "Failed to fetch dependencies");

    let status = Command::new("make")
        .current_dir(&mvk_checkout_dir)
        .arg(target_name)
        .spawn()
        .expect("Failed to build MoltenVK")
        .wait()
        .expect("Failed to build MoltenVK");

    assert!(status.success(), "Failed to build MoltenVK");

    let dylib_path = mvk_checkout_dir
        .join("MoltenVK")
        .join("dylib")
        .join(dylib_dir)
        .join("libMoltenVK.dylib");

    std::fs::copy(&dylib_path, &lib_path).expect("Failed to copy MoltenVK dylib");
}
