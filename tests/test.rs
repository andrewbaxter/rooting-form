#[derive(rooting_form::Form)]
pub struct Alpha {
    #[title("A")]
    pub a: i32,
}

#[derive(rooting_form::Form)]
pub enum Beta {
    #[title("A")]
    A,
    #[title("B")]
    B(i32),
    #[title("C")]
    C {
        #[title("Something")]
        nix: i32,
    },
}
