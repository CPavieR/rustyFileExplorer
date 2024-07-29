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
    let mut currentPath = get_current_dir().unwrap_or_default();
    let mut searchActive = false;
    match get_filtered_content_dir(&currentPath, &"".to_string(), false) {
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
    
    let mut files = get_filtered_content_dir(&currentPath, &"".to_string(), false).unwrap_or_default();
    let mut latestScannedFolder = currentPath.clone();
    let mut searchString = "".to_string();
    //getContentDir(&currentPath).;
    eframe::run_simple_native("The rusty explorer", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                files = get_filtered_content_dir(&currentPath, &"".to_string(), false).unwrap();
            }
            if latestScannedFolder != currentPath {
                files = get_filtered_content_dir(&currentPath, &"".to_string(), false).unwrap();
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
                let clonedSearchString = searchString.clone();
                let searchWorker = thread::spawn(move || {
                    let searchResult = search_worker(clonedCurrentPath, clonedSearchString, cloned_tx);
                });
                //we wait for a communication from the thread

            }
            if(searchActive){
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
                                            match get_filtered_content_dir(
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

fn get_current_dir() -> std::io::Result<std::path::PathBuf> {
    let relative_app_path = std::env::current_exe()?;
    let absolute_app_path = relative_app_path.canonicalize()?;
    let path_file = absolute_app_path.parent().unwrap();
    Ok(path_file.to_path_buf())
}

fn get_filtered_content_dir(
    dir_path: &PathBuf,
    filter: &String,
    filter_active: bool,
) -> Result<Vec<File>, std::io::Error> {
    let mut files = Vec::new();
    if(!filter_active){
        files.push(File {
            name: "..".to_string(),
            fileType: TypeFile::Folder,
            size: 0,
            completePath: dir_path.clone().parent().unwrap().to_path_buf(),
        });
    }
    for entry in std::fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let filename = entry.file_name();
        let filename = filename.to_str().unwrap().to_string();
        let file_size = entry.metadata().unwrap().len();
        let file_type = if entry.metadata().unwrap().is_dir() {
            TypeFile::Folder
        } else {
            TypeFile::File
        };
        let absolute_path = entry.path();
        if filter_active && filename.contains(filter) {
            files.push(File {
                name: absolute_path.to_str().unwrap_or_default().to_owned(),
                fileType: file_type,
                completePath: absolute_path,
                size: file_size,
            });
        } else {
            if (!filter_active) {
                files.push(File {
                    name: filename,
                    fileType: file_type,
                    completePath: absolute_path,
                    size: file_size,
                });
            }
        }
    }
    Ok(files)
}

fn search_worker(
    topFolder: PathBuf,
    filter: String,
    tx: mpsc::Sender<Vec<File>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files_filtered = Vec::new();
    let mut folders = Vec::new();
    folders.push(topFolder.clone());
    let mut explored_folders = Vec::new();
    let copy_top_folder = topFolder.clone();
    let top_folder_as_str = copy_top_folder.to_str().unwrap_or_default();
    while !folders.is_empty() {
        if let Some(folder) = folders.pop() {
            files_filtered.clear();
            let files = get_filtered_content_dir(&folder, &filter, true)?;
            for file in files {
                let _ = file.name.replace(top_folder_as_str, "");
                files_filtered.push(file);
            }
            explored_folders.push(folder.clone());
            let sub_folders = extract_subfolders(&folder)?;
            for sub_folder in sub_folders {
                if !explored_folders.contains(&sub_folder) {
                    folders.push(sub_folder);
                }
            }
            //we send the files to the main thread
            let res = tx.send(files_filtered.clone());
            match res {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }

        }
    }
    Ok(())
}

fn extract_subfolders(topFolder: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(topFolder)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            folders.push(entry.path());
        }
    }
    Ok(folders)
}
