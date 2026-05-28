use std::sync::Arc;

use axum::middleware::{self, FromExtractorLayer};

use crate::{app_state::AppState, domain::extractor::TokenData};

pub fn auth_token_middleware(state: Arc<AppState>) -> FromExtractorLayer<TokenData, Arc<AppState>> {
    middleware::from_extractor_with_state::<TokenData, Arc<AppState>>(state)
}
