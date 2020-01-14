mod gen_alg;
mod reader;

fn main() {
    gen_alg::do_stuff(reader::read_to_string("src/data/p01"));
}