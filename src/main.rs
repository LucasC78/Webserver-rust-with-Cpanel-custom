use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::io::Write;
use std::fs::{File, create_dir_all};
use serde::Deserialize;
use std::fs;
use std::path::{Path};

#[derive(Deserialize)]
struct FolderForm {
    folder_name: String,
}

async fn create_and_upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut folder_name = String::new();
    let mut files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;

        if field.name() == "folder_name" {
            let mut folder_name_field = String::new();
            while let Some(chunk) = field.next().await {
                folder_name_field.push_str(&String::from_utf8_lossy(&chunk?));
            }
            folder_name = folder_name_field.trim().to_string();
        }

        if field.name() == "file" {
            let mut file_content = Vec::new();
            while let Some(chunk) = field.next().await {
                file_content.extend_from_slice(&chunk?);
            }
            let filename = field.content_disposition().get_filename().unwrap_or("file").to_string();
            files.push((filename, file_content));
        }
    }

    let folder_path = format!("./public/{}", folder_name);
    if let Err(e) = create_dir_all(&folder_path) {
        return Ok(HttpResponse::InternalServerError().body(format!("Error creating folder: {}", e)));
    }

    for (filename, data) in files {
        let filepath = format!("{}/{}", folder_path, filename);
        let mut file = match File::create(&filepath) {
            Ok(file) => file,
            Err(e) => return Ok(HttpResponse::InternalServerError().body(format!("Error creating file {}: {}", filename, e))),
        };

        if let Err(e) = file.write_all(&data) {
            return Ok(HttpResponse::InternalServerError().body(format!("Error writing file {}: {}", filename, e)));
        }
    }

    Ok(HttpResponse::Ok().body("Folder created and files uploaded successfully!"))
}
async fn list_directory(path: web::Path<String>) -> HttpResponse {

    let dir_path = format!("./public/{}", path);
    let path = Path::new(&dir_path);
    
    if path.exists() && path.is_dir() {

        let entries = fs::read_dir(path)
            .map(|entries| {

                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| {
                        let file_name = entry.file_name().into_string().ok().unwrap_or_default();
                        if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                            format!("<li><a href='/public/{}/'>{}/</a></li>", file_name, file_name)
                        } else {
                            format!("<li><a href='/public/{}/{}'>{}</a></li>", path.display().to_string().replace("./public", ""), file_name, file_name)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .unwrap_or_else(|_| String::from("<li>Erreur lors de la lecture du dossier</li>")); 

       
        HttpResponse::Ok().body(format!(
            "<html>
                <head>
                    <link rel='stylesheet' type='text/css' href='/static/main.css'>
                    <link href='https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0-beta3/css/all.min.css' rel='stylesheet'>
                    <title>Dossier: {}</title>

                </head>
                <body>
                    <nav class='navbar'>
                        <div class='logo'>
                            <i class='fab fa-github nav-icon'></i>
                            <a>LucasC78</a>
                        </div>
                        <ul class='nav-links'>
                            <li><a href='http://127.0.0.1:8080/'>Home</a></li>
                            <li><a href='http://127.0.0.1:8080/allfiles'>All Files</a></li>
                            <li><a href='https://github.com/LucasC78'>Contact</a></li>
                        </ul>
                        <div class='menu-icon' id='menu-icon'>
                            &#9776;
                        </div>
                    </nav>
                    
                    <div class='container'>
                        <h1>Files of your project : {}</h1>
                        <ul>{}</ul>
                        <a onclick='window.history.back()'>Back</a>
                    </div>

                </body>
            </html>",
            path.display(),
            path.display(),
            entries
        ))
    } else {
        HttpResponse::NotFound().body("Folder not found")
    }
}

async fn delete_folder_web(folder: web::Form<FolderForm>) -> HttpResponse {
    let folder_path = format!("./public/{}", folder.folder_name);
    if Path::new(&folder_path).exists() {
        if let Err(e) = fs::remove_dir_all(&folder_path) {
            return HttpResponse::InternalServerError().body(format!("Error deleting folder: {}", e));
        }
        return HttpResponse::Ok().body("<script>window.location.href = '/allfiles';</script>");
    } else {
        HttpResponse::NotFound().body("Folder not found")
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()

            .route("/", web::get().to(|| async {
                HttpResponse::Ok().body(include_str!("../static/cpanel.html"))
            }))

            .route("/create_and_upload", web::post().to(create_and_upload))

            .route("/allfiles", web::get().to(|| async {
                let files = fs::read_dir("./public")
                    .map(|entries| {

                        entries
                            .filter_map(|entry| entry.ok())
                            .map(|entry| {
                                let file_name = entry.file_name().into_string().ok().unwrap_or_default();
                    if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                        format!(
                            "<li>
                                <a href='/public/{}/'>{}</a>
                                <form action='/delete_folder' method='post' style='display:inline;'>
                                    <input type='hidden' name='folder_name' value='{}'>
                                    <button type='submit' class='fas fa-trash-alt'></button>
                                </form>
                            </li>",
                            file_name, file_name, file_name
                        )
                    } else {
                        format!(
                            "<li>
                                <a href='/public/{}/{}'>{}</a>
                                <form action='/delete_folder' method='post' style='display:inline;'>
                                    <input type='hidden' name='folder_name' value='{}'>
                                    <button type='submit' class='fas fa-trash-alt'></button>
                                </form>
                            </li>",
                            file_name, file_name, file_name, file_name
                        )
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or_else(|_| String::from("<li>Erreur lors de la lecture du dossier</li>"));



                HttpResponse::Ok().body(format!(
                    "<html>
                        <head>
                            <link rel='stylesheet' type='text/css' href='/static/main.css'>
                            <link href='https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0-beta3/css/all.min.css' rel='stylesheet'>
                            <title>Liste des Dossiers</title>
                        </head>
                        <body>

                            <nav class='navbar'>
                                <div class='logo'>
                                    <i class='fab fa-github nav-icon'></i>
                                    <a>LucasC78</a>
                                </div>
                                <ul class='nav-links'>
                                    <li><a href='http://127.0.0.1:8080/'>Home</a></li>
                                    <li><a href='http://127.0.0.1:8080/allfiles'>All Files</a></li>
                                    <li><a href='https://github.com/LucasC78'>Contact</a></li>
                                </ul>
                                <div class='menu-icon' id='menu-icon'>
                                    &#9776;
                                </div>
                            </nav>
                            
                            <div class='container'>
                                <h1>All projects</h1>
                                <ul>{}</ul>
                                
                            </div>
                            
                        </body>
                    </html>",
                    files
                ))
            }))

            .route("/public/{folder_name}/", web::get().to(list_directory))

            .route("/delete_folder", web::post().to(delete_folder_web))

            .service(Files::new("/static", "./static"))

            .service(Files::new("/public", "./public").show_files_listing())
    })
    
    .bind("127.0.0.1:8080")?
    .run()
    .await
}



