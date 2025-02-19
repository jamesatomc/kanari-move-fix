// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::*;
use move_package::source_package::layout::SourcePackageLayout;
use std::{
    fmt::Display,
    fs::create_dir_all,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub const MOVE_STDLIB_PACKAGE_NAME: &str = "MoveStdlib";
pub const MOVE_STDLIB_PACKAGE_PATH: &str = "{ \
    git = \"https://github.com/jamesatomc/kanari-move-fix.git\", \
    subdir = \"framework/packages/move-stdlib\", rev = \"master\" \
}";
pub const MOVE_STDLIB_ADDR_NAME: &str = "std";
pub const MOVE_STDLIB_ADDR_VALUE: &str = "0x1";

pub const KANARI_FRAMEWORK_PACKAGE_NAME: &str = "KanariFramework";
pub const KANARI_FRAMEWORK_PACKAGE_PATH: &str = "{ \
    git = \"https://github.com/jamesatomc/kanari-move-fix.git\", \
    subdir = \"framework/packages/kanari-framework\", rev = \"master\" \
}";
pub const KANARI_FRAMEWORK_ADDR_NAME: &str = "kanari_framework";
pub const KANARI_FRAMEWORK_ADDR_VALUE: &str = "0x2";

/// Create a new Move package with name `name` at `path`. If `path` is not provided the package
/// will be created in the directory `name`.
#[derive(Parser)]
#[clap(name = "new")]
pub struct New {
    /// The name of the package to be created.
    pub name: String,
}

impl New {
    pub fn execute_with_defaults(self, path: Option<PathBuf>) -> anyhow::Result<()> {
        self.execute(
            path,
            "0.0.0",
            [
                (MOVE_STDLIB_PACKAGE_NAME, MOVE_STDLIB_PACKAGE_PATH),
                (KANARI_FRAMEWORK_PACKAGE_NAME, KANARI_FRAMEWORK_PACKAGE_PATH),
            ],
            [
                (MOVE_STDLIB_ADDR_NAME, MOVE_STDLIB_ADDR_VALUE),
                (KANARI_FRAMEWORK_ADDR_NAME, KANARI_FRAMEWORK_ADDR_VALUE),
            ],
            "",
        )
    }

    pub fn execute(
        self,
        path: Option<PathBuf>,
        version: &str,
        deps: impl IntoIterator<Item = (impl Display, impl Display)>,
        addrs: impl IntoIterator<Item = (impl Display, impl Display)>,
        custom: &str, // anything else that needs to end up being in Move.toml (or empty string)
    ) -> anyhow::Result<()> {
        // TODO warn on build config flags
        let Self { name } = self;
        let p: PathBuf;
        let path: &Path = match path {
            Some(path) => {
                p = path;
                &p
            }
            None => Path::new(&name),
        };
        // Create source directory
        create_dir_all(path.join(SourcePackageLayout::Sources.path()))?;

        // Create Move.toml manifest
        let mut w = std::fs::File::create(path.join(SourcePackageLayout::Manifest.path()))?;

        // Create initial module file
        let file_path = path
            .join(SourcePackageLayout::Sources.path())
            .join(format!("{}.move", name));
        let mut file = File::create(file_path)?;
        write!(file, "module {}::{} {{\n\n}}", name, name)?;

        writeln!(
            &mut w,
            "[package]
name = \"{name}\"
version = \"{version}\"

[dependencies]"
        )?;
        for (dep_name, dep_val) in deps {
            writeln!(w, "{dep_name} = {dep_val}")?;
        }

        writeln!(
            w,
            "
[addresses]"
        )?;
        writeln!(w, "{name} = \"0x0\"")?;
        for (addr_name, addr_val) in addrs {
            writeln!(w, "{addr_name} =  \"{addr_val}\"")?;
        }
        if !custom.is_empty() {
            writeln!(w, "{}", custom)?;
        }

        // Create tests directory and basic test file
        create_dir_all(path.join("tests"))?;
        let test_file_path = path.join("tests").join(format!("{}_tests.move", name));
        let mut test_file = File::create(test_file_path)?;
        write!(
            test_file,
            r#"#[test_only]
module {}::{}_tests {{
    use std::debug;
    use std::signer;
    use {}::{};

    #[test]
    fun test_basic() {{
        // Add your test code here
    }}
}}"#,
            name, name, name, name
        )?;

        // Create .gitignore
        create_gitignore(path)?;

        Ok(())
    }
}

fn create_gitignore(project_path: &Path) -> std::io::Result<()> {
    let gitignore_content = r#"# Move build output
build/

# Move cache
.move/

# IDE
.idea/
.vscode/

# OS
.DS_Store
Thumbs.db

# Move coverage and test files
*.coverage
*.test
"#;
    std::fs::write(project_path.join(".gitignore"), gitignore_content)
}
