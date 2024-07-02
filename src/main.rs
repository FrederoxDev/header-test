extern crate clang;
use clang::{Clang, Entity, EntityKind, Index};

fn main() {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let header_path = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/src/minecraft/src/common/world/level/block/BlockLegacy.hpp";

    // msvc stuff
    let msvc_include_path = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.40.33807/include";
    let windows_sdk_include_path = "C:/Program Files (x86)/Windows Kits/10/Include/10.0.22621.0";
    let amethyst_include_path = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/src";
    let amethyst_additional_include = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/include";

    let tu = index
        .parser(header_path)
        .arguments(&[
            "-fms-extensions",
            "-fms-compatibility",
            &format!("-I{}", msvc_include_path),
            &format!("-I{}", windows_sdk_include_path),
            &format!("-I{}", amethyst_include_path),
            &format!("-I{}", amethyst_additional_include),
        ])
        .parse()
        .expect("Failed to parse the header file");

    for diag in tu.get_diagnostics() {
        println!("Diagnostic: {:?}", diag);
    }

    for entity in tu.get_entity().get_children() {
        if entity.get_kind() != EntityKind::ClassDecl { continue; }

        let name = entity.get_name().unwrap();
        if name != "BlockLegacy" { continue; }

        for child in entity.get_children() {
            if child.get_kind() != EntityKind::Method { continue; }

            let method_name = child.get_name().unwrap();

            if let Some(comment) = child.get_comment() {
                println!("{}\n\t '{}'", method_name, comment);
            }

            if let Some(mangled_name) = child.get_mangled_name() {
                println!("{}", mangled_name)
            }
        }
    }
}
