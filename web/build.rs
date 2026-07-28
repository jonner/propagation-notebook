fn main() {
    topcoat::tailwind::BuildConfig::new()
        .input("styles/tailwind.css")
        .render()
        .unwrap();
}
