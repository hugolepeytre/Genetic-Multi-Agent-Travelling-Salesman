mod gen_alg;
mod reader;

fn main() {
    gen_alg::train(reader::read_to_string("src/data/p01"));
}