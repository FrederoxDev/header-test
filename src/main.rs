extern crate clang;
use clang::{Clang, EntityKind, Index};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SIGNATURE_REGEX: Regex = Regex::new(r"@signature\s*\{([^}]*)\}").unwrap();
}

fn main() {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let header_path = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/src/minecraft/src/common/world/level/block/registry/BlockTypeRegistry.hpp";

    // msvc stuff
    let msvc_include_path = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.40.33807/include";
    let windows_sdk_include_path = "C:/Program Files (x86)/Windows Kits/10/Include/10.0.22621.0";
    let amethyst_include_path = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/src";
    let amethyst_additional_include = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/include";

    let tu = index
        .parser(header_path)
        .arguments(&[
            "-std=c++23",
            "-fms-extensions",
            "-fms-compatibility",
            &format!("-I{}", msvc_include_path),
            &format!("-I{}", windows_sdk_include_path),
            &format!("-I{}", amethyst_include_path),
            &format!("-I{}", amethyst_additional_include),
        ])
        .parse()
        .expect("Failed to parse the header file");

    // for diag in tu.get_diagnostics() {
    //     println!("Diagnostic: {:?}", diag);
    // }

    for entity in tu.get_entity().get_children() {
        if entity.get_kind() != EntityKind::ClassDecl { continue; }

        let name = entity.get_name().unwrap();
        if name != "BlockTypeRegistry" { continue; }

        for child in entity.get_children() {
            if child.get_kind() != EntityKind::Method { continue; }

            let comment_opt = child.get_comment();
            if comment_opt.is_none() {continue;}
            let comment = comment_opt.unwrap();

            let signature_opt = try_get_signature_comment(&comment);
            if signature_opt.is_none() {continue;}
            let signature = signature_opt.unwrap();

            let mangled_name = child.get_mangled_name().unwrap();

            println!("{}:\n\t'{}'", mangled_name, signature)
        }
    }
}

fn try_get_signature_comment(comment: &str) -> Option<String> {
    let captures = SIGNATURE_REGEX.captures(comment);
    if captures.is_none() { return None; }

    if let Some(signature) = captures.unwrap().get(1) {
        return Some(signature.as_str().to_string());
    }

    return None;
}