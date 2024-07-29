#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use core::{alloc, str};

use std::path::PathBuf;
use std::thread;
use std::sync::mpsc;
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
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 900.0]),
        ..Default::default()
    };
    let mut files : Vec<File> = Vec::new();
    let mut filtered_files_ref: Vec<File> = Vec::new();
    let mut currentPath = getCurrentDir().unwrap_or_default();
    let mut searchString = "".to_string();
    match getFilteredContentDir(&currentPath, &searchString, searchString == "") {
        Ok(files_list) => {
            files = files_list.clone();
            filtered_files_ref = files_list.clone();
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    //let mut files = getContentDir(&currentPath).unwrap_or_default();
    let mut latestScannedFolder = currentPath.clone();
    
    //getContentDir(&currentPath).;
    eframe::run_simple_native("The rusty explorer", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                files = getFilteredContentDir(&currentPath, &searchString, searchString == "").unwrap_or_default();
            }
            if latestScannedFolder != currentPath {
                files = getFilteredContentDir(&currentPath, &searchString, searchString == "").unwrap_or_default();
                latestScannedFolder = currentPath.clone();
            }
            ui.heading("The rusty explorer");
            ui.label(format!("Current path: {:?}", currentPath.to_str().unwrap()));
            let searchField = ui.text_edit_singleline(&mut searchString).on_hover_text("Search for files");
            
            if(searchField.changed() &&searchString != ""){
                filtered_files_ref = files.iter().filter(|file| file.name.contains(&searchString))
                    .cloned().collect();
                
            }
            if(searchField.changed() && (searchString == "")){
                filtered_files_ref = files.clone();
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
                                            match getFilteredContentDir(&currentPath, &searchString, searchString == "") {
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
                                    if file.fileType == TypeFile::Folder {

                        }}
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



fn extractSubfolders(topFolder : PathBuf) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(topFolder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.file_name();
        let path = path.to_str().unwrap().to_string();
        let fileType = if entry.metadata().unwrap().is_dir() {
            TypeFile::Folder
        } else {
            TypeFile::File
        };
        if fileType == TypeFile::Folder {
            folders.push(entry.path());
        }
    }
    folders
}
