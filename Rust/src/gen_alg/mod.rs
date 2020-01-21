use rand::prelude::*;
use std::collections::HashSet;
use std::cmp::Ordering;
use rand::seq::SliceRandom;
use std::i64::MAX;

// General constants
const POP_SIZE: usize = 1_000; // Make sure POP_SIZE and ELITES have the same parity
const ELITES: i64 = 500;
const GENERATIONS: i64 = 1000;
const POOL_SIZE: i64 = 500;
const CHILDREN: usize = 4;
const REPAIRED: bool = true;

// Mutation and Crossover constants
const PROB_MUTATION: f64 = 0.1;
const FRAC_SWAP: f64 = 1.0/3.0;
const FRAC_INSERT: f64 = 1.0/3.0;
const FRAC_SCRAMBLE: f64 = 1.0/3.0;

const PROB_CROSSOVER: f64 = 0.9;
const FRAC_ORDER1: f64 = 1.0/3.0;
const FRAC_PMX: f64 = 1.0/3.0;
const FRAC_EDGE_RECOMB: f64 = 1.0/3.0;

// Selection constants
const TOURNAMENT_SIZE: usize = 100; // 1 is random, higher up to pop.len() is higher pressure
const SELECTION_PRESSURE: f64 = 0.7; // Higher = closer to deterministic, should be between 0 and 1

pub fn train(input: String) -> (String, String) {
    let mut rng = rand::thread_rng();
    let mut averages = String::new();
    let mut bests = String::new();

    let mut depots: Vec<Depot> = Vec::new();
    let mut customers: Vec<Customer> = Vec::new();
    let vehicles_per_depot = read_input(&mut depots, &mut customers, input);
    let num_vehicles = vehicles_per_depot as usize * depots.len();

    // Generate population
    let mut pop: Vec<Genome> = Vec::new();
    for _ in 0..POP_SIZE {
        pop.push(Genome::random(customers.len(), num_vehicles, &depots, &customers));
    }

    // For each generation, do the stuff
    for i in 0..GENERATIONS {
        let mut new_generation: Vec<Genome> = Vec::new();
        let gene_pool = tournament_selection(pop, &mut new_generation);
        for _ in 0..CHILDREN*POP_SIZE/2 {
            let p1: usize = rng.gen_range(0, gene_pool.len());
            let p2: usize = rng.gen_range(0, gene_pool.len());
            let cross: f64 = rng.gen();
            if cross < PROB_CROSSOVER {
                let (child1, child2) = gene_pool[p1].cut_crossfill_crossover(&gene_pool[p2], &depots, &customers, num_vehicles);
                new_generation.push(child1.mutate(&depots, &customers));
                new_generation.push(child2.mutate(&depots, &customers));
            }
            else {
                new_generation.push(gene_pool[p1].clone());
                new_generation.push(gene_pool[p2].clone());
            }
        }

        new_generation.sort_by(|a, b| match a.fitness.partial_cmp(&b.fitness) {None => Ordering::Equal, Some(eq) => eq});
        new_generation.drain(0..(new_generation.len()-POP_SIZE));

        // To keep track of the progress
        let (best, valid, total_v, total_a) = new_generation.iter().fold((MAX, 0, 0, 0), |(mut b, v, t_v, t_a), gene| {
            let d = gene.total_distance;
            if gene.valid {
                if d < b {b = d;}
                return (b, v+1, t_v+d, t_a+d)
            }
            (b, v, t_v, t_a+d)
        });
        if total_a < 0 {
            println!("wtf");
        }
        println!("Gen {}, Pool : {}, Valid Avg : {}, Avg : {}, Best : {}, Valid : {}", 
                    i + 1, gene_pool.len(), total_v/valid, total_a/POP_SIZE as i64, best, valid);
        bests.push_str(format!("{} ", best).as_str());
        averages.push_str(format!("{} ", total_v/valid).as_str());

        pop = new_generation;
    }

    // Then take the best individual, and display it
    return (manage_outputs(pop.pop().unwrap(), &depots, &customers), format!("{}\n{}", bests, averages));
}

