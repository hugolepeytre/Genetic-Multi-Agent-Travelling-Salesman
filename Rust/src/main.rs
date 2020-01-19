mod gen_alg;
mod file_io;

fn main() {
    for i in 1..=23 {
        let output = gen_alg::train(file_io::read_to_string(format!("src/data/p{:02}", i).as_str()));
        file_io::write_to_file(format!("result_p{}.txt", i).as_str(), output.as_str()).expect("Couldn't write to file");
    }
}