use rand::prelude::*;
use std::collections::HashSet;
use std::cmp::Ordering;
use rand::seq::SliceRandom;
use std::i32::MAX;

// Choose fitness or rank selection
// Make sure POP_SIZE and ELITES have the same parity
const POP_SIZE: i32 = 100;
const ELITES: i32 = 0;
const GENERATIONS: i32 = 10;
const POOL_SIZE: i32 = 20;
const NUM_MUTATIONS: f64 = 0.01;
const POINTS_CROSSOVER: i32 = 1;

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
    for _ in 0..GENERATIONS {
        let mut new_generation: Vec<Genome> = Vec::new();
        let gene_pool = fitness_selection(pop, &mut new_generation);
        for _ in 0..(POP_SIZE - ELITES)/2 {
            let p1: usize = rng.gen_range(0, gene_pool.len());
            let p2: usize = rng.gen_range(0, gene_pool.len());
            let (child1, child2) = gene_pool[p1].crossover(&gene_pool[p2], &depots, &customers);
            new_generation.push(child1.mutate(&depots, &customers));
            new_generation.push(child2.mutate(&depots, &customers));
        }
        pop = new_generation;
    }

    // Then take the best individual, and display it I guess ?
    pop.sort();
    manage_outputs(pop.pop().unwrap());
}

fn read_input(depots: &mut Vec<Depot>, customers: &mut Vec<Customer>, input: String) -> i32 {
    let data : Vec<Vec<i32>> = input.split('\n').map(|l| l.split_whitespace().map(|n| n.parse::<i32>().unwrap()).collect()).collect();
    let vehicles_per_depot = data[0][0];
    let n_customers = data[0][1];
    let n_depots = data[0][2];
    println!("veh : {}, cust : {}, deps : {}", vehicles_per_depot, n_customers, n_depots);
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

fn manage_outputs(_best: Genome) {
    // TODO
}

// Returns a selection of the old population, and puts the best individuals in the new generation if elitism is on
// Probability of being selected is based on fitness, no need to sort
fn fitness_selection(mut old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = rand::thread_rng();
    let fit_total = old_pop.iter().fold(0, |acc, g| acc + g.get_fitness());
    let mut selected: HashSet<i32> = HashSet::new();
    let mut gene_pool: Vec<Genome> = Vec::new();

    if ELITES > 0 {
        old_pop.sort();
        let l = selected.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }
    for _ in 0..(POOL_SIZE - ELITES) {
        selected.insert(rng.gen_range(0, fit_total));
    }
    let mut acc = 0;
    for elem in old_pop {
        let last_size = selected.len();
        acc = acc + elem.get_fitness();
        selected.retain(|&e| e > acc);
        if last_size - selected.len() > 0 {
            gene_pool.push(elem);
        }
        if last_size - selected.len() > 1 {
            println!("I hope this does not happen");
        } 
    }
    return gene_pool;
}

// Returns a selection of the old population, and puts the best individuals in the new generation if elitism is on
// Probability of being selected is based on rank, so we need to sort
fn _rank_selection(mut old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = rand::thread_rng();
    let rank_total = POP_SIZE*(POP_SIZE-1)/2;
    let mut selected: HashSet<i32> = HashSet::new();
    let mut gene_pool: Vec<Genome> = Vec::new();
    old_pop.sort();

    let l = selected.len();
    for i in 0..ELITES {
        new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
    }
    for _ in 0..(POOL_SIZE - ELITES) {
        selected.insert(rng.gen_range(0, rank_total));
    }
    let mut acc = 0;
    for (i, elem) in old_pop.into_iter().enumerate() {
        let last_size = selected.len();
        acc = acc + i;
        selected.retain(|&e| e > acc as i32);
        if last_size - selected.len() > 0 {
            gene_pool.push(elem);
        }
        if last_size - selected.len() > 1 {
            println!("I hope this does not happen");
        } 
    }
    return gene_pool;
}

struct Customer {
    x: i32,
    y: i32,
    duration: i32,
    demand: i32,
}

impl Customer {
    fn dist(&self, x: i32, y: i32) -> i32 {
        (x - self.x).abs() + (y - self.y).abs()
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
    fn dist(&self, x: i32, y: i32) -> i32 {
        (x - self.x).abs() + (y - self.y).abs()
    }
}


#[derive(Clone)]
struct Genome {
    customer_order: Vec<i32>,
    fitness: i32,
}

 impl Genome {
    pub fn get_fitness(&self) -> i32 {
        self.fitness
    }

    // Just shuffles the customers and insert the right amount of zeros anywhere
    fn random(n_customers: usize, total_vehicles: usize, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let mut rng = thread_rng();
        let mut customer_list: Vec<i32> = (1..=n_customers).map(|n| n as i32).collect();
        customer_list.shuffle(&mut rng);
        for _ in 0..total_vehicles {
            customer_list.insert(rng.gen_range(0, customer_list.len()), 0);
        }
        Self::generate(customer_list, depots, customers)
    }

    fn generate(customer_order: Vec<i32>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        let fit = Self::fitness(&customer_order, depots, customers);
        Genome{customer_order, fitness: fit}
    }

    fn fitness(customer_order: &Vec<i32>, depots: &Vec<Depot>, customers: &Vec<Customer>) -> i32 {
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
                    return MAX
                }
                if load > depots[depot].max_load {
                    return MAX
                }
                total_distance = total_distance + depots[depot].dist(x, y);
                // Initialize new vehicle
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
                        total_distance = cust.dist(x, y);
                        x = cust.x;
                        y = cust.y;
                    }
                }
            }
        }
        total_distance
    }

    fn mutate(mut self, depots: &Vec<Depot>, customers: &Vec<Customer>) -> Genome {
        // TODO
        // Just swap two things (customers or zeros) randomly
        // Go in the array and for each thing, a/size chances to be swapped with 
        // a uniformly chosen other stuff, where a is the expected number of swaps
        // we want
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

    pub fn crossover(&self, parent2: &Genome, depots: &Vec<Depot>, customers: &Vec<Customer>) -> (Genome, Genome) {
        // k-point crossover : repeat the process with the children but with 
        // further point ??
        let mut rng = thread_rng();
        let mut point: usize = rng.gen_range(0, self.customer_order.len());
        let mut last_point = point;

        let mut p1 = self.customer_order.clone();
        let mut p2 = parent2.customer_order.clone();
        let mut halfp1 = Vec::new();
        let mut halfp2 = Vec::new();
        for _ in 0..POINTS_CROSSOVER {
            point = rng.gen_range(last_point, self.customer_order.len());
            last_point = point;
            halfp1 = p1.drain(0..point).collect();
            halfp2 = p2.drain(0..point).collect();
            for el in &self.customer_order {
                if !halfp2.contains(el) {
                    halfp2.push(*el);
                }
            }
            for el in &parent2.customer_order {
                if !halfp1.contains(el) {
                    halfp1.push(*el);
                }
            }
            p1 = halfp1.clone();
            p2 = halfp2.clone();
    }
        (Self::generate(halfp1, depots, customers), Self::generate(halfp2, depots, customers))
    }
}

impl Ord for Genome {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fitness.cmp(&other.fitness)
    }
}

impl PartialOrd for Genome {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Genome {}
impl PartialEq for Genome {
    fn eq(&self, other: &Self) -> bool {
        self.fitness == other.fitness
    }
}