mod event_store;
mod event_version;

#[cfg(test)]
mod tests {
    use std::panic;

    fn setup() {
        println!("Some setup");
    }

    fn teardown() {
        println!("some tear down");
    }

    fn run_test<T>(test: T) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe
    {
        setup();
        let result = panic::catch_unwind(|| {
            test()
        });
        teardown();
        assert!(result.is_ok())
    }

    fn add() {
        assert_eq!(2+ 2, 4);
    }

    fn sub() {
        assert_eq!(2 - 2, 0);
    }

    #[test]
    fn it_works() {
        let tests: Vec<fn()> = vec![add, sub];
        tests.iter().for_each(|x| {
           run_test(|| x() )
        } );
    }
}