fn manage_outputs(best: Genome, depots: &Vec<Depot>, customers: &Vec<Customer>) -> String {
    println!("Check : {}", best.total_distance);
    let mut output = String::new();
    match Genome::output_result(&best.customer_order, depots, customers) {
        (_, None) => println!("Gros rip"),
        (s, Some(d)) => {
            print!("{}\n{}", d, s);
            output = format!("{}\n{}", d, s);
        },
    }
    return output
}

// Selection methods
fn _fitness_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = rand::thread_rng();
    let fit_total = old_pop.iter().fold(0.0, |acc, g| acc + g.fitness);
    let mut selected: Vec<f64> = Vec::new();
    let mut gene_pool: Vec<Genome> = Vec::new();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }
    for _ in 0..POOL_SIZE {
        let r: f64 = rng.gen();
        selected.push(r*fit_total);
    }
    let mut acc = 0.0;
    for elem in old_pop {
        let last_size = selected.len();
        acc = acc + elem.fitness;
        selected.retain(|&e| e > acc);
        if last_size - selected.len() > 0 {
            gene_pool.push(elem);
        }
    }
    return gene_pool;
}

fn tournament_selection(mut old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut pool = Vec::new();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }

    for _ in 0..POOL_SIZE {
        let mut rng = rand::thread_rng();
        
        let mut participants: Vec<usize> = Vec::new();
        for _ in 0..TOURNAMENT_SIZE {
            participants.push(rng.gen_range(0, old_pop.len()));
        }
        participants.sort(); // Assumption : old_pop is sorted
        
        let winner: f64 = rng.gen();
        let mut acc = 0.0;
        let mut k = 0;
        while k < TOURNAMENT_SIZE && acc < winner {
            acc = acc + SELECTION_PRESSURE*(1.0 - SELECTION_PRESSURE).powi(k as i32);
            k = k + 1;
        }
        k = TOURNAMENT_SIZE - k;

        pool.push(old_pop.remove(participants[k]));
    }
    return pool
}

fn _rank_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = rand::thread_rng();
    let rank_total = POP_SIZE*(POP_SIZE-1)/2;
    let mut selected: HashSet<i64> = HashSet::new();
    let mut gene_pool: Vec<Genome> = Vec::new();

    let l = old_pop.len();
    for i in 0..ELITES {
        new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
    }
    for _ in 0..POOL_SIZE {
        selected.insert(rng.gen_range(0, rank_total as i64));
    }
    let mut acc = 0;
    for (i, elem) in old_pop.into_iter().enumerate() {
        let last_size = selected.len();
        acc = acc + i;
        selected.retain(|&e| e > acc as i64);
        if last_size - selected.len() > 0 {
            gene_pool.push(elem);
        }
    }
    return gene_pool;
}

impl Genome {
    // Mutation functions 

    fn mutate(mut self, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let l = self.customer_order.len();
        // Insert
        // Swap
        // Scramble
        let mutat: f64 = rng.gen();
        if mutat < PROB_MUTATION {
            if mutat < FRAC_INSERT {
                // Pick src, dst, move src to dest without deleting anything else
            }
            else if mutat < FRAC_INSERT + FRAC_SWAP {
                // Pick a src, dst, exchange order[src] and order[dst]
            }
            else if mutat < FRAC_INSERT + FRAC_SWAP + FRAC_SCRAMBLE {
                // Pick len then src
            }
        }
        for i in 0..l {
            if mutat < PROB_MUTATION {
                let tmp = self.customer_order[i];
                let other = rng.gen_range(0, l);
                self.customer_order[i] = self.customer_order[other];
                self.customer_order[other] = tmp;
            }
        }
        Self::generate(self.customer_order, depots, customers)
    }

    // Crossover functions 

