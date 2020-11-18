use web_view::*;

fn main() {
    web_view::builder()
        .title("Blank Screen")
        .content(Content::Html(""))
        .size(1024, 768)
        .resizable(true)
        .debug(false)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}
