use base64::Engine;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response, ResponseInit, Url};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_request(request: &Request) -> js_sys::Promise;
}

fn response(body: &[u8], status: u16, content_type: &str) -> Result<Response, JsValue> {
    let options = ResponseInit::new();
    options.set_status(status);
    let mut bytes = body.to_vec();
    let response = Response::new_with_opt_u8_array_and_init(Some(&mut bytes), &options)?;
    response.headers().set("Content-Type", content_type)?;
    response.headers().set(
        "Cache-Control",
        if status == 200 {
            "public, max-age=900"
        } else {
            "no-store"
        },
    )?;
    Ok(response)
}

async fn github_get(url: &str, token: &str) -> Result<Response, JsValue> {
    let init = RequestInit::new();
    init.set_signal(Some(&web_sys::AbortSignal::timeout_with_u32(10000)));
    let request = Request::new_with_str_and_init(url, &init)?;
    request.headers().set("User-Agent", "meetzli-portfolio")?;
    request
        .headers()
        .set("Accept", "application/vnd.github+json")?;
    if !token.is_empty() {
        request
            .headers()
            .set("Authorization", &format!("Bearer {token}"))?;
    }
    JsFuture::from(fetch_request(&request)).await?.dyn_into()
}

async fn preview(url: &Url, token: &str) -> Result<Response, JsValue> {
    let Some((repo, image)) = crate::route(&url.pathname()) else {
        return response(b"Not found", 404, "text/plain");
    };
    let github = github_get(&format!("https://api.github.com/repos/{repo}"), token).await?;
    if !github.ok() {
        return response(
            b"Repository unavailable",
            if github.status() == 404 { 404 } else { 503 },
            "text/plain",
        );
    }
    let text = JsFuture::from(github.text()?)
        .await?
        .as_string()
        .ok_or_else(|| JsValue::from_str("invalid response"))?;
    let mut info: crate::Repo =
        serde_json::from_str(&text).map_err(|_| JsValue::from_str("invalid repository data"))?;
    if info.private {
        return response(b"Not found", 404, "text/plain");
    }
    if image {
        let contributors = github_get(
            &format!("https://api.github.com/repos/{repo}/contributors?per_page=1&anon=true"),
            token,
        )
        .await?;
        if !contributors.ok() {
            return response(b"Contributors unavailable", 503, "text/plain");
        }
        let link = contributors.headers().get("Link")?.unwrap_or_default();
        info.contributors = match link
            .split(',')
            .find(|part| part.contains("rel=\"last\""))
            .and_then(|part| part.rsplit("page=").next())
            .and_then(|rest| rest.split('>').next())
            .and_then(|page| page.parse().ok())
        {
            Some(count) => count,
            None => {
                let text = JsFuture::from(contributors.text()?)
                    .await?
                    .as_string()
                    .unwrap_or_default();
                serde_json::from_str::<Vec<serde_json::Value>>(&text)
                    .map_err(|_| JsValue::from_str("invalid contributors"))?
                    .len() as u64
            }
        };
        let avatar_url = Url::new(&info.owner.avatar_url)?;
        if avatar_url.hostname() != "avatars.githubusercontent.com"
            || avatar_url.protocol() != "https:"
        {
            return response(b"Avatar unavailable", 503, "text/plain");
        }
        avatar_url.set_search("?s=240&v=4");
        let avatar = github_get(&avatar_url.href(), "").await?;
        if !avatar.ok() {
            return response(b"Avatar unavailable", 503, "text/plain");
        }
        let mime = avatar
            .headers()
            .get("Content-Type")?
            .unwrap_or_else(|| "image/png".into());
        let bytes =
            js_sys::Uint8Array::new(&JsFuture::from(avatar.array_buffer()?).await?).to_vec();
        info.avatar = format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        );
        let png = crate::png(&info).map_err(|_| JsValue::from_str("render failed"))?;
        response(&png, 200, "image/png")
    } else {
        response(
            crate::html(
                &repo,
                info.description.as_deref().unwrap_or(""),
                &url.origin(),
            )
            .as_bytes(),
            200,
            "text/html; charset=utf-8",
        )
    }
}

#[wasm_bindgen]
pub async fn handle(url: String, method: String, token: String) -> Result<Response, JsValue> {
    if method != "GET" && method != "HEAD" {
        let response = response(b"Method not allowed", 405, "text/plain")?;
        response.headers().set("Allow", "GET, HEAD")?;
        return Ok(response);
    }
    let url = Url::new(&url)?;
    match preview(&url, &token).await {
        Ok(response) => Ok(response),
        Err(_) => response(
            b"Repository preview temporarily unavailable",
            503,
            "text/plain",
        ),
    }
}

#[wasm_bindgen]
pub fn image_version() -> String {
    crate::IMAGE_VERSION.into()
}
