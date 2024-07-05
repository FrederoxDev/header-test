use clang::{Entity, Index, TranslationUnit};
use regex::Regex;

pub struct ParserArgs {
    pub msvc_include_path: String,
    pub windows_sdk_include_path: String,
    pub amethyst_src_path: String,
    pub amethyst_additional_include: String,
    pub header_path: String,
    pub asm_out_path: String
}

pub fn parse_header<'a>(index: &'a Index, args: &ParserArgs) -> TranslationUnit<'a> {
    return index
        .parser(&args.header_path)
        .arguments(&[
            "-std=c++23",
            "-fms-extensions",
            "-fms-compatibility",
            &format!("-I{}", args.msvc_include_path),
            &format!("-I{}", args.windows_sdk_include_path),
            &format!("-I{}", args.amethyst_src_path),
            &format!("-I{}", args.amethyst_additional_include),
        ])
        .parse()
        .expect("Failed to parse the header file");
}

pub fn get_variable_directive(entity: &Entity, directive: &str) -> Option<String> {
    let comment = &entity.get_comment()?;
    let regex = Regex::new(&format!(r"@{}\s*\{{([^}}]*)\}}", directive)).unwrap();
    let captures = regex.captures(comment)?;
    let capture = captures.get(1)?;
    return Some(capture.as_str().to_string());
}

pub fn get_entity_path(entity: &Entity) -> Option<std::path::PathBuf> {
    let location = entity.get_location()?;
    let spelling_location = location.get_spelling_location().file?;
    return Some(spelling_location.get_path());
}