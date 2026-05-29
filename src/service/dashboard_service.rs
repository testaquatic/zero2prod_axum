use anyhow::Context;

use crate::{
    app_state::AppState,
    database::postgres::users::get_user_info_by_user_id,
    domain::{extractor::TokenData, response::UserInfoResponse},
    service::error::ServiceError,
};

pub struct DashboardService;

impl DashboardService {
    pub async fn get_admin_dashboard(
        &self,
        app_stat: &AppState,
        token_data: &TokenData,
    ) -> Result<UserInfoResponse, ServiceError> {
        get_user_info_by_user_id(&app_stat.pg_pool, &token_data.user_id)
            .await?
            .context("Cannot find ueser")
            .map_err(ServiceError::UnexpectedError)
    }
}
