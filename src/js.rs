use dioxus::document::eval;

pub fn call(function: &str) {
    let _ = eval(&format!("site.{function}()"));
}

pub fn reveal(selector: &str) {
    let _ = eval(&format!("site.reveal('{selector}')"));
}
