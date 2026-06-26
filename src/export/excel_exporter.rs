use crate::dto::user::UserProfileResponse;
use crate::errors::app_error::AppError;
use uuid::Uuid;
use xlsxwriter::*;

pub struct ExcelExporter;

impl ExcelExporter {
    pub fn export_users(users: Vec<UserProfileResponse>) -> Result<Vec<u8>, AppError> {
        // Gunakan nama file unik per request untuk mencegah race condition
        // ketika ada multiple concurrent export requests
        let file_name = format!("/tmp/users_export_{}.xlsx", Uuid::new_v4());
        let workbook = Workbook::new(&file_name)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            
        let mut sheet = workbook.add_worksheet(None)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Headers
        sheet.write_string(0, 0, "ID", None).ok();
        sheet.write_string(0, 1, "Name", None).ok();
        sheet.write_string(0, 2, "Email", None).ok();
        sheet.write_string(0, 3, "Phone", None).ok();
        sheet.write_string(0, 4, "Verified", None).ok();

        for (row, user) in users.into_iter().enumerate() {
            let r = (row + 1) as u32;
            sheet.write_string(r, 0, &user.id.to_string(), None).ok();
            sheet.write_string(r, 1, &user.name, None).ok();
            sheet.write_string(r, 2, &user.email, None).ok();
            sheet.write_string(r, 3, user.phone.as_deref().unwrap_or_default(), None).ok();
            sheet.write_string(r, 4, &user.is_verified.to_string(), None).ok();
        }

        workbook.close()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // Read back from file unik
        let data = std::fs::read(&file_name)
            .map_err(|e| AppError::StorageError(e.to_string()))?;
            
        // Hapus file temporary setelah dibaca
        let _ = std::fs::remove_file(&file_name);
            
        Ok(data)
    }
}
