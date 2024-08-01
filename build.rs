use std::fs::File;
use std::io::Write;

use shadow_rs::{git_clean, is_release, SdResult};

fn main() -> SdResult<()> {
    // no support for both a deny list _and_ an append hook. :(
    // let mut deny = BTreeSet::new();
    // deny.insert(shadow_rs::CARGO_MANIFEST_DIR);
    // deny.insert(shadow_rs::CARGO_METADATA);
    // deny.insert(shadow_rs::CARGO_TREE);
    // deny.insert(shadow_rs::CARGO_VERSION);
    // deny.insert(shadow_rs::COMMIT_AUTHOR);
    // deny.insert(shadow_rs::COMMIT_DATE);
    // deny.insert(shadow_rs::COMMIT_DATE_2822);
    // deny.insert(shadow_rs::COMMIT_DATE_3339);
    // deny.insert(shadow_rs::COMMIT_EMAIL);
    // deny.insert(shadow_rs::COMMIT_HASH);
    // deny.insert(shadow_rs::GIT_CLEAN);
    // deny.insert(shadow_rs::GIT_STATUS_FILE);
    // deny.insert(shadow_rs::LAST_TAG);
    // deny.insert(shadow_rs::PKG_DESCRIPTION);
    // deny.insert(shadow_rs::PKG_VERSION_MAJOR);
    // deny.insert(shadow_rs::PKG_VERSION_MINOR);
    // deny.insert(shadow_rs::PKG_VERSION_PATCH);
    // deny.insert(shadow_rs::PKG_VERSION_PRE);
    // shadow_rs::new_deny(deny)
    shadow_rs::new_hook(append_write_const)
}

fn append_write_const(mut file: &File) -> SdResult<()> {
    if git_clean() {
        writeln!(file, r#"pub const DIRTY_SUFFIX: &str = "";"#)?;
        writeln!(file, r#"pub const DIRTY_LINE: &str = "";"#)?;
    } else {
        writeln!(file, r#"pub const DIRTY_SUFFIX: &str = "?!";"#)?;
        writeln!(file, r#"pub const DIRTY_LINE: &str = "\n..dirty?!";"#)?;
    }
    if is_release() {
        writeln!(file, r#"pub const TYPE_SUFFIX: &str = "";"#)?;
        writeln!(file, r#"pub const TYPE_LINE: &str = "";"#)?;
    } else {
        writeln!(file, r#"pub const TYPE_SUFFIX: &str = "-debug";"#)?;
        writeln!(file, r#"pub const TYPE_LINE: &str = "\n..debug";"#)?;
    }
    writeln!(
        file,
        "pub const SHORT_VERSION: &str = shadow_rs::formatcp!(\"{{}}-{{}}{{}}{{}}\",
        PKG_VERSION,
        SHORT_COMMIT,
        DIRTY_SUFFIX,
        TYPE_SUFFIX);"
    )?;
    writeln!(
        file,
        "pub const LONG_VERSION: &str = shadow_rs::formatcp!(r#\"{{}}
commit_hash:{{}}
build_time:{{}}
build_env:{{}},{{}}{{}}{{}}\"#,
        PKG_VERSION, 
        SHORT_COMMIT,
        BUILD_TIME,
        RUST_VERSION,
        RUST_CHANNEL,
        DIRTY_LINE,
        TYPE_LINE);"
    )?;
    Ok(())
}
