use param_rs::{LeafRef, Tree};

struct Root<T: Tree> {
    name: &'static str,
    tree: T,
}

impl<T: Tree> Root<T> {
    pub fn print_leaves(&self) {
        for (name, leaf) in param_rs::leaf_iter(self, self.name) {
            println!(
                "{:?} => {:?} (str_len: {})",
                name.as_str(),
                leaf,
                name.as_str().len()
            );
        }
    }

    pub fn get_leaf_ref(&self, path: &str) -> Option<LeafRef<'_>> {
        let path = path.strip_prefix(self.name)?;
        param_rs::get_leaf_ref(self, path)
    }
}

impl<T: Tree> Tree for Root<T> {
    fn get_ref<'a>(&'a self, field: &str) -> Option<param_rs::EitherRef<'a>> {
        self.tree.get_ref(field)
    }

    fn get_mut<'a>(&'a mut self, field: &str) -> Option<param_rs::EitherMut<'a>> {
        self.tree.get_mut(field)
    }

    fn entries(&self) -> &'static [&'static str] {
        self.tree.entries()
    }
}

fn main() {
    #[derive(param_rs::Tree, Default)]
    struct MavlinkParams {
        data_rate: u32,
        timeout_ms: u16,
        profile: Profile,
        stream: [Stream; 10],
        id: MavId,
    }

    #[derive(param_rs::Tree, Default)]
    struct MavId {
        sys: u8,
        com: u8,
    }

    #[derive(param_rs::Tree, Default)]
    struct Stream {
        id: u32,
        hz: f32,
    }

    let mav = Root {
        name: "mav",
        tree: MavlinkParams::default(),
    };

    mav.print_leaves();

    match mav.get_leaf_ref("mav.id.sys") {
        Some(LeafRef::U8(sys_id)) => println!("System id: {sys_id}"),
        _ => println!("warn: No such parameter"),
    }

    #[derive(param_rs::Tree, Default)]
    struct Parameters {
        cfg: Config,
    }

    #[derive(param_rs::Tree, Default)]
    struct Config {
        var: (u8, f32, i16),
    }

    let mut param = Parameters::default();
    // Stringy syntax
    match param_rs::get_leaf_mut(&mut param, ".cfg.var.1") {
        Some(param_rs::LeafMut::F32(var_0)) => *var_0 = 2.718,
        _ => println!("warn: No such parameter"),
    }

    for (name, leaf) in param_rs::leaf_iter(&param, "param") {
        println!("{:?} = {:?}", name.as_str(), leaf);
    }
}

#[derive(Tree, Default)]
struct AxisConfig(u8);

bitflags::bitflags! {
    impl AxisConfig: u8 {
        const WRAPPING = 1 << 0;
        const SATURATE = 1 << 1;
    }
}

#[derive(Tree, Default)]
struct RcMapVar(u8);

bitflags::bitflags! {
    impl RcMapVar: u8 {
        const NONE = 0x00;
        const ACTUAL = 0x01;
        const LINEAR = 0x02;
    }
}

#[derive(Tree, Default)]
struct Profile(u8);

bitflags::bitflags! {
    impl Profile: u8 {
        const ATTITUDE = 1 << 0;
        const POSITION = 1 << 1;
        const CHANNELS = 1 << 2;
    }
}
