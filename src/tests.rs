#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn init_pocketbase() {
        let pb1 = PocketBase::default("http://localhost:8091/").expect("Could not create default PocketBase instance");
        let pb2: PocketBase<DefaultAuthRecord> = PocketBase::new("http://localhost:8091/").expect("Could not create new PocketBase instance");
        assert_eq!(pb1.base_url(), "http://localhost:8091");
        assert_eq!(pb2.base_url(), "http://localhost:8091");
    }
}