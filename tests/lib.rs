use std::fs::{File, FileTimes};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::{fixture::ChildPath, prelude::*};
use predicates::prelude::*;

/// A struct to hold the paths of the old and new files
/// created in the test directories.
struct DirItems {
    old_file_path: ChildPath,
    new_file_path: ChildPath,
}

/// Creates a directory with an old file and a new file.
/// The old file is modified to be older than the specified age.
/// If `keep` is true, a `.keep` file is created in the directory.
/// The function returns a `DirItems` struct containing the paths of the old and new files.
fn create_dir(dir: &ChildPath, old_age: &Duration, keep: bool) -> Result<DirItems> {
    dir.create_dir_all()?;
    if keep {
        // create empty .keep file
        let keep_file_path = dir.child(".keep");
        keep_file_path.touch()?;
    }
    let old_file_path = dir.child("old_file.txt");
    old_file_path.touch()?;
    let new_file_path = dir.child("new_file.txt");
    new_file_path.touch()?;

    // change modification time of the old file to be older than the specified age
    let old_file_time = SystemTime::now() - *old_age;
    let times = FileTimes::new().set_modified(old_file_time);
    let old_file = File::options().write(true).open(&old_file_path)?;
    old_file.set_times(times)?;

    Ok(DirItems {
        old_file_path,
        new_file_path,
    })
}

#[test]
fn test_cleanup_stale_data() -> anyhow::Result<()> {
    // Create a temporary directory for testing
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let old_age = Duration::from_secs(10);

    let keep_dir_path = temp_dir.child("keep_dir");

    let remove_dir = create_dir(&temp_dir.child("test1"), &old_age, false)?;
    let keep_dir = create_dir(&keep_dir_path, &old_age, true)?;

    // Run the cleanup command
    let mut cmd = Command::cargo_bin("emcp-tools").unwrap();
    cmd.arg("cleanup-stale-data")
        .arg("--dir")
        .arg(temp_dir.path().to_str().unwrap())
        .arg("--age")
        .arg("5s")
        .assert()
        .success();

    // Check that the old file was removed and the new file still exists
    remove_dir.old_file_path.assert(predicate::path::missing());
    remove_dir.new_file_path.assert(predicate::path::exists());

    keep_dir.old_file_path.assert(predicate::path::exists());
    keep_dir.new_file_path.assert(predicate::path::exists());
    keep_dir_path
        .child(".keep")
        .assert(predicate::path::exists());

    // Check that the given dir is not removed
    let remove_dir_path = temp_dir.child("test_not_remove_given");
    let remove_dir = create_dir(&remove_dir_path, &old_age, false)?;

    let mut cmd = Command::cargo_bin("emcp-tools").unwrap();
    cmd.arg("cleanup-stale-data")
        .arg("--dir")
        .arg(remove_dir_path.path().to_str().unwrap())
        .arg("--age")
        .arg("0s") // ensure that everything is removed in the dir
        .assert()
        .success();

    // but the dir itself has to stay
    // (otherwise one would risk that remove is applied to e.g. a mount point)
    remove_dir_path.assert(predicate::path::exists());
    remove_dir.old_file_path.assert(predicate::path::missing());
    remove_dir.new_file_path.assert(predicate::path::missing());

    Ok(())
}
