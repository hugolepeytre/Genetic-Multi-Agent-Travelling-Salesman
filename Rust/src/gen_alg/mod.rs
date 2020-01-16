use rand::prelude::*;
use std::collections::HashSet;
use std::cmp::Ordering;
use rand::seq::SliceRandom;
use std::i32::MAX;

// Choose fitness or rank selection
// Make sure POP_SIZE and ELITES have the same parity
// Best found for now (problem 1) : 10000, 1000, 10000, 1000, 0.015, 1
const POP_SIZE: usize = 20000;
const ELITES: i32 = 1000;
const GENERATIONS: i32 = 1000;
const POOL_SIZE: i32 = 1000;
const NUM_MUTATIONS: f64 = 0.08;
const CHILDREN: usize = 4;

pub fn train(input: String) {
    let mut rng = rand::thread_rng();

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
        let gene_pool = fitness_selection(pop, &mut new_generation);
        for _ in 0..CHILDREN*POP_SIZE/2 {
            let p1: usize = rng.gen_range(0, gene_pool.len());
            let p2: usize = rng.gen_range(0, gene_pool.len());
            let (child1, child2) = gene_pool[p1].one_point_crossover(&gene_pool[p2], &depots, &customers, num_vehicles);
            new_generation.push(child1.mutate(&depots, &customers));
            new_generation.push(child2.mutate(&depots, &customers));
        }

        new_generation.sort_by(|a, b| match a.fitness.partial_cmp(&b.fitness) {None => Ordering::Equal, Some(eq) => eq});
        new_generation.drain(0..(new_generation.len()-POP_SIZE));
        // To keep track of the progress
        let (best, valid, total) = new_generation.iter().fold((MAX, 0, 0), |(mut b, v, t), gene| {
            match gene.total_distance {
                Some(d) => {
                    if d < b {b = d;}
                    (b, v+1, t+d)
                    },
                _ => (b, v, t),
            }
        });
        println!("Gen {}, Pool : {}, Avg : {}, Best : {}, Valid : {}", i + 1, gene_pool.len(), total/valid, best, valid);

        pop = new_generation;
    }

    // Then take the best individual, and display it
    manage_outputs(pop.pop().unwrap(), &depots, &customers);
}

fn read_input(depots: &mut Vec<Depot>, customers: &mut Vec<Customer>, input: String) -> i32 {
    let data : Vec<Vec<i32>> = input.split('\n').map(|l| l.split_whitespace().map(|n| n.parse::<i32>().unwrap()).collect()).collect();
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

fn manage_outputs(best: Genome, depots: &Vec<Depot>, customers: &Vec<Customer>) {
    // TODO : Graphic output
    println!("Check : {}", best.total_distance.unwrap());
    match Genome::output_result(&best.customer_order, depots, customers) {
        (_, None) => println!("Gros rip"),
        (s, Some(d)) => {print!("{}\n{}", d, s)},
    }
}

// Returns a selection of the old population, and puts the best individuals in the new generation if elitism is on
// Probability of being selected is based on fitness, no need to sort
fn fitness_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
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

// Returns a selection of the old population, and puts the best individuals in the new generation if elitism is on
// Probability of being selected is based on rank, so we need to sort
fn _rank_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = rand::thread_rng();
    let rank_total = POP_SIZE*(POP_SIZE-1)/2;
    let mut selected: HashSet<i32> = HashSet::new();
    let mut gene_pool: Vec<Genome> = Vec::new();

    let l = old_pop.len();
    for i in 0..ELITES {
        new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
    }
    for _ in 0..POOL_SIZE {
        selected.insert(rng.gen_range(0, rank_total as i32));
    }
    let mut acc = 0;
    for (i, elem) in old_pop.into_iter().enumerate() {
        let last_size = selected.len();
        acc = acc + i;
        selected.retain(|&e| e > acc as i32);
        if last_size - selected.len() > 0 {
            gene_pool.push(elem);
        }
    }
    return gene_pool;
}

impl Genome {
    // Just shuffles the customers and insert the right amount of zeros uniformly
    fn random(n_customers: usize, total_vehicles: usize, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let mut customer_list: Vec<i32> = (1..=n_customers).map(|n| n as i32).collect();
        customer_list.shuffle(&mut rng);
        let step = n_customers/total_vehicles;
        for i in (1..total_vehicles).rev() {
            customer_list.insert(step*i, 0);
        }
        Self::generate(customer_list, depots, customers)
    }

    fn generate(customer_order: Vec<i32>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        // TODO : Create new customer order applying : count load/duration. If at any moment it's
        // too much, swap current customer with the next zero. Then do the same but going down
        let tot = Self::tot_dist(&customer_order, depots, customers);
        let fit = Self::fitness(tot);
        Genome{customer_order, fitness: fit, total_distance: tot}
    }

    fn tot_dist(customer_order: &Vec<i32>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Option<i32> {
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
                    return None
                }
                if load > depots[depot].max_load {
                    return None
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
            return None
        }
        if load > depots[depot].max_load {
            return None
        }
        total_distance = total_distance + depots[depot].euclid_dist(x, y);
        Some(total_distance)
    }

    fn fitness(total_distance: Option<i32>) -> f64 {
        match total_distance {
            None => 0.0,
            Some(d) => 1.0/d as f64,
            //Some(d) => {-1.0/(1.0/d as f64).ln()},
        }
    }

    fn mutate(mut self, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let l = self.customer_order.len();
        for i in 0..l {
            let mutat: f64 = rng.gen();
            if mutat < NUM_MUTATIONS {
                let tmp = self.customer_order[i];
                let other = rng.gen_range(0, l);
                self.customer_order[i] = self.customer_order[other];
                self.customer_order[other] = tmp;
            }
        }
        Self::generate(self.customer_order, depots, customers)
    }

    pub fn one_point_crossover(&self, parent2: &Genome, depots: &Vec<Depot>, customers: &Vec<Customer>, total_vehicles: usize) -> (Genome, Genome) {
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

    fn output_result(customer_order: &Vec<i32>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (String, Option<i32>) {
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
        result_string.push_str(format!("{:<4} {:<4} {}\n", duration, load, cus_list).as_str());
        (result_string, Some(total_distance))
    }
}

struct Customer {
    x: i32,
    y: i32,
    duration: i32,
    demand: i32,
}

impl Customer {
    fn euclid_dist(&self, x: i32, y: i32) -> i32 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i32
    }
}

struct Depot {
    x: i32,
    y: i32,
    max_duration: i32,
    max_load: i32,
    vehicles: i32,
}

impl Depot {
    fn euclid_dist(&self, x: i32, y: i32) -> i32 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i32
    }
}


#[derive(Clone)]
struct Genome {
    customer_order: Vec<i32>,
    fitness: f64,
    total_distance: Option<i32>,
}