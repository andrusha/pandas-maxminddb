#![allow(dead_code)]

use struct_deep_getter::StructDeepGetter;
use struct_deep_getter_derive::make_paths;

struct Hello {
    this: Vec<Something>,
    that: usize
}

struct Something {
    field: usize,
    another_one: Option<f32>
}

make_paths!(
    #[struct_deep_getter(return_type = "SuperType", replacement_type = "Hello2")]
    struct Hello {
        this: Vec<Something>,
        that: usize
    }

    struct Something {
        field: usize,
        another_one: Option<f32>
    }
);

#[test]
fn test_paths() {
    assert_eq!(Hello::deeper_structs(), vec!["lol"])
}