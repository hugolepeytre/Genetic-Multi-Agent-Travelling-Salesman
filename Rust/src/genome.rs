use crate::world::{Depot, Customer};

use rand::prelude::*;

const REPAIRED: bool = false;
const ALPHA: f64 = 1000.0;

#[derive(Clone)]
pub struct Genome {
    pub customer_order: Vec<i64>,
    fitness: f64,
    total_distance: i64,
    penalty: i64,
}

impl Genome {
    // Fitness function and derived stuff

    fn tot_dist(customer_order: &Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (i64, i64) {
        let mut penalty = 0;
        let mut total_distance = 0;
        
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;
        let mut duration = 0;

        let mut x = depots[0].x();
        let mut y = depots[0].y();

        for &c in customer_order {
            if c == 0 {
                // Check limits
                let dist = depots[depot].dist(x, y);
                duration = duration + dist;
                total_distance = total_distance + dist;
                if depots[depot].over_load(load) {
                    let err = load - depots[depot].max_load();
                    penalty = penalty + err;
                }
                if depots[depot].over_duration(duration) {
                    let err = duration - depots[depot].max_duration();
                    penalty = penalty + err;
                }
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles() {
                    vehicle = 0;
                    depot = depot + 1;
                }
                x = depots[depot].x();
                y = depots[depot].y();
                load = 0;
                duration = 0;
            }
            else {
                match customers.get((c - 1) as usize) {
                    None => panic!("Wrong customer number : {}", c),
                    Some(cust) => {
                        let dist = cust.dist(x, y);
                        load = load + cust.load();
                        duration = duration + cust.duration() + dist;
                        total_distance = total_distance + dist;
                        x = cust.x();
                        y = cust.y();
                    }
                }
            }
        }

        // Check limits
        let dist = depots[depot].dist(x, y);
        duration = duration + dist;
        total_distance = total_distance + dist;
        if depots[depot].over_load(load) {
            let err = load - depots[depot].max_load();
            penalty = penalty + err;
        }
        if depots[depot].over_duration(duration) {
            let err = duration - depots[depot].max_duration();
            penalty = penalty + err;
        }
        (total_distance, penalty)
    }

    fn repair_load(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Vec<i64> {
        // println!("From : {}", customer_order.len());
        // for &n in &customer_order {
        //     if n == 0 {
        //         println!("");
        //     }
        //     print!("{} ", n);
        // }
        // println!("");
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;

        let mut i = 0;
        while i < customer_order.len() {
            if customer_order[i] == 0 {
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles() {
                    vehicle = 0;
                    depot = depot + 1;
                }
                load = 0;
            }
            else {
                match customers.get((customer_order[i] - 1) as usize) {
                    None => panic!("Wrong customer number : {}", customer_order[i]),
                    Some(cust) => {
                        load = load + cust.load();
                    }
                }
                if depots[depot].over_load(load) {
                    let (custoz, worked) = Self::pull_next_zero(i, customer_order);
                    customer_order = custoz;
                    if worked {
                        i = i - 1;
                    }
                }
            }
            i = i + 1;
        }
        // println!("To : {}", customer_order.len());
        // for &n in &customer_order {
        //     if n == 0 {
        //         println!("");
        //     }
        //     print!("{} ", n);
        // }
        // println!("");
        customer_order
    }

    pub fn output_result(customer_order: &Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (String, Option<i64>) {
        let mut result_string = String::new();
        let mut cus_list = String::from("0 ");
        let mut total_distance = 0;
        
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;
        let mut duration = 0;

        let mut x = depots[0].x();
        let mut y = depots[0].y();

        result_string.push_str(format!("{:<3} {:<3} ", depot+1, vehicle+1).as_str());

        for &c in customer_order {
            if c == 0 {
                // Check limits
                let dist = depots[depot].dist(x, y);
                duration = duration + dist;
                total_distance = total_distance + dist;
                if depots[depot].over_duration(duration) {
                    println!("Invalid duration");
                }
                if depots[depot].over_load(load) {
                    println!("Invalid load");
                }
                cus_list.push_str("0");
                result_string.push_str(format!("{:<4} {:<4} {}\n", duration, load, cus_list).as_str());
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles() {
                    vehicle = 0;
                    depot = depot + 1;
                }
                x = depots[depot].x();
                y = depots[depot].y();
                load = 0;
                duration = 0;
                cus_list = String::from("0 ");
                result_string.push_str(format!("{:<3} {:<3} ", depot+1, vehicle+1).as_str());
            }
            else {
                match customers.get((c - 1) as usize) {
                    None => panic!("Wrong customer number : {}", c),
                    Some(cust) => {
                        let dist = cust.dist(x, y);
                        load = load + cust.load();
                        duration = duration + dist;
                        total_distance = total_distance + dist;
                        x = cust.x();
                        y = cust.y();
                        cus_list.push_str(format!("{} ", c).as_str());
                    }
                }
            }
        }
        // Check limits
        let dist = depots[depot].dist(x, y);
        duration = duration + dist;
        total_distance = total_distance + dist;
        if depots[depot].over_duration(duration) {
            println!("Invalid duration");
        }
        if depots[depot].over_load(load) {
            println!("Invalid load");
        }
        cus_list.push_str("0");
        result_string.push_str(format!("{:<4} {:<4} {}", duration, load, cus_list).as_str());
        (result_string, Some(total_distance))
    }

    // Misc

    pub fn random(n_customers: usize, total_vehicles: usize, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let mut customer_list: Vec<i64> = (1..=n_customers).map(|n| n as i64).collect();
        customer_list.shuffle(&mut rng);
        let step = n_customers/total_vehicles;
        for i in (1..total_vehicles).rev() {
            customer_list.insert(step*i, 0);
        }
        Self::generate(customer_list, depots, customers)
    }

    pub fn fitness(total_distance: i64, penalty: i64) -> f64 {
        1.0/(total_distance as f64 + ALPHA*penalty as f64)
    }

    pub fn generate(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        if REPAIRED {
            customer_order = Self::repair_load(customer_order, depots, customers);
            customer_order.reverse();
            customer_order = Self::repair_load(customer_order, depots, customers);
            customer_order.reverse();
        }
        let (tot, penalty) = Self::tot_dist(&customer_order, depots, customers);
        let fit = Self::fitness(tot, penalty);
        Genome{customer_order, fitness: fit, total_distance: tot, penalty}
    }

    fn pull_next_zero(i: usize, mut customer_order: Vec<i64>) -> (Vec<i64>, bool) {
        match customer_order.iter().skip(i).position(|&e| e == 0) {
            None => return (customer_order, false),
            Some(a) => {
                customer_order.remove(a + i);
                customer_order.insert(i, 0);
                return (customer_order, true)
            }
        }
    }

    fn _swap_elems(mut customer_order: Vec<i64>, idx1: usize, idx2: usize) -> Vec<i64> {
        let tmp = customer_order[idx1];
        customer_order[idx1] = customer_order[idx2];
        customer_order[idx2] = tmp;
        return customer_order
    }

    pub fn get_fitness(&self) -> f64 {
        self.fitness
    }

    pub fn penalty(&self) -> i64 {
        self.penalty
    }

    pub fn total_distance(&self) -> i64 {
        self.total_distance
    }
}