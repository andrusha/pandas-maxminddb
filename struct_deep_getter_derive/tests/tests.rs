#![allow(dead_code)]

use struct_deep_getter_derive::make_paths;

make_paths!(
    #[struct_deep_getter(return_type = "SuperType", replacement_type = "Hello2")]
    struct Hello {
        this: Something
    }

    struct Something {
        field: usize
    }
);

#[test]
fn test_paths() {
    assert_eq!(structs(), "lol")
}