    pub fn cut_crossfill_crossover(&self, parent2: &Genome, depots: &Vec<Depot>, customers: &Vec<Customer>, total_vehicles: usize) -> (Genome, Genome) {
        let mut rng = thread_rng();

        let mut halfp1 = Vec::new();
        let mut halfp2 = Vec::new();

        let point = rng.gen_range(0, self.customer_order.len());
        let mut zero_count1 = 0;
        let mut zero_count2 = 0;
        
        for &n in self.customer_order.iter().take(point) {
            if n == 0 {
                zero_count1 = zero_count1 + 1;
            }
            halfp1.push(n);
        }
        for &n in parent2.customer_order.iter().skip(point).chain(parent2.customer_order.iter().take(point)) {
            if n == 0 && zero_count1 < total_vehicles - 1 {
                zero_count1 = zero_count1 + 1;
                halfp1.push(n);
            }
            else if !halfp1.contains(&n) {
                halfp1.push(n);
            }
        }
        for &n in parent2.customer_order.iter().take(point) {
            if n == 0 {
                zero_count2 = zero_count2 + 1;
            }
            halfp2.push(n);
        }
        for &n in self.customer_order.iter().skip(point).chain(self.customer_order.iter().take(point)) {
            if n == 0 && zero_count2 < total_vehicles - 1 {
                zero_count2 = zero_count2 + 1;
                halfp2.push(n);
            }
            else if !halfp2.contains(&n) {
                halfp2.push(n);
            }
        }
        (Self::generate(halfp1, depots, customers), Self::generate(halfp2, depots, customers))
    }


    // Fitness function and derived stuff

    fn tot_dist(customer_order: &Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (i64, bool) {
        let mut valid = true;
        let mut total_distance = 0;
        
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;
        let mut duration = 0;

        let mut x = depots[0].x;
        let mut y = depots[0].y;

        for &c in customer_order {
            if c == 0 {
                // Check limits
                if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
                    total_distance = total_distance * depots.len() as i64;
                    valid = false;
                }
                if load > depots[depot].max_load {
                    total_distance = total_distance * depots.len() as i64;
                    valid = false;
                }
                total_distance = total_distance + depots[depot].euclid_dist(x, y);
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles {
                    vehicle = 0;
                    depot = depot + 1;
                }
                x = depots[depot].x;
                y = depots[depot].y;
                load = 0;
                duration = 0;
            }
            else {
                match customers.get((c - 1) as usize) {
                    None => panic!("Wrong customer number : {}", c),
                    Some(cust) => {
                        load = load + cust.demand;
                        duration = duration + cust.duration;
                        total_distance = total_distance + cust.euclid_dist(x, y);
                        x = cust.x;
                        y = cust.y;
                    }
                }
            }
        }

        // Check limits
        if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
            total_distance = total_distance * depots.len() as i64;
            valid = false;
        }
        if load > depots[depot].max_load {
            total_distance = total_distance * depots.len() as i64;
            valid = false;
        }
        total_distance = total_distance + depots[depot].euclid_dist(x, y);
        (total_distance, valid)
    }

    fn repair(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Vec<i64> {
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;
        let mut duration = 0;

        for i in 0..customer_order.len() {
            if customer_order[i] == 0 {
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles {
                    vehicle = 0;
                    depot = depot + 1;
                }
                load = 0;
                duration = 0;
            }
            else {
                match customers.get((customer_order[i] - 1) as usize) {
                    None => panic!("Wrong customer number : {}", customer_order[i]),
                    Some(cust) => {
                        load = load + cust.demand;
                        duration = duration + cust.duration;
                    }
                }
            }
            // Check limits
            if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
                customer_order = Self::swap_next_zero(i, customer_order);
                load = 0;
                duration = 0;
            }
            if load > depots[depot].max_load {
                customer_order = Self::swap_next_zero(i, customer_order);
                load = 0;
                duration = 0;
            }
        }
        customer_order
    }

    fn output_result(customer_order: &Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (String, Option<i64>) {
        let mut result_string = String::new();
        let mut cus_list = String::from("0 ");
        let mut total_distance = 0;
        
        let mut depot = 0;
        let mut vehicle = 0;

        let mut load = 0;
        let mut duration = 0;

        let mut x = depots[0].x;
        let mut y = depots[0].y;

        result_string.push_str(format!("{:<3} {:<3} ", depot+1, vehicle+1).as_str());

        for &c in customer_order {
            if c == 0 {
                // Check limits
                if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
                    return (String::from("Invalid duration"), None)
                }
                if load > depots[depot].max_load {
                    return (String::from("Invalid load"), None)
                }
                total_distance = total_distance + depots[depot].euclid_dist(x, y);
                cus_list.push_str("0");
                result_string.push_str(format!("{:<4} {:<4} {}\n", duration, load, cus_list).as_str());
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles {
                    vehicle = 0;
                    depot = depot + 1;
                }
                x = depots[depot].x;
                y = depots[depot].y;
                load = 0;
                duration = 0;
                cus_list = String::from("0 ");
                result_string.push_str(format!("{:<3} {:<3} ", depot+1, vehicle+1).as_str());
            }
            else {
                match customers.get((c - 1) as usize) {
                    None => panic!("Wrong customer number : {}", c),
                    Some(cust) => {
                        load = load + cust.demand;
                        duration = duration + cust.duration;
                        total_distance = total_distance + cust.euclid_dist(x, y);
                        x = cust.x;
                        y = cust.y;
                        cus_list.push_str(format!("{} ", c).as_str());
                    }
                }
            }
        }
        // Check limits
        if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
            return (String::from("Invalid duration"), None)
        }
        if load > depots[depot].max_load {
            return (String::from("Invalid load"), None)
        }
        total_distance = total_distance + depots[depot].euclid_dist(x, y);
        cus_list.push_str("0");
        result_string.push_str(format!("{:<4} {:<4} {}", duration, load, cus_list).as_str());
        (result_string, Some(total_distance))
    }

    // Misc

    fn random(n_customers: usize, total_vehicles: usize, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let mut customer_list: Vec<i64> = (1..=n_customers).map(|n| n as i64).collect();
        customer_list.shuffle(&mut rng);
        let step = n_customers/total_vehicles;
        for i in (1..total_vehicles).rev() {
            customer_list.insert(step*i, 0);
        }
        Self::generate(customer_list, depots, customers)
    }

    fn fitness(total_distance: i64) -> f64 {
        1.0/total_distance as f64
    }

    fn generate(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
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

    fn swap_next_zero(i: usize, mut customer_order: Vec<i64>) -> Vec<i64> {
        match customer_order.iter().skip(i).position(|&e| e == 0) {
            None => return customer_order,
            Some(a) => {
                customer_order[a + i] = customer_order[i];
                customer_order[i] = 0;
                return customer_order
            }
        }
    }
}

