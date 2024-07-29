#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use core::{alloc, str};

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TypeFile {
    File,
    Folder,
    Drive,
}
#[derive(PartialEq, Eq, Clone, Debug)]
struct File {
    name: String,
    fileType: TypeFile,
    size: u64,
    completePath: PathBuf,
}
use eframe::egui;
use egui_extras::{Column, TableBuilder};
fn main() -> eframe::Result {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    let mut files: Vec<File> = Vec::new();
    let mut filtered_files_ref: Vec<File> = Vec::new();
    let mut currentPath = getCurrentDir().unwrap_or_default();
    let mut searchActive = false;
    match getFilteredContentDir(&currentPath, &"".to_string(), false) {
        Ok(files_list) => {
            files = files_list.clone();
            filtered_files_ref = files_list.clone();
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    //we create a mspc channel to communicate with the search worker
    let (tx, rx): (mpsc::Sender<Vec<File>>, mpsc::Receiver<Vec<File>>) = mpsc::channel();

    let mut files = getFilteredContentDir(&currentPath, &"".to_string(), false).unwrap_or_default();
    let mut latestScannedFolder = currentPath.clone();
    let mut searchString = "".to_string();
    //getContentDir(&currentPath).;
    eframe::run_simple_native("The rusty explorer", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                files = getFilteredContentDir(&currentPath, &"".to_string(), false).unwrap();
            }
            if latestScannedFolder != currentPath {
                files = getFilteredContentDir(&currentPath, &"".to_string(), false).unwrap();
                latestScannedFolder = currentPath.clone();
            }
            ui.heading("The rusty explorer");
            ui.label(format!("Current path: {:?}", currentPath.to_str().unwrap()));
            let searchField = ui
                .text_edit_singleline(&mut searchString)
                .on_hover_text("Search for files");

            if (searchField.changed() && searchString != "") {
                if(!searchActive){
                    searchActive = true;
                    
                    
                }
                filtered_files_ref.clear();

                //we initialize the thread
                let clonedCurrentPath = currentPath.clone();
                let cloned_tx = tx.clone();
                let searchWorker = thread::spawn(move || {
                    let searchResult = searchWorker(clonedCurrentPath, "test".to_string(), cloned_tx);
                });
                //we wait for a communication from the thread

            }
            let searchResult = rx.try_recv();
            match searchResult {
                Ok(result) => {
                    //we add the new files to the list
                    for file in result {
                        filtered_files_ref.push(file);
                    }
                                    }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
            if (searchField.changed() && (searchString == "")) {
                filtered_files_ref = files.clone();
                searchActive = false;
            }

            ui.separator();
            let mut table = TableBuilder::new(ui)
                .column(Column::remainder().at_least(100.0))
                .column(Column::remainder().at_least(100.0))
                .column(Column::remainder().at_least(100.0))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Files");
                    });
                    header.col(|ui| {
                        ui.strong("type");
                    });
                    header.col(|ui| {
                        ui.strong("size");
                    });
                })
                .body(|body| {
                    body.rows(15.0, files.len(), |mut fileRow| {
                        match filtered_files_ref.get(fileRow.index()) {
                            Some(file) => {
                                let file = file.clone();
                                if file.fileType == TypeFile::Folder {
                                    fileRow.col(|ui| {
                                        if ui.button(file.name.clone()).clicked() {
                                            currentPath = file.completePath.clone();
                                            match getFilteredContentDir(
                                                &currentPath,
                                                &"".to_string(),
                                                false,
                                            ) {
                                                Ok(files_list) => {
                                                    files = files_list.clone();
                                                    filtered_files_ref = files_list.clone();
                                                }
                                                Err(e) => {
                                                    eprintln!("Error: {:?}", e);
                                                }
                                            }
                                        }
                                    });
                                } else {
                                    fileRow.col(|ui| {
                                        ui.label(&file.name);
                                    });
                                }

                                fileRow.col(|ui| {
                                    ui.label(match file.fileType {
                                        TypeFile::File => "File",
                                        TypeFile::Folder => "Folder",
                                        TypeFile::Drive => "Drive",
                                    });
                                });
                                fileRow.col(|ui| {
                                    ui.label(file.size.to_string());
                                });
                            }
                            None => {}
                        }
                    });
                });
            for file in files.iter() {
                if file.fileType == TypeFile::Folder {}
            }
        });
    })
}

fn getCurrentDir() -> std::io::Result<std::path::PathBuf> {
    let appFile = std::env::current_exe()?;
    let appFile = appFile.canonicalize()?;
    let appFile = appFile.parent().unwrap();
    Ok(appFile.to_path_buf())
}

fn getFilteredContentDir(
    DirPath: &PathBuf,
    filter: &String,
    filterActive: bool,
) -> Result<Vec<File>, std::io::Error> {
    let mut files = Vec::new();
    files.push(File {
        name: "..".to_string(),
        fileType: TypeFile::Folder,
        size: 0,
        completePath: DirPath.clone().parent().unwrap().to_path_buf(),
    });
    for entry in std::fs::read_dir(DirPath).unwrap() {
        let entry = entry.unwrap();
        let path = entry.file_name();
        let path = path.to_str().unwrap().to_string();
        let fileSize = entry.metadata().unwrap().len();
        let fileType = if entry.metadata().unwrap().is_dir() {
            TypeFile::Folder
        } else {
            TypeFile::File
        };
        let absolutePath = entry.path();
        if (filterActive && path.contains(filter)) {
            files.push(File {
                name: absolutePath
                    .to_str()
                    .unwrap()
                    .replace(DirPath.to_str().unwrap(), ""),
                fileType: fileType,
                completePath: absolutePath,
                size: fileSize,
            });
        } else {
            if (!filterActive) {
                files.push(File {
                    name: path,
                    fileType: fileType,
                    completePath: absolutePath,
                    size: fileSize,
                });
            }
        }
    }
    Ok(files)
}

fn searchWorker(
    topFolder: PathBuf,
    filter: String,
    tx: mpsc::Sender<Vec<File>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut filesFiltered = Vec::new();
    let mut folders = extractSubfolders(&topFolder)?;
    println!("searching in {:?}", topFolder);
    println!("searching for {:?}", filter);
    println!("searching in {:?}", folders);
    let mut exploredFolders = Vec::new();
    
    while !folders.is_empty() {
        if let Some(folder) = folders.pop() {
            filesFiltered.clear();
            let files = getFilteredContentDir(&folder, &filter, true)?;
            for file in files {
                filesFiltered.push(file);
            }
            exploredFolders.push(folder.clone());
            let subFolders = extractSubfolders(&folder)?;
            for subFolder in subFolders {
                if !exploredFolders.contains(&subFolder) {
                    folders.push(subFolder);
                }
            }
            //we send the files to the main thread
            tx.send(filesFiltered.clone());

        }
    }
    Ok(())
}

fn extractSubfolders(topFolder: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(topFolder)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            folders.push(entry.path());
        }
    }
    Ok(folders)
}
