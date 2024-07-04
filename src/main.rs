extern crate clang;
use clang::{Clang, Entity, EntityKind, Index};
use regex::Regex;
use lazy_static::lazy_static;
use std::{path::PathBuf};

lazy_static! {
    static ref SIGNATURE_REGEX: Regex = Regex::new(r"@signature\s*\{([^}]*)\}").unwrap();
}

fn main() {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let header_path = "C:/Users/freddie/Documents/Amethyst/AmethystAPI/src/minecraft/src/common/world/level/dimension/Dimension.hpp";
    let header_path_buf = std::path::Path::new(header_path);

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

    for entity in tu.get_entity().get_children() {
        match get_entity_path(&entity) {
            Some(v) => {
                if v != header_path_buf { 
                    continue; 
                }
            },
            None => continue
        };

        match entity.get_kind() {
            EntityKind::ClassDecl => {
                traverse_class(&entity)
            },
            _ => {}
        }
    }
}

fn traverse_class(entity: &Entity) {
    match entity.get_comment() {
        Some(v) => {
            if v.contains("@vtable") {
                generate_vtable(entity);
            }
        },
        _ => {}
    }
}

fn generate_vtable(entity: &Entity) {
    let name = entity.get_name().unwrap();
    let vtable_name = format!("{}_vtable", name);
    let mut vtable = format!("extern {}\n\n", vtable_name);

    for child in entity.get_children() {
        let directive = match get_variable_directive(&child, "vIndex") {
            Some(v) => v,
            None => continue
        };

        let v_index: Result<u32, _> = directive.parse();
        let mangled_name = child.get_mangled_name().unwrap();

        match v_index {
            Ok(index) => {
                vtable += &format!(
                    "global {}\n{}:\n\tmov rax, [rel {}]\n\tjmp [rax + {}]\n\n",
                    mangled_name, mangled_name, vtable_name, index * 8
                );
            },
            _ => continue
        }
    }

    println!("{}", vtable)
}

fn get_variable_directive(entity: &Entity, directive: &str) -> Option<String> {
    let comment = &entity.get_comment()?;
    let regex = Regex::new(&format!(r"@{}\s*\{{([^}}]*)\}}", directive)).unwrap();
    let captures = regex.captures(comment)?;
    let capture = captures.get(1)?;
    return Some(capture.as_str().to_string());
}

fn get_entity_path(entity: &Entity) -> Option<PathBuf> {
    let location = entity.get_location()?;
    let spelling_location = location.get_spelling_location().file?;
    return Some(spelling_location.get_path());
}