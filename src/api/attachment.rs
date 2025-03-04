use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::post;
use actix_web::{error, web, Error, HttpResponse};
use async_fs::File;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use futures::AsyncWriteExt;
use image::{ImageReader, Limits};
use rand::Rng;
use std::path::Path;

use crate::api::user::get_login_uid;
use crate::models::Attachment;
use crate::BB8Pool;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    file: TempFile,
}

/// Uploads an attachment via multipart form. The provided form must contain
/// a field named "file" with the attachment as content. The endpoint returns
/// information on the newly created attachment, such as an attachment id,
/// which can later be used to link uploaded attachment to an item listing.
/// Cron will take care of removing old attachments not bound to items.
#[post("/attachment/upload")]
pub async fn upload(
    pool: web::Data<BB8Pool>,
    session: Session,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<HttpResponse, Error> {
    use crate::schema::attachments;

    const PUBLIC_DIR: &str = "public"; // TODO: make configurable
    const EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];
    const RANDOM_FILE_NAME_LENGTH: usize = 20;
    const MAX_IMAGE_RESOLUTION: u32 = 10_000;
    const THUMBNAIL_SIZE: u32 = 320;
    const THUMBNAIL_QUALITY: f32 = 50.0;

    let temp_file = form.file;

    // Validate login
    let user_id =
        get_login_uid(&session)?.ok_or_else(|| error::ErrorUnauthorized("Not logged in"))?;

    // Parse file extension
    let file_name = temp_file
        .file_name
        .ok_or_else(|| error::ErrorBadRequest("No file name provided with file"))?;
    let extension = Path::new(&file_name)
        .extension()
        .ok_or_else(|| error::ErrorBadRequest("File name does not contain extension"))?
        .to_string_lossy()
        .into_owned();
    if !EXTENSIONS.contains(&extension.as_str()) {
        return Err(error::ErrorBadRequest(format!(
            "Bad file extension. Accepted extensions are: {:?}",
            EXTENSIONS
        )));
    }

    // Generate new file name with path
    let (file_path, thumbnail_path) = loop {
        let id = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(RANDOM_FILE_NAME_LENGTH)
            .map(char::from)
            .collect::<String>();
        let path = format!("{PUBLIC_DIR}/{id}.{extension}");
        if (File::open(&path).await).is_err() {
            break (path, format!("{PUBLIC_DIR}/{id}.thumb.webp"));
        }
    };

    // Prevent loading "zip bomb" images before the image is decoded in memory
    let mut decoder = ImageReader::open(temp_file.file.path())
        .map_err(error::ErrorInternalServerError)?
        .with_guessed_format()
        .map_err(error::ErrorInternalServerError)?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024); /* 512 MiB */
    limits.max_image_height = Some(MAX_IMAGE_RESOLUTION);
    limits.max_image_width = Some(MAX_IMAGE_RESOLUTION);
    decoder.limits(limits);

    // Generate thumbnail for image
    let img = decoder.decode().map_err(|_| {
        error::ErrorBadRequest(format!(
            "Could not decode {file_name}. Uploaded image might be too large or corrupted."
        ))
    })?;
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let thumbnail_bytes = webp::Encoder::from_image(&thumbnail)
        .unwrap()
        .encode(THUMBNAIL_QUALITY);
    let mut file = File::create(&thumbnail_path)
        .await
        .map_err(error::ErrorInternalServerError)?;
    file.write_all(&thumbnail_bytes)
        .await
        .map_err(error::ErrorInternalServerError)?;
    file.flush()
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Persist provided image file when everything above has passed without errors
    async_fs::copy(temp_file.file.path(), &file_path)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Index image to db
    let mut con = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let attachment = diesel::insert_into(attachments::table)
        .values((
            attachments::columns::file_path.eq(file_path),
            attachments::columns::thumbnail_path.eq(thumbnail_path),
            attachments::columns::uploader_id.eq(user_id),
            attachments::columns::uploaded_at.eq(chrono::offset::Utc::now()),
        ))
        .returning(Attachment::as_returning())
        .get_result(&mut con)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Return info of newly created attachment
    Ok(HttpResponse::Ok().json(attachment))
}

#[cfg(test)]
mod tests {
    use image::ImageBuffer;
    use rand::random;
    use reqwest::Result;
    use std::sync::Arc;
    use temp_dir::TempDir;

    use crate::api::{item::NewItemQuery, user::UserQuery};

    use super::*;
    const URL: &str = "http://backend:3030";

    // Test attachment uploading
    #[test]
    fn attachment_operations() -> Result<()> {
        // Set things up for testing
        let cookie_provider = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider.clone())
            .build()?;
        let cookie_provider2 = Arc::new(reqwest::cookie::Jar::default());
        let client2 = reqwest::blocking::ClientBuilder::new()
            .cookie_provider(cookie_provider2.clone())
            .build()?;
        let temp_dir = TempDir::new().unwrap();

        // Generate test images
        let random_image_generator = || {
            ImageBuffer::from_fn(500, 300, |_, _| {
                let a = || random::<u8>() % 255_u8;
                image::Rgb([a(), a(), a()])
            })
        };
        let image = random_image_generator();
        let attachment_path = temp_dir.child("test_image.png");
        image.save(&attachment_path).unwrap();
        let image2 = random_image_generator();
        let attachment_path2 = temp_dir.child("test_image_2.jpg");
        image2.save(&attachment_path2).unwrap();

        // Clear database for testing
        let result = client.get(format!("{URL}/api/admin/db/clear")).send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not clear db. Make sure the server is compiled in debug mode."
        );

        // Register test users and log them in to their clients
        let result = client
            .post(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a new user");

        let result = client2
            .post(format!("{URL}/api/user/new"))
            .json(&UserQuery {
                username: "test2".to_string(),
                password: "test".to_string(),
            })
            .send()?;
        assert_eq!(result.status(), 200, "Could not create a second user");

        // Upload attachments
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", attachment_path)
            .unwrap();
        let result = client
            .post(format!("{URL}/api/attachment/upload"))
            .multipart(form)
            .send()?;
        assert_eq!(result.status(), 200, "Could not upload attachment");
        let attachment_id = result.json::<Attachment>().unwrap().id;

        let form2 = reqwest::blocking::multipart::Form::new()
            .file("file", attachment_path2)
            .unwrap();
        let result = client2
            .post(format!("{URL}/api/attachment/upload"))
            .multipart(form2)
            .send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not upload attachment for second user"
        );
        let attachment_id2 = result.json::<Attachment>().unwrap().id;

        // Sell an item with uploaded attachment
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
                attachments: vec![attachment_id],
            })
            .send()?;
        assert_eq!(
            result.status(),
            200,
            "Could not create new item with attachment"
        );

        // Try to sell an item with attachment beloning to another user
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 1,
                price: "1,00".to_string(),
                attachments: vec![attachment_id2],
            })
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Could use attachment which is not owned"
        );

        // Try to re-use attachment
        let result = client
            .post(format!("{URL}/api/item/new"))
            .json(&NewItemQuery {
                title: "test item".to_string(),
                description: "test description".to_string(),
                amount: 3,
                price: "1.11".to_string(),
                attachments: vec![attachment_id],
            })
            .send()?;
        assert_ne!(
            result.status(),
            200,
            "Could use the same attachment in 2 different items"
        );

        Ok(())
    }
}
