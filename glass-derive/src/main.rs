use glass_derive::*;

#[derive(Indexable)]
pub struct Test {
    pub name: Option<String>,
    pub author: Option<String>,
    pub img_url: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

fn main() {
    println!("{:?}", Test::fields());
}
