mod gen_alg;
mod file_io;

fn main() {
    for i in 1..=23 {
        let (output, graphing) = gen_alg::train(file_io::read_to_string(format!("src/data/p{:02}", i).as_str()));
        file_io::write_to_file(format!("result_p{}.txt", i).as_str(), output.as_str()).expect("Couldn't write to file");
        file_io::write_to_file(format!("graph_data_p{}.txt", i).as_str(), graphing.as_str()).expect("Couldn't write to file");
    }
    // let num = 1;
    // let (output, graphing) = gen_alg::train(file_io::read_to_string(format!("src/data/p{:02}", num).as_str()));
    // file_io::write_to_file(format!("result_p{}.txt", num).as_str(), output.as_str()).expect("Couldn't write to file");
    // file_io::write_to_file(format!("graph_data_p{}.txt", num).as_str(), graphing.as_str()).expect("Couldn't write to file");
}