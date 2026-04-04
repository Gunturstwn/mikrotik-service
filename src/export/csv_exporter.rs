use crate::dto::user::UserProfileResponse;
use crate::errors::app_error::AppError;
use csv::Writer;

pub struct CSVExporter;

impl CSVExporter {
    pub fn export_users(users: Vec<UserProfileResponse>) -> Result<Vec<u8>, AppError> {
        let mut wtr = Writer::from_writer(vec![]);
        
        wtr.write_record(&["ID", "Name", "Email", "Phone", "Verified"])
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        for user in users {
            wtr.write_record(&[
                user.id.to_string(),
                user.name,
                user.email,
                user.phone.unwrap_or_default(),
                user.is_verified.to_string(),
            ]).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        }

        let data = wtr.into_inner()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(data)
    }
}
