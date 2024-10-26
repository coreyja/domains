use std::convert::Infallible;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
    response::{IntoResponse, Response},
};
use maud::html;

use crate::AppState;

pub(crate) struct TemplateBuilder {
    app_state: AppState,
}

#[async_trait]
impl FromRequestParts<AppState> for TemplateBuilder {
    type Rejection = Infallible;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            app_state: state.clone(),
        })
    }
}

pub struct Template {
    html: maud::Markup,
}

impl IntoResponse for Template {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
        );

        (headers, self.html.into_string()).into_response()
    }
}

impl TemplateBuilder {
    pub fn render(&self, inner: maud::Markup) -> Template {
        let html = html! {
            html {
                head {
                    link rel="stylesheet" href="/public/styles.css" {}
              }

              body {
                  (inner)
              }
            }
        };

        Template { html }
    }
}
