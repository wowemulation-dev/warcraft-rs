//! Test listfile modification and updating

use tempfile::NamedTempFile;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, FormatVersion, ListfileOption, MutableArchive,
};

#[test]
fn test_listfile_updates_correctly() {
    // Create initial archive with some files
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Build initial archive with automatic listfile
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .add_file_data(b"File 1 content".to_vec(), "file1.txt")
        .add_file_data(b"File 2 content".to_vec(), "file2.txt")
        .build(path)
        .unwrap();

    // Open and verify initial contents
    {
        let mut archive = Archive::open(path).unwrap();
        let files = archive.list().unwrap();
        assert_eq!(files.len(), 3); // 2 files + listfile

        // Check listfile content
        let listfile_data = archive.read_file("(listfile)").unwrap();
        let listfile_content = String::from_utf8_lossy(&listfile_data);
        assert!(listfile_content.contains("file1.txt"));
        assert!(listfile_content.contains("file2.txt"));
        assert!(listfile_content.contains("(listfile)"));
    }

    // Open for modification and add more files
    {
        let mut mutable = MutableArchive::open(path).unwrap();

        // Add new files
        mutable
            .add_file_data(
                b"File 3 content".as_ref(),
                "file3.txt",
                AddFileOptions::default(),
            )
            .unwrap();
        mutable
            .add_file_data(
                b"File 4 content".as_ref(),
                "file4.txt",
                AddFileOptions::default(),
            )
            .unwrap();

        // Flush changes
        mutable.flush().unwrap();
    }

    // Reopen and verify all files are listed
    {
        let mut archive = Archive::open(path).unwrap();
        let files = archive.list().unwrap();

        // Should have 5 files now: file1.txt, file2.txt, file3.txt, file4.txt, (listfile)
        assert_eq!(
            files.len(),
            5,
            "Expected 5 files, found: {:?}",
            files.iter().map(|f| &f.name).collect::<Vec<_>>()
        );

        // Check that all files are in the list
        let file_names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(
            file_names.contains(&"file1.txt"),
            "file1.txt not found in: {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"file2.txt"),
            "file2.txt not found in: {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"file3.txt"),
            "file3.txt not found in: {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"file4.txt"),
            "file4.txt not found in: {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"(listfile)"),
            "(listfile) not found in: {:?}",
            file_names
        );

        // Verify we can read all files
        assert_eq!(archive.read_file("file1.txt").unwrap(), b"File 1 content");
        assert_eq!(archive.read_file("file2.txt").unwrap(), b"File 2 content");
        assert_eq!(archive.read_file("file3.txt").unwrap(), b"File 3 content");
        assert_eq!(archive.read_file("file4.txt").unwrap(), b"File 4 content");

        // Check updated listfile content
        let listfile_data = archive.read_file("(listfile)").unwrap();
        let listfile_content = String::from_utf8_lossy(&listfile_data);
        assert!(
            listfile_content.contains("file1.txt"),
            "file1.txt not in listfile"
        );
        assert!(
            listfile_content.contains("file2.txt"),
            "file2.txt not in listfile"
        );
        assert!(
            listfile_content.contains("file3.txt"),
            "file3.txt not in listfile"
        );
        assert!(
            listfile_content.contains("file4.txt"),
            "file4.txt not in listfile"
        );
        assert!(
            listfile_content.contains("(listfile)"),
            "(listfile) not in listfile"
        );
    }
}

#[test]
fn test_multiple_listfile_updates() {
    // Test that multiple updates don't cause block table bloat
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Build initial archive
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .add_file_data(b"Initial file".to_vec(), "initial.txt")
        .build(path)
        .unwrap();

    // Get initial block count
    let initial_block_count = {
        let mut archive = Archive::open(path).unwrap();
        archive.get_info().unwrap().file_count
    };

    // Add files one at a time (each triggers listfile update)
    {
        let mut mutable = MutableArchive::open(path).unwrap();

        for i in 1..=5 {
            let filename = format!("added_{}.txt", i);
            let content = format!("Added file {}", i);
            mutable
                .add_file_data(content.as_bytes(), &filename, AddFileOptions::default())
                .unwrap();
        }

        mutable.flush().unwrap();
    }

    // Check final state
    let mut final_archive = Archive::open(path).unwrap();
    let final_info = final_archive.get_info().unwrap();

    // We should have: initial.txt + 5 added files + (listfile) = 7 files
    assert_eq!(final_info.file_count, 7, "Expected 7 files total");

    // The block table should not have grown excessively
    // Without the fix, it would grow by 2 for each added file (file + listfile update)
    // With the fix, it should only grow by 1 for each added file
    let block_growth = final_info.file_count - initial_block_count;
    assert_eq!(
        block_growth, 5,
        "Block table grew by {} instead of expected 5",
        block_growth
    );
}
