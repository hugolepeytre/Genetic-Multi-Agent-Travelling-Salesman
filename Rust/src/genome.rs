use crate::world::{Depot, Customer};

use rand::prelude::*;
use std::i64::MAX;

const REPAIRED: bool = true;

#[derive(Clone)]
pub struct Genome {
    pub customer_order: Vec<i64>,
    fitness: f64,
    total_distance: i64,
    valid: bool,
}

impl Genome {
    // Fitness function and derived stuff

    fn tot_dist(customer_order: &Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (i64, bool) {
        let mut valid = true;
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
                if depots[depot].over_limits(load, duration) {
                    total_distance = total_distance * depots.len() as i64;
                    valid = false;
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
        if depots[depot].over_limits(load, duration) {
            total_distance = total_distance * depots.len() as i64;
            valid = false;
        }
        (total_distance, valid)
    }

    fn repair(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Vec<i64> {
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
        let mut duration = 0;

        let mut x = depots[0].x();
        let mut y = depots[0].y();

        let mut i = 0;
        while i < customer_order.len() {
            if customer_order[i] == 0 {
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
                let prev_load = load;
                let prev_dur = duration;
                let prev_x = x;
                let prev_y = y;
                match customers.get((customer_order[i] - 1) as usize) {
                    None => panic!("Wrong customer number : {}", customer_order[i]),
                    Some(cust) => {
                        let dist = cust.dist(x, y);
                        load = load + cust.load();
                        duration = duration + cust.duration() + dist;
                        x = cust.x();
                        y = cust.y();
                    }
                }
                if depots[depot].over_duration(duration + depots[depot].dist(x, y)) {
                    let (_, best_idx, _) = customer_order.iter().skip(i).fold((i, i, MAX), |(curr_idx, idx, min_dist), &x| {
                        if x == 0 {
                            (curr_idx + 1, idx, min_dist)
                        }
                        else {
                            let cust = customers.get(x as usize - 1).unwrap();
                            let distance = cust.dist(prev_x, prev_y) + cust.dist_dep(&depots[depot]);
                            if distance < min_dist {
                                (curr_idx + 1, curr_idx, distance)
                            }
                            else {
                                (curr_idx + 1, idx, min_dist)
                            }
                        }
                    });
                    let cust = customers.get(customer_order[best_idx] as usize - 1).unwrap();
                    let new_dur = prev_dur + cust.duration() + cust.dist(prev_x, prev_y);
                    let new_load = prev_load + cust.load();
                    if !depots[depot].over_limits(new_load, new_dur + depots[depot].dist_cust(&cust)) {
                        customer_order = Self::swap_elems(customer_order, i, best_idx);
                        load = prev_load;
                        duration = prev_dur;
                        x = prev_x;
                        y = prev_y;
                        i = i - 1;
                    }
                    else {
                        let (custoz, worked) = Self::pull_next_zero(i, customer_order);
                        customer_order = custoz;
                        if worked {
                            i = i - 1;
                        }
                    }
                }
                else if depots[depot].over_load(load) {
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
                    return (String::from("Invalid duration"), None)
                }
                if depots[depot].over_load(load) {
                    return (String::from("Invalid load"), None)
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
            return (String::from("Invalid duration"), None)
        }
        if depots[depot].over_load(load) {
            return (String::from("Invalid load"), None)
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

    pub fn fitness(total_distance: i64) -> f64 {
        if total_distance == 0 {
            0.0
        }
        else {
            1.0/total_distance as f64
        }
    }

    pub fn generate(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        if REPAIRED {
            customer_order = Self::repair(customer_order, depots, customers);
            customer_order.reverse();
            customer_order = Self::repair(customer_order, depots, customers);
            customer_order.reverse();
        }
        let (tot, valid) = Self::tot_dist(&customer_order, depots, customers);
        let fit = Self::fitness(tot);
        Genome{customer_order, fitness: fit, total_distance: tot, valid}
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

    fn swap_elems(mut customer_order: Vec<i64>, idx1: usize, idx2: usize) -> Vec<i64> {
        let tmp = customer_order[idx1];
        customer_order[idx1] = customer_order[idx2];
        customer_order[idx2] = tmp;
        return customer_order
    }

    pub fn get_fitness(&self) -> f64 {
        self.fitness
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn total_distance(&self) -> i64 {
        self.total_distance
    }
}