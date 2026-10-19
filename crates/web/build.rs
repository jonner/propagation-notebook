fn main() {
    topcoat::tailwind::BuildConfig::new()
        .input("styles/tailwind.css")
        .render()
        .unwrap();
    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("mdi")
        .stage()
        .unwrap();
}
