extern crate protobuf_parser;
extern crate protobuf;
extern crate protobuf_codegen;

mod convert;

use std::collections::HashMap;
use std::path::Path;
use std::io;
use std::io::Read;
use std::fs;


// TODO: merge with protoc-rust def
#[derive(Debug, Default)]
pub struct Args<'a> {
    /// --lang_out= param
    pub out_dir: &'a str,
    /// -I args
    pub includes: &'a [&'a str],
    /// List of .proto files to compile
    pub input: &'a [&'a str],
}

/// Convert OS path to protobuf path (with slashes)
/// Function is `pub(crate)` for test.
pub(crate) fn relative_path_to_protobuf_path(path: &Path) -> String {
    assert!(path.is_relative());
    let path = path.to_str().expect("not a valid UTF-8 name");
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_owned()
    }
}

#[derive(Clone)]
struct FileDescriptorPair {
    parsed: protobuf_parser:: FileDescriptor,
    descriptor: protobuf::descriptor::FileDescriptorProto,
}

struct Run<'a> {
    parsed_files: HashMap<String, FileDescriptorPair>,
    args: Args<'a>,
}

impl<'a> Run<'a> {
    fn get_file_and_all_deps_already_parsed(
        &self, protobuf_path: &str, result: &mut HashMap<String, FileDescriptorPair>)
    {
        if let Some(_) = result.get(protobuf_path) {
            return;
        }

        let pair = self.parsed_files.get(protobuf_path).expect("must be already parsed");
        result.insert(protobuf_path.to_owned(), pair.clone());

        self.get_all_deps_already_parsed(&pair.parsed, result);
    }

    fn get_all_deps_already_parsed(
        &self,
        parsed: &protobuf_parser::FileDescriptor,
        result: &mut HashMap<String, FileDescriptorPair>)
    {
        for import in &parsed.import_paths {
            self.get_file_and_all_deps_already_parsed(import.to_str().unwrap(), result);
        }
    }

    fn add_file(&mut self, protobuf_path: &str, fs_path: &Path) -> io::Result<()> {
        if let Some(_) = self.parsed_files.get(protobuf_path) {
            return Ok(());
        }

        let mut content = Vec::new();
        fs::File::open(fs_path)?.read_to_end(&mut content)?;

        let parsed = protobuf_parser::FileDescriptor::parse(content)
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other,
                    format!("failed to parse {:?}: {:?}", fs_path, e))
            })?;

        for import_path in &parsed.import_paths {
            self.add_imported_file(import_path.to_str().unwrap())?;
        }

        let mut this_file_deps = HashMap::new();
        self.get_all_deps_already_parsed(&parsed, &mut this_file_deps);

        let this_file_deps: Vec<_> = this_file_deps.into_iter().map(|(_, v)| v.parsed).collect();

        let descriptor = convert::file_descriptor(
            protobuf_path.to_owned(), &parsed, &this_file_deps);

        self.parsed_files.insert(
            protobuf_path.to_owned(), FileDescriptorPair { parsed, descriptor });

        Ok(())
    }

    fn add_imported_file(&mut self, protobuf_path: &str) -> io::Result<()> {
        for include_dir in self.args.includes {
            let fs_path = Path::new(include_dir).join(protobuf_path);
            if fs_path.exists() {
                return self.add_file(protobuf_path, &fs_path)
            }
        }

        Err(io::Error::new(io::ErrorKind::Other,
             format!("protobuf path {:?} is not found in import path {:?}",
                 protobuf_path, self.args.includes)))
    }

    fn add_fs_file(&mut self, fs_path: &Path) -> io::Result<String> {
        let relative_path = self.args.includes.iter()
            .filter_map(|include_dir| fs_path.strip_prefix(include_dir).ok())
            .next();

        match relative_path {
            Some(relative_path) => {
                let protobuf_path = relative_path_to_protobuf_path(relative_path);
                self.add_file(&protobuf_path, fs_path)?;
                Ok(protobuf_path)
            }
            None => {
                Err(io::Error::new(io::ErrorKind::Other,
                    format!("file {:?} must reside in include path {:?}",
                        fs_path, self.args.includes)))
            }
        }
    }
}

/// Like `protoc --rust_out=...` but without requiring `protoc` or `protoc-gen-rust`
/// commands in `$PATH`.
pub fn run(args: Args) -> io::Result<()> {
    let mut run = Run { parsed_files: HashMap::new(), args };

    let mut relative_paths = Vec::new();

    for input in run.args.input {
        relative_paths.push(run.add_fs_file(&Path::new(input))?);
    }

    let file_descriptors: Vec<_> =
        run.parsed_files.into_iter().map(|(_, v)| v.descriptor).collect();

    protobuf_codegen::gen_and_write(
        &file_descriptors, &relative_paths, &Path::new(&run.args.out_dir))
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn test_relative_path_to_protobuf_path_windows() {
        assert_eq!("foo/bar.proto", relative_path_to_protobuf_path(&Path::new("foo\\bar.proto")));
    }

    #[test]
    fn test_relative_path_to_protobuf_path() {
        assert_eq!("foo/bar.proto", relative_path_to_protobuf_path(&Path::new("foo/bar.proto")));
    }
}