struct Customer {
    x: i64,
    y: i64,
    duration: i64,
    demand: i64,
}

impl Customer {
    fn euclid_dist(&self, x: i64, y: i64) -> i64 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i64
    }
}

struct Depot {
    x: i64,
    y: i64,
    max_duration: i64,
    max_load: i64,
    vehicles: i64,
}

impl Depot {
    fn euclid_dist(&self, x: i64, y: i64) -> i64 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i64
    }
}


#[derive(Clone)]
struct Genome {
    customer_order: Vec<i64>,
    fitness: f64,
    total_distance: i64,
    valid: bool,
}

fn read_input(depots: &mut Vec<Depot>, customers: &mut Vec<Customer>, input: String) -> i64 {
    let data : Vec<Vec<i64>> = input.split('\n').map(|l| l.split_whitespace().map(|n| n.parse::<i64>().unwrap()).collect()).collect();
    let vehicles_per_depot = data[0][0];
    let n_customers = data[0][1];
    let n_depots = data[0][2];

    for i in 1..=n_depots {
        let max_duration = data[i as usize][0];
        let max_load = data[i as usize][1];
        let idx = (i + n_depots + n_customers) as usize;
        let x = data[idx][1];
        let y = data[idx][2];
        depots.push(Depot{x, y, max_duration, max_load, vehicles: vehicles_per_depot});
    }

    for i in (n_depots + 1)..=(n_depots + n_customers) {
        let j = i as usize;
        let x = data[j][1];
        let y = data[j][2];
        let duration = data[j][3];
        let demand = data[j][4];
        customers.push(Customer{x, y, duration, demand});
    }
    return vehicles_per_depot
}