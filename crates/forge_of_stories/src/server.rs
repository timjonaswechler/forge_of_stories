fn main() {
    #[cfg(feature = "server")]
    wizard::run().expect("Failed to run the Forge of Stories Wizard");
}
