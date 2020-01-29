use rand::prelude::*;
use std::cmp::Ordering;
use std::i64::MAX;
use rayon::prelude::*;

use crate::genome::Genome;
use crate::selection::tournament_selection;
use crate::evolution::{crossover, mutate};
use crate::world::{Depot, Customer};

// General constants
const POP_SIZE: usize = 10_000; // Make sure POP_SIZE and ELITES have the same parity
const GENERATIONS: i64 = 2_000;
const CHILDREN: usize = 4;

pub fn train(input: String) -> (String, String) {
    let mut rng = thread_rng();
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
            let (child1, child2) = crossover(&gene_pool[p1], &gene_pool[p2], &depots, &customers, num_vehicles, &mut rng);
            new_generation.push(mutate(child1, &depots, &customers, &mut rng));
            new_generation.push(mutate(child2, &depots, &customers, &mut rng));
        }

        new_generation.sort_by(|a, b| match a.get_fitness().partial_cmp(&b.get_fitness()) {None => Ordering::Equal, Some(eq) => eq});
        new_generation.drain(0..(new_generation.len()-POP_SIZE));

        // To keep track of the progress
        let (best, valid, total_v, total_a, tot_penalty, best_penalty) = new_generation.iter().fold((MAX, 0, 0, 0, 0, MAX), |(mut b, v, t_v, t_a, t_p, mut b_p), gene| {
            let d = gene.total_distance();
            let p = gene.penalty();
            if p == 0 {
                if d < b {b = d;}
                return (b, v+1, t_v+d, t_a+d, t_p, b_p)
            }
            if p < b_p {b_p = p;}
            (b, v, t_v, t_a+d, t_p + p, b_p)
        });
        println!("Gen {}, Pool : {}, Valid Avg : {}, Avg : {}, Best : {}, Valid : {}, Lowest penalty : {}, Avg Penalty : {}", 
                i + 1, gene_pool.len(), if valid == 0 {0} else {total_v/valid}, total_a/POP_SIZE as i64, best, valid, best_penalty, if valid == POP_SIZE as i64 {0} else {tot_penalty/(POP_SIZE as i64 -valid)});
        bests.push_str(format!("{} ", best).as_str());
        averages.push_str(format!("{} ", if valid == 0 {0} else {total_v/valid}).as_str());

        pop = new_generation;
    }

    // Then take the best individual, and display it
    let first = pop.pop().unwrap();
    let mut best = first.clone();
    while best.penalty() != 0 && pop.len() > 0 {
        best = pop.pop().unwrap();
    }
    if best.penalty() == 0 {
        return (manage_outputs(best, &depots, &customers), format!("{}\n{}", bests, averages));
    }
    else {
        return (manage_outputs(first, &depots, &customers), format!("{}\n{}", bests, averages));
    }
}

fn manage_outputs(best: Genome, depots: &Vec<Depot>, customers: &Vec<Customer>) -> String {
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
        depots.push(Depot::init(x, y, max_duration, max_load, vehicles_per_depot));
    }

    for i in (n_depots + 1)..=(n_depots + n_customers) {
        let j = i as usize;
        let x = data[j][1];
        let y = data[j][2];
        let duration = data[j][3];
        let load = data[j][4];
        customers.push(Customer::init(x, y, duration, load));
    }
    return vehicles_per_depot
}