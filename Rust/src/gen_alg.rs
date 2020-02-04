use rand::prelude::*;
use std::cmp::Ordering;
use std::i64::MAX;
use rayon::prelude::*;
use rayon::iter::once;
use std::collections::HashSet;

use crate::genome::Genome;
use crate::selection::tournament_selection;
use crate::evolution::{crossover, mutate};
use crate::world::{Depot, Customer};


// General constants
const POP_SIZE: usize = 50;
const GENERATIONS: i64 = 50_000;
const CHILDREN: usize = 7;
const CONVERGENCE_TIME: i64 = 10_000;

pub fn train(input: String) -> (String, String) {
    let mut repeat_count = 0;
    let mut last_best = 0;
    let mut last_pest_penalty = 0;

    let mut rng = thread_rng();
    let mut penalties = String::new();
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
    let mut i = 0;
    while i < GENERATIONS && (last_pest_penalty > 0 || repeat_count < CONVERGENCE_TIME) { // 
        let mut new_generation: Vec<Genome> = Vec::new();
        let gene_pool = tournament_selection(pop, &mut new_generation);

        let random_numbers: Vec<(usize, usize)> = (0..CHILDREN*POP_SIZE/2).into_iter().map(|_| (rng.gen_range(0, gene_pool.len()), rng.gen_range(0, gene_pool.len()))).collect();
        let new_people: HashSet<Genome> = random_numbers.into_par_iter().flat_map(|(p1, p2)| {
            let (child1, child2) = crossover(&gene_pool[p1], &gene_pool[p2], &depots, &customers, num_vehicles);
            let child1 = mutate(child1, &depots, &customers);
            let child2 = mutate(child2, &depots, &customers);
            once(child1).chain(once(child2))
        }).collect();

        for new_p in new_people {
            new_generation.push(new_p);
        }
        new_generation.sort_by(|a, b| match a.get_fitness().partial_cmp(&b.get_fitness()) {None => Ordering::Equal, Some(eq) => eq});
        new_generation.drain(0..(new_generation.len()-POP_SIZE));
        // for bla in &new_generation {
        //     println!("{}0, bla.total_distance());
        // }

        // To keep track of the progress
        let (best, valid, total_a, mut best_penalty, worst_penalty, _) = new_generation.iter().fold((MAX, 0, 0, MAX, 0, 0.0), |(mut b, v, t_a, mut b_p, mut w_p, mut b_f), gene| {
            let d = gene.total_distance();
            let f = gene.get_fitness();
            let p = gene.penalty();
            if f > b_f {b = d; b_f = f;}
            if p < b_p {b_p = p;}
            if p > w_p {w_p = p;};
            (b, if p == 0 {v+1} else {v}, t_a+d, b_p, w_p, b_f)
        });
        if best_penalty == MAX {best_penalty = 0;};
        println!("Gen {}, Individuals: {}, Avg : {}, Best : {}, Valid : {}, Lowest penalty : {}, Highest Penalty : {}", 
                i + 1, new_generation.len(), total_a/new_generation.len() as i64, best, valid, best_penalty, worst_penalty);
        bests.push_str(format!("{} ", best).as_str());
        penalties.push_str(format!("{} ", best_penalty).as_str());

        pop = new_generation;
        if best == last_best && best_penalty == last_pest_penalty {
            repeat_count = repeat_count + 1;
        }
        else {
            repeat_count = 0;
            last_best = best;
            last_pest_penalty = best_penalty;
        }

        i = i + 1;
    }

    // Then take the best individual, and display it
    let first = pop.pop().unwrap();
    let mut best = first.clone();
    while best.penalty() != 0 && pop.len() > 0 {
        best = pop.pop().unwrap();
    }
    if best.penalty() == 0 {
        return (manage_outputs(best, &depots, &customers), format!("{}\n{}", bests, penalties));
    }
    else {
        return (manage_outputs(first, &depots, &customers), format!("{}\n{}", bests, penalties));
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
        let max_duration = if data[i as usize][0] == 0 {200} else {data[i as usize][0]};// data[i as usize][0]; //
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