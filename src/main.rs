extern crate clang;
use std::{env, fs::{self, OpenOptions}, io::Write};
use clang::{Clang, Entity, EntityKind, Index};
use parser::ParserArgs;
use regex::Regex;

mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return print_usage(&args);
    }

    let header_path = &args[1];
    match fs::metadata(header_path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                println!("'{}' is not a file!", header_path);
            }
        },
        Err(e) => {
            println!("error: '{}'\n", e);
            print_usage(&args);
        }
    }

    let path = std::path::Path::new(header_path);
    let parent_path = path.parent().unwrap();

    let file_stem = path.file_stem().unwrap();
    let output_path = parent_path.join(format!("{}.asm", file_stem.to_str().unwrap()));

    let parser_args = ParserArgs {
        msvc_include_path: String::from("C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.40.33807/include"),
        windows_sdk_include_path: String::from("C:/Program Files (x86)/Windows Kits/10/Include/10.0.22621.0"),
        amethyst_src_path: String::from("C:/Users/freddie/Documents/Amethyst/AmethystAPI/src"),
        amethyst_additional_include: String::from("C:/Users/freddie/Documents/Amethyst/AmethystAPI/include"),
        header_path: String::from(header_path),
        asm_out_path: String::from(output_path.to_str().unwrap())
    };

    parse(&parser_args);
}

fn print_usage(args: &Vec<String>) {
    println!("Usage:\n\t{} <header_path>", args[0]);
}

fn parse(args: &ParserArgs) {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let tu = parser::parse_header(&index, args);
    let mut generated_asm: Vec<AssemblyText> = Vec::new();

    for entity in tu.get_entity().get_children() {
        match parser::get_entity_path(&entity) {
            Some(v) => {
                if v != std::path::Path::new(&args.header_path) { 
                    continue; 
                }
            },
            None => continue
        };

        match entity.get_kind() {
            EntityKind::ClassDecl => {
                generated_asm.extend(traverse_class(&entity));
            },
            _ => {}
        }
    }

    let mut header = String::from("; automatically generated by FrederoxDev/Header-Test");
    header += "\nsection .text\n\n";

    let mut body = String::from("");

    for entry in generated_asm {
        header += &format!("{}\n", entry.header);
        body += &format!("{}\n", entry.body);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&args.asm_out_path)
        .unwrap();

    let content = format!("{}\n{}", header, body);
    file.write_all(content.as_bytes()).unwrap();
}

fn traverse_class(entity: &Entity) -> Vec<AssemblyText> {
    let mut generated_asm: Vec<AssemblyText> = Vec::new();

    match entity.get_comment() {
        Some(v) => {
            if v.contains("@vtable") {
                generated_asm.push(generate_vtable(entity));
            }
        },
        _ => {}
    }

    return generated_asm;
}

struct AssemblyText {
    pub header: String,
    pub body: String
}

fn generate_vtable(entity: &Entity) -> AssemblyText {
    let name = entity.get_name().unwrap();
    let vtable_name = format!("{}_vtable", name);
    let header = format!("extern {}", vtable_name);

    let mut body = String::from("");

    for child in entity.get_children() {
        let directive = match parser::get_variable_directive(&child, "vIndex") {
            Some(v) => v,
            None => continue
        };

        let v_index: Result<u32, _> = directive.parse();
        let mangled_name = child.get_mangled_name().unwrap();

        match v_index {
            Ok(index) => {
                body += &format!(
                    "global {}\n{}:\n\tmov rax, [rel {}]\n\tjmp [rax + {}]\n\n",
                    mangled_name, mangled_name, vtable_name, index * 8
                );
            },
            _ => continue
        }
    }

    return AssemblyText {
        header,
        body
    };
}