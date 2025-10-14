pub trait Reader {
    async fn page();
    async fn tail();
}
