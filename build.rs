
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/Bitcoder.ico");
        res.compile().unwrap_or(());
    }
}
