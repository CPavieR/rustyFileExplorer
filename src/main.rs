#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example



use std::path::PathBuf;
use sha2;
use clipboard::{ClipboardContext, ClipboardProvider};
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
    file_type: TypeFile,
    size: u64,
    complete_path: PathBuf,
}
use eframe::egui;
use egui::{Color32, RichText};
use egui_extras::{Column, TableBuilder};
fn main() -> eframe::Result {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 900.0]),
        ..Default::default()
    };
    let mut files : Vec<File> = Vec::new();
    let mut filtered_files_ref: Vec<File> = Vec::new();
    let mut current_path = get_current_dir().unwrap_or_default();
    let mut search_string = "".to_string();
    match get_filtered_content_dir(&current_path, &search_string, search_string == "") {
        Ok(files_list) => {
            files = files_list.clone();
            filtered_files_ref = files_list.clone();
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    //let mut files = getContentDir(&currentPath).unwrap_or_default();
    let mut latest_scanned_folder = current_path.clone();
    
    //getContentDir(&currentPath).;
    eframe::run_simple_native("The rusty explorer", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                files = get_filtered_content_dir(&current_path, &search_string, search_string == "").unwrap_or_default();
            }
            if latest_scanned_folder != current_path {
                files = get_filtered_content_dir(&current_path, &search_string, search_string == "").unwrap_or_default();
                latest_scanned_folder = current_path.clone();
            }
            ui.heading("The rusty explorer");
            ui.label(format!("Current path: {:?}", current_path.to_str().unwrap()));
            let search_field = ui.text_edit_singleline(&mut search_string).on_hover_text("Search for files");
            
            if search_field.changed() &&!search_string.is_empty() {
                filtered_files_ref = files.iter().filter(|file| file.name.contains(&search_string))
                    .cloned().collect();
                
            }
            if search_field.changed() && (search_string.is_empty()) {
                filtered_files_ref = files.clone();
            }
            ui.separator();
            let mut table = TableBuilder::new(ui)
                .column(Column::remainder().at_least(100.0))
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
                    header.col(|ui| {
                        ui.strong("actions");
                    });
                })
                .body(|body| {
                    body.rows(15.0, files.len(), |mut file_row| {
                        match filtered_files_ref.get(file_row.index()) {
                            Some(file) => {
                                let file = file.clone();
                                if file.file_type == TypeFile::Folder {
                                    file_row.col(|ui| {
                                        if ui.button(file.name.clone()).clicked() {
                                            current_path = file.complete_path.clone();
                                            match get_filtered_content_dir(&current_path, &search_string, search_string == "") {
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
                                    file_row.col(|ui| {
                                        ui.label(&file.name);
                                    });
                                }

                                file_row.col(|ui| {
                                    ui.label(match file.file_type {
                                        TypeFile::File => "File",
                                        TypeFile::Folder => "Folder",
                                        TypeFile::Drive => "Drive",
                                    });
                                });
                                file_row.col(|ui| {
                                    ui.label(file.size.to_string());
                                });
                                file_row.col(|ui| {
                                    //we add the duplicate button and delete button in an horizontal line
                                    ui.horizontal(|ui| {
                                        if ui.button(RichText::new("Copy path").color(Color32::BLUE)).clicked() {
                                            copy_file_to_clipboard(file.complete_path.clone());
                                        }
                                        if ui.button(RichText::new("Duplicate").color(Color32::GREEN)).clicked() {
                                            duplicate_file(file.complete_path.clone());
                                        }
                                        /**/if ui.button(RichText::new("Delete").color(Color32::RED)).clicked() {
                                            if(file.file_type == TypeFile::File){
                                                delete_file(file.complete_path.clone());
                                            }
                                            delete_file(file.complete_path.clone());
                                            //println!("Delete file {}", file.complete_path.to_str().unwrap());
                                        }
                                    });


                                });
                            }
                            None => {}
                        }
                    });
                });
            for file in files.iter() {
                                    if file.file_type == TypeFile::Folder {

                        }}
        });
    })
}

fn get_current_dir() -> std::io::Result<std::path::PathBuf> {
    let app_file = std::env::current_exe()?;
    let app_file_canonize = app_file.canonicalize()?;
    let app_file_var = app_file_canonize.parent().unwrap();
    Ok(app_file_var.to_path_buf())
}

fn get_filtered_content_dir(
    dir_path: &PathBuf,
    filter: &String,
    filter_active: bool,
) -> Result<Vec<File>, std::io::Error> {
    let mut files = Vec::new();
    files.push(File {
        name: "..".to_string(),
        file_type: TypeFile::Folder,
        size: 0,
        complete_path: dir_path.clone().parent().unwrap().to_path_buf(),
    });
    for entry in std::fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.file_name();
        let path = path.to_str().unwrap().to_string();
        let file_size = entry.metadata().unwrap().len();
        let file_type = if entry.metadata().unwrap().is_dir() {
            TypeFile::Folder
        } else {
            TypeFile::File
        };
        let absolute_path = entry.path();
        if filter_active && path.to_lowercase().contains(&filter.to_lowercase()) {
            files.push(File {
                name: absolute_path
                    .to_str()
                    .unwrap()
                    .replace(dir_path.to_str().unwrap(), ""),
                file_type,
                complete_path: absolute_path,
                size: file_size,
            });
        } else {
            if !filter_active {
                files.push(File {
                    name: path,
                    file_type,
                    complete_path: absolute_path,
                    size: file_size,
                });
            }
        }
    }
    Ok(files)
}



fn extract_subfolders(top_folder : PathBuf) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(top_folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.file_name();
        let _path = path.to_str().unwrap().to_string();
        let file_type = if entry.metadata().unwrap().is_dir() {
            TypeFile::Folder
        } else {
            TypeFile::File
        };
        if file_type == TypeFile::Folder {
            folders.push(entry.path());
        }
    }
    folders
}

fn copy_file_to_clipboard(file_path : PathBuf) {
    let file_path_str = file_path.to_str().unwrap();
    let mut clipboard: ClipboardContext;
    match clipboard::ClipboardProvider::new() {
        Ok(clip) => {
            clipboard=clip;
            clipboard.set_contents(file_path_str.to_string()).unwrap();
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

fn delete_file(file_path : PathBuf) {
    std::fs::remove_file(file_path).unwrap();
}
fn duplicate_file(file_path : PathBuf) {
    if(!file_path.is_dir()){
    let file_path_str = file_path.to_str().unwrap();
    let file_path_str = file_path_str.to_string();
    let file_path_str = match file_path_str.contains(".") {
        true => file_path_str.replace(".", "_copy."),
        false => file_path_str + "_copy",
    };
    std::fs::copy(file_path, file_path_str).unwrap();
}}
/*
fn calculate_file_hash(file_path : PathBuf) -> String {
    let file = std::fs::File::open(file_path).unwrap();
    let mut hasher = sha2::Sha256::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    format!("{:x}", hash)
}*/
