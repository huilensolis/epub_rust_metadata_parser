use core::panic;
use std::{
    env, fs,
    io::{self, BufRead},
    path::Path,
    process::Command,
};

fn main() {
    let args: Vec<_> = env::args().collect();

    // there is a default first argument, therefore the first argument passed to the cli, is the
    // number 2
    if args.len() > 2 || args.len() == 1 {
        panic!("only one argument is required: a path to a .epub file")
    }

    let str_path_to_epub_file = &args[1];

    let path_to_epub_file = Path::new(str_path_to_epub_file);

    let destiny_folder = match path_to_epub_file.file_stem() {
        Some(path) => path.to_str().unwrap(),
        None => panic!("could not extract file name from path to epub file"),
    };

    let current_dir = env::current_dir().unwrap();

    let ebook_dir_path = current_dir.join(destiny_folder);

    fs::create_dir(&ebook_dir_path).expect("failed to create destiny folder");

    Command::new("unzip")
        .args(["-q", path_to_epub_file.to_str().unwrap()])
        .args(["-d", destiny_folder])
        .status()
        .expect("failed to unzip epub file");

    let container_file = fs::File::open(ebook_dir_path.join("META-INF/container.xml"))
        .expect("could not read contents.xml file");

    let container_reader = io::BufReader::new(container_file);

    let mut contents_file_path: String = String::new();

    for raw_line in container_reader.lines() {
        let line = raw_line.unwrap();
        if line.trim().starts_with("<rootfile") && line.trim().ends_with("/>") {
            let mut splited_line = line.trim().split(' ');

            let contents_tag = splited_line
                .find(|text| text.starts_with("full-path="))
                .expect("could not find full-path proprety in container.xml file");

            let contents_path = contents_tag
                .split('=')
                .last()
                .unwrap()
                .replace(['\\', '\"'], "");

            contents_file_path.push_str(contents_path.as_str());
        }
    }

    if contents_file_path.is_empty() {
        panic!("could not parse container.xml file lines correctly to extract contents path")
    }

    let absolute_path_to_contents_file = ebook_dir_path.join(contents_file_path);

    println!(
        "contents file path: {}",
        absolute_path_to_contents_file.to_str().unwrap()
    );

    let contents_file_text =
        fs::read_to_string(absolute_path_to_contents_file).expect("could not read contents file");

    let metadata_section = contents_file_text
        .split("</metadata>")
        .next()
        .expect("could not extract metadata from contents file")
        .split("<package")
        .last()
        .expect("could not extract metadata from contents file");

    let manifest_section = contents_file_text
        .split("<manifest>")
        .last()
        .expect("could not extract manifest from contents file")
        .split("</manifest>")
        .next()
        .expect("could not extract manifest from contents file");

    println!("metadata: {}", metadata_section);
    println!("manifest: {}", manifest_section);
}
