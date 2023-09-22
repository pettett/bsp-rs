// for an item materials/x/y/z.vmt
pub trait VPath {
    // return vmt
    fn ext(&self) -> &str;
    // return z
    fn filename(&self) -> &str;

    fn dir(&self) -> String;
}
pub struct VGlobalPath<'a> {
    path: &'a str,
}

impl<'a> VGlobalPath<'a> {
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }
}
pub struct VLocalPath<'a> {
    local_path: &'a str,
    root_directory: &'static str,
    ext: &'static str,
}
pub struct VSplitPath<'a> {
    directory: &'a str,
    filename: &'a str,
    ext: &'a str,
}

impl<'a> VSplitPath<'a> {
    pub fn new(directory: &'a str, filename: &'a str, ext: &'a str) -> Self {
        Self {
            directory,
            filename,
            ext,
        }
    }
}
impl<'a> VLocalPath<'a> {
    pub fn new(root_directory: &'static str, local_path: &'a str, ext: &'static str) -> Self {
        Self {
            root_directory,
            local_path,
            ext,
        }
    }
}

impl<'a> From<&'a str> for VGlobalPath<'a> {
    fn from(value: &'a str) -> Self {
        VGlobalPath { path: value }
    }
}

impl<'a> VPath for VLocalPath<'a> {
    fn ext(&self) -> &str {
        self.ext
    }

    fn filename(&self) -> &str {
        if let Some(last_sep) = self.local_path.rfind('/') {
            &self.local_path[last_sep + 1..]
        } else {
            &self.local_path
        }
    }

    fn dir(&self) -> String {
        let d = if let Some(last_sep) = self.local_path.rfind('/') {
            &self.local_path[..last_sep]
        } else {
            ""
        };

        format!("{}/{}", self.root_directory, d)
    }
}

impl<'a> VPath for VGlobalPath<'a> {
    fn ext(&self) -> &str {
        if let Some(ext_sep) = self.path.rfind('.') {
            &self.path[ext_sep + 1..]
        } else {
            ""
        }
    }

    fn filename(&self) -> &str {
        if let Some(last_sep) = self.path.rfind('/') {
            if let Some(ext_sep) = self.path.rfind('.') {
                &self.path[last_sep + 1..ext_sep]
            } else {
                &self.path[last_sep + 1..]
            }
        } else {
            &self.path
        }
    }

    fn dir(&self) -> String {
        if let Some(last_sep) = self.path.rfind('/') {
            self.path[..last_sep].to_owned()
        } else {
            "".to_owned()
        }
    }
}
impl<'a> VPath for VSplitPath<'a> {
    fn ext(&self) -> &str {
        self.ext
    }

    fn filename(&self) -> &str {
        self.filename
    }

    fn dir(&self) -> String {
        self.directory.to_owned()
    }
}
