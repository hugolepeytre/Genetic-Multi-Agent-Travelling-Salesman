pub fn do_stuff(input : String) {
    let data : Vec<Vec<i32>> = input.split('\n').map(|l| l.split_whitespace().map(|n| n.parse::<i32>().unwrap()).collect()).collect();
    let vehicles_per_depot = data[0][0];
    let n_customers = data[0][1];
    let n_depots = data[0][2];
    println!("veh : {}, cust : {}, deps : {}", vehicles_per_depot, n_customers, n_depots);
    for i in 1..=n_depots {
        // initialize depot i
    }
    for i in (n_depots + 1)..=(n_depots + n_customers) {
        // initialize customer i - n_depots
    }
}