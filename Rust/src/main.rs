mod gen_alg;
mod file_io;
use std::time::SystemTime;

fn main() {
    let begin = SystemTime::now();

    // for i in 20..=23 {
    //     let (output, graphing) = gen_alg::train(file_io::read_to_string(format!("src/data/p{:02}", i).as_str()));
    //     file_io::write_to_file(format!("results/result_p{}.txt", i).as_str(), output.as_str()).expect("Couldn't write to file");
    //     file_io::write_to_file(format!("results/graph_data_p{}.txt", i).as_str(), graphing.as_str()).expect("Couldn't write to file");
    // }
    let num = 8;
    let (output, graphing) = gen_alg::train(file_io::read_to_string(format!("src/data/p{:02}", num).as_str()));
    file_io::write_to_file(format!("results/result_p{}.txt", num).as_str(), output.as_str()).expect("Couldn't write to file");
    file_io::write_to_file(format!("results/graph_data_p{}.txt", num).as_str(), graphing.as_str()).expect("Couldn't write to file");
    
    println!("\nTime elapsed in minutes and seconds : {}m{}s", begin.elapsed().unwrap().as_secs()/60, begin.elapsed().unwrap().as_secs()%60);
}