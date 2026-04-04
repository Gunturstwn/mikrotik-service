use crate::dto::user::UserProfileResponse;
use crate::errors::app_error::AppError;
use xlsxwriter::*;

pub struct ExcelExporter;

impl ExcelExporter {
    pub fn export_users(users: Vec<UserProfileResponse>) -> Result<Vec<u8>, AppError> {
        let workbook = Workbook::new("/tmp/users_export.xlsx")
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

        // Read back from file
        let data = std::fs::read("/tmp/users_export.xlsx")
            .map_err(|e| AppError::StorageError(e.to_string()))?;
            
        Ok(data)
    }
}
