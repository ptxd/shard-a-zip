use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::utils::{format_size, get_output_filename, BUFFER_SIZE, MAX_ZIP_SIZE};

/// Result of the splitting operation
pub struct SplitResult {
    pub output_files: Vec<std::path::PathBuf>,
    pub total_files_processed: usize,
    pub files_split: usize,
}

/// Splits a ZIP file into multiple smaller ZIP files
pub fn split_zip(source_path: &Path) -> Result<SplitResult> {
    let file = File::open(source_path).context("Failed to open source ZIP file")?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).context("Failed to read ZIP archive")?;

    let total_files = archive.len();
    let mut output_files: Vec<std::path::PathBuf> = Vec::new();
    let mut current_zip_index = 1;
    let mut current_size: u64 = 0;
    let mut files_split = 0;
    let mut manifest_entries: Vec<String> = Vec::new();

    // Create progress bar
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    // Initialize first output ZIP
    let mut current_output_path = get_output_filename(source_path, current_zip_index);
    let mut current_zip = create_zip_writer(&current_output_path)?;
    output_files.push(current_output_path.clone());

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .large_file(true);

    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();
        let file_size = file.size();

        pb.set_message(format!("Processing: {}", truncate_name(&file_name, 30)));

        // Skip directories
        if file.is_dir() {
            current_zip.add_directory(&file_name, options.clone())?;
            pb.inc(1);
            continue;
        }

        // Check if this single file is larger than MAX_ZIP_SIZE
        if file_size > MAX_ZIP_SIZE {
            // Need to split this file into parts
            files_split += 1;
            let parts = split_large_file(&mut file, &file_name, source_path, &mut current_zip_index, &mut output_files, &options)?;
            
            manifest_entries.push(format!(
                "SPLIT: {} -> {} parts ({} each, original: {})",
                file_name,
                parts,
                format_size(MAX_ZIP_SIZE),
                format_size(file_size)
            ));

            // Start fresh zip after large file
            current_zip.finish()?;
            current_zip_index += 1;
            current_output_path = get_output_filename(source_path, current_zip_index);
            current_zip = create_zip_writer(&current_output_path)?;
            output_files.push(current_output_path.clone());
            current_size = 0;
        } else {
            // Check if adding this file would exceed the limit
            if current_size + file_size > MAX_ZIP_SIZE && current_size > 0 {
                // Close current ZIP and start a new one
                current_zip.finish()?;
                current_zip_index += 1;
                current_output_path = get_output_filename(source_path, current_zip_index);
                current_zip = create_zip_writer(&current_output_path)?;
                output_files.push(current_output_path.clone());
                current_size = 0;
            }

            // Write file to current ZIP using streaming
            current_zip.start_file(&file_name, options.clone())?;
            let bytes_written = stream_copy(&mut file, &mut current_zip)?;
            current_size += bytes_written;
        }

        pb.inc(1);
    }

    // Write manifest if any files were split
    if !manifest_entries.is_empty() {
        let manifest_content = generate_manifest(&manifest_entries);
        current_zip.start_file("_SHARD_A_ZIP_MANIFEST.txt", options.clone())?;
        current_zip.write_all(manifest_content.as_bytes())?;
    }

    // Finish the last ZIP
    current_zip.finish()?;

    // Remove empty ZIPs (last one might be empty)
    output_files.retain(|p| {
        if let Ok(metadata) = std::fs::metadata(p) {
            if metadata.len() < 100 {
                let _ = std::fs::remove_file(p);
                return false;
            }
        }
        true
    });

    pb.finish_with_message("Complete!");

    Ok(SplitResult {
        output_files,
        total_files_processed: total_files,
        files_split,
    })
}

/// Creates a new ZIP writer for the given path
fn create_zip_writer(path: &Path) -> Result<ZipWriter<BufWriter<File>>> {
    let file = File::create(path).context("Failed to create output ZIP file")?;
    let writer = BufWriter::new(file);
    Ok(ZipWriter::new(writer))
}

/// Streams data from reader to writer efficiently
fn stream_copy<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut total = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
        total += bytes_read as u64;
    }

    Ok(total)
}

/// Splits a large file into multiple parts across multiple ZIP files
fn split_large_file<R: Read>(
    reader: &mut R,
    file_name: &str,
    source_path: &Path,
    zip_index: &mut usize,
    output_files: &mut Vec<std::path::PathBuf>,
    options: &FileOptions<()>,
) -> Result<usize> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut part_num = 1;
    let mut bytes_in_part: u64 = 0;

    // Start first part in a new ZIP
    *zip_index += 1;
    let mut current_path = get_output_filename(source_path, *zip_index);
    let mut current_zip = create_zip_writer(&current_path)?;
    output_files.push(current_path.clone());

    let part_name = format!("{}.part{:03}", file_name, part_num);
    current_zip.start_file(&part_name, options.clone())?;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        // Check if we need to start a new part
        if bytes_in_part + bytes_read as u64 > MAX_ZIP_SIZE {
            // Finish current ZIP
            current_zip.finish()?;

            // Start new ZIP with next part
            part_num += 1;
            *zip_index += 1;
            current_path = get_output_filename(source_path, *zip_index);
            current_zip = create_zip_writer(&current_path)?;
            output_files.push(current_path.clone());

            let part_name = format!("{}.part{:03}", file_name, part_num);
            current_zip.start_file(&part_name, options.clone())?;
            bytes_in_part = 0;
        }

        current_zip.write_all(&buffer[..bytes_read])?;
        bytes_in_part += bytes_read as u64;
    }

    current_zip.finish()?;
    Ok(part_num)
}

/// Generates manifest content explaining how to rejoin split files
fn generate_manifest(entries: &[String]) -> String {
    let mut content = String::from(
        "═══════════════════════════════════════════════════════════════════
                         SHARD-A-ZIP MANIFEST
═══════════════════════════════════════════════════════════════════

This archive was created by Shard-A-Zip. Some files were too large
to fit within the 25MB limit and have been split into parts.

HOW TO REJOIN SPLIT FILES:
--------------------------
1. Extract all ZIP files to the same directory
2. For each split file (*.part001, *.part002, etc.), run:

   Windows (PowerShell):
   Get-Content file.ext.part* -Raw | Set-Content file.ext -NoNewline

   Windows (CMD):
   copy /b file.ext.part001+file.ext.part002+... file.ext

   Linux/macOS:
   cat file.ext.part* > file.ext

SPLIT FILES IN THIS ARCHIVE:
----------------------------
",
    );

    for entry in entries {
        content.push_str(&format!("• {}\n", entry));
    }

    content.push_str(
        "
═══════════════════════════════════════════════════════════════════
",
    );

    content
}

/// Truncates a filename for display
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("...{}", &name[name.len() - max_len + 3..])
    }
}
