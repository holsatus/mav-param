use mav_param::{Value, Tree, param_iter_named, get_value, set_value};

#[test]
fn get_value_func() {
    #[derive(mav_param::Tree, Default)]
    struct Params {
        entry1: u8,
        entry2: u8,
        var: Union,
    }

    fn def() -> Inner {
        Inner {
            i1: 2,
            i2: 4,
            i3: 8,
        }
    }

    #[repr(u8)]
    #[derive(mav_param::Enum)]
    enum Union {
        #[param(default = def())]
        Var1(Inner) = 0,
        #[param(default = 5.0)]
        Var2(f32) = 1,
    }

    #[derive(mav_param::Tree, Default)]
    struct Inner {
        i1: u8,
        i2: u8,
        i3: u8,
    }

    impl Default for Union {
        fn default() -> Self {
            Union::Var1(Default::default())
        }
    }

    let mut test = Params::default();

    assert_eq!(get_value(&test, ".var.#"), Some(crate::Value::U8(0)));
    assert_eq!(get_value(&test, ".var.i1"), Some(crate::Value::U8(0)));

    // This will implicitly update the discriminant 'leaf'
    test.var = Union::Var2(5.0);

    assert_eq!(get_value(&test, ".var.#"), Some(crate::Value::U8(1)));
    assert_eq!(get_value(&test, ".var"), Some(crate::Value::F32(5.0)));

    // Setting the discriminant will set the variant to default
    assert_eq!(
        set_value(&mut test, ".var.#", crate::Value::U8(0)),
        Some(())
    );

    assert_eq!(get_value(&test, ".var.#"), Some(crate::Value::U8(0)));
    assert_eq!(get_value(&test, ".var.i1"), Some(crate::Value::U8(2)));
}

#[test]
fn basic_derive() {
    #[derive(mav_param::Tree, Default)]
    struct Params {
        entry1: u8,
        entry2: u8,
        var: Union,
    }

    #[repr(u8)]
    #[derive(mav_param::Enum)]
    enum Union {
        Var1(Inner) = 0,
        Var2(f32) = 1,
    }

    #[derive(mav_param::Tree, Default)]
    struct Inner {
        i1: u8,
        i2: u8,
        i3: u8,
    }

    impl Default for Union {
        fn default() -> Self {
            Union::Var1(Default::default())
        }
    }

    let mut test = Params::default();

    let params_vec = param_iter_named(&test, "param")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "param.entry1".to_owned(),
        "param.entry2".to_owned(),
        "param.var.#".to_owned(),
        "param.var.i1".to_owned(),
        "param.var.i2".to_owned(),
        "param.var.i3".to_owned(),
    ];

    assert_eq!(names, params_vec);

    test.var = Union::Var2(2.0);

    let params_vec = param_iter_named(&test, "param")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "param.entry1".to_owned(),
        "param.entry2".to_owned(),
        "param.var.#".to_owned(),
        "param.var".to_owned(),
    ];

    assert_eq!(names, params_vec)
}

#[test]
fn option_enum() {
    #[derive(mav_param::Tree, Default)]
    struct Params {
        entry1: u8,
        entry2: u8,
        var: Option<(f32, f32)>,
        float1: f32,
        float2: f32,
    }

    let mut test = Params::default();

    let params_vec = param_iter_named(&test, "param")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "param.entry1".to_owned(),
        "param.entry2".to_owned(),
        "param.var.#".to_owned(),
        "param.float1".to_owned(),
        "param.float2".to_owned(),
    ];

    assert_eq!(names, params_vec);

    test.var = Some((2.0, 3.0));

    let params_vec = param_iter_named(&test, "param")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "param.entry1".to_owned(),
        "param.entry2".to_owned(),
        "param.var.#".to_owned(),
        "param.var.0".to_owned(),
        "param.var.1".to_owned(),
        "param.float1".to_owned(),
        "param.float2".to_owned(),
    ];

    assert_eq!(names, params_vec)
}

#[test]
fn basic_derive_renamed() {
    #[derive(Tree, Default)]
    struct NestedParams {
        #[param(rename = "e1")]
        entry1: u8,
        #[param(rename = "e2")]
        entry2: u8,
    }

    #[derive(Tree, Default)]
    struct TestParams {
        entry1: u8,
        entry2: u8,
        nest: NestedParams,
    }

    let test = TestParams::default();

    let params_vec = param_iter_named(&test, "test")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "test.entry1".to_owned(),
        "test.entry2".to_owned(),
        "test.nest.e1".to_owned(),
        "test.nest.e2".to_owned(),
    ];

    assert_eq!(names, params_vec)
}

#[test]
fn basic_derive_conditional() {
    #[derive(Tree, Default)]
    struct NestedParams {
        entry1: u8,
        entry2: u8,
    }

    #[derive(Tree, Default)]
    struct TestParams {
        entry1: u8,
        entry2: u8,
        #[param(condition = "self.entry1 != self.entry2")]
        nest: NestedParams,
    }

    let mut test = TestParams::default();

    let params_vec = param_iter_named(&test, "test")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec!["test.entry1".to_owned(), "test.entry2".to_owned()];

    assert_eq!(names, params_vec);

    test.entry1 += 1;

    let params_vec = param_iter_named(&test, "test")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "test.entry1".to_owned(),
        "test.entry2".to_owned(),
        "test.nest.entry1".to_owned(),
        "test.nest.entry2".to_owned(),
    ];

    assert_eq!(names, params_vec)
}

#[test]
fn derive_conditional_and_rename() {
    #[derive(Tree, Default)]
    struct NestedParams {
        entry1: u8,
        entry2: u8,
    }

    #[derive(Tree, Default)]
    struct TestParams {
        entry1: u8,
        entry2: u8,
        #[param(rename = "n")]
        #[param(condition = "self.entry1 != self.entry2")]
        nest: NestedParams,
    }

    let mut test = TestParams::default();

    let params_vec = param_iter_named(&test, "test")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec!["test.entry1".to_owned(), "test.entry2".to_owned()];

    assert_eq!(names, params_vec);

    test.entry1 += 1;

    let params_vec = param_iter_named(&test, "test")
        .filter_map(|e| e.map(|e| e.ident.as_str().to_owned()).ok())
        .take(20)
        .collect::<Vec<_>>();

    let names = vec![
        "test.entry1".to_owned(),
        "test.entry2".to_owned(),
        "test.n.entry1".to_owned(),
        "test.n.entry2".to_owned(),
    ];

    assert_eq!(names, params_vec)
}