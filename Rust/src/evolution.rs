use std::collections::BTreeSet;

use crate::genome::Genome;
use crate::world::{Depot, Customer};

use rand::prelude::*;
use std::i64::MAX;

const PROB_MUTATION: f64 = 0.4;
const FRAC_INSERT: f64 = 0.7;
const FRAC_SWAP: f64 = 0.1;
const FRAC_SCRAMBLE: f64 = 0.2;

const PROB_CROSSOVER: f64 = 0.9;
const FRAC_ORDER1: f64 = 0.0;
const FRAC_PMX: f64 = 0.9;
const FRAC_EDGE_RECOMB: f64 = 0.0;

pub fn mutate(mut old: Genome, depots: &Vec<Depot>, customers: &Vec<Customer>, rng: &mut ThreadRng) -> Genome {
    let l = old.customer_order.len();
    let mutat: f64 = rng.gen();

    if mutat < PROB_MUTATION {
        let mutat: f64 = rng.gen();
        if mutat < FRAC_INSERT {
            let src: usize = rng.gen_range(0, l);
            let dst: usize = rng.gen_range(0, l);
            let elem = old.customer_order.remove(src);
            old.customer_order.insert(dst, elem);
        }
        else if mutat < FRAC_INSERT + FRAC_SWAP {
            let src: usize = rng.gen_range(0, l);
            let dst: usize = rng.gen_range(0, l);
            let tmp = old.customer_order[src];
            old.customer_order[src] = old.customer_order[dst];
            old.customer_order[dst] = tmp;
        }
        else if mutat < FRAC_INSERT + FRAC_SWAP + FRAC_SCRAMBLE {
            let len: usize = rng.gen_range(0, l);
            let begin: usize = rng.gen_range(0, l - len);
            let mut sub: Vec<i64> = old.customer_order.iter().skip(begin).take(len).map(|&e| e).collect();
            sub.shuffle(rng);
            for (i, &e) in sub.iter().enumerate() {
                old.customer_order[i + begin] = e;
            }
        }
    }
    Genome::generate(old.customer_order, depots, customers)
}

pub fn crossover(parent1: &Genome, parent2: &Genome, depots: &Vec<Depot>, customers: &Vec<Customer>, total_vehicles: usize, rng: &mut ThreadRng) -> (Genome, Genome) {
    let child1: Vec<i64>;
    let child2: Vec<i64>;
    let cross: f64 = rng.gen();
    if cross < PROB_CROSSOVER {
        let cross: f64 = rng.gen();
        if cross < FRAC_ORDER1 {
            child1 = order_1_crossover(parent1, parent2, total_vehicles, rng);
            child2 = order_1_crossover(parent2, parent1, total_vehicles, rng);
        }
        else if cross < FRAC_ORDER1 + FRAC_PMX {
            let children = partially_mapped_crossover(parent1, parent2, customers.len(), rng);
            child1 = children.0;
            child2 = children.1;
        }
        else if cross < FRAC_ORDER1 + FRAC_PMX + FRAC_EDGE_RECOMB {
            let children = edge_recombination_crossover(parent1, parent2, customers.len(), rng);
            child1 = children.0;
            child2 = children.1;
        }
        else {
            child1 = parent1.customer_order.clone();
            child2 = parent2.customer_order.clone();
        }
    }
    else {
        child1 = parent1.customer_order.clone();
        child2 = parent2.customer_order.clone();
    }
    
    (Genome::generate(child1, depots, customers), Genome::generate(child2, depots, customers))
}   

pub fn order_1_crossover(parent1: &Genome, parent2: &Genome, total_vehicles: usize, rng: &mut ThreadRng) -> Vec<i64> {
    let mut child = Vec::new();

    let len = rng.gen_range(0, parent1.customer_order.len());
    let begin = rng.gen_range(0, parent1.customer_order.len() - len);
    let mut zero_count = 0;
    
    for &n in parent1.customer_order.iter().skip(begin).take(len) {
        if n == 0 {
            zero_count = zero_count + 1;
        }
        child.push(n);
    }
    for &n in parent2.customer_order.iter().skip(begin+len).chain(parent2.customer_order.iter().take(begin+len)) {
        if n == 0 && zero_count < total_vehicles - 1 {
            zero_count = zero_count + 1;
            child.push(n);
        }
        else if !child.contains(&n) {
            child.push(n);
        }
    }
    child
}

pub fn partially_mapped_crossover(parent1: &Genome, parent2: &Genome, num_customers: usize, rng: &mut ThreadRng) -> (Vec<i64>, Vec<i64>) {
    // Idea : Transform all zeroes into num_customers + 1 to num_customers + num_vehicles, do algo then turn them back to 0
    let mut p1: Vec<i64> = Vec::new();
    let mut p2: Vec<i64> = Vec::new();
    let mut transform = num_customers as i64;
    for &c in &parent1.customer_order {
        if c == 0 {
            transform = transform + 1;
            p1.push(transform);
        }
        else {
            p1.push(c);
        }
    }
    transform = num_customers as i64;
    for &c in &parent2.customer_order {
        if c == 0 {
            transform = transform + 1;
            p2.push(transform);
        }
        else {
            p2.push(c);
        }
    }

    let l = p1.len();
    let len = rng.gen_range(0, l);
    let begin = rng.gen_range(0, l - len);

    let mut child1 = vec![MAX; l];
    for (i, &n) in p1.iter().skip(begin).take(len).enumerate() {
        child1[i+begin] = n;
    }
    for &n in p2.iter().skip(begin).take(len) {
        if !child1.contains(&n) {
            let mut var = n;
            let mut idx;
            let mut over = false;
            while !over {
                idx = p2.iter().position(|&el| el == var).unwrap();
                var = p1[idx];
                idx = p2.iter().position(|&el| el == var).unwrap();
                if !(begin <= idx && idx < begin + len) {
                    over = true;
                    child1[idx] = n;
                }
            }
        }
    }
    let mut next_empty = 0;
    for &n in &p2 {
        if !child1.contains(&n) {
            while child1[next_empty] != MAX {
                next_empty = next_empty + 1;
            }
            child1[next_empty] = n;
        }
    }

    let mut child2 = vec![MAX; l];
    for (i, &n) in p2.iter().skip(begin).take(len).enumerate() {
        child2[i+begin] = n;
    }
    for &n in p1.iter().skip(begin).take(len) {
        if !child2.contains(&n) {
            let mut var = n;
            let mut idx;
            let mut over = false;
            while !over {
                idx = p1.iter().position(|&el| el == var).unwrap();
                var = p2[idx];
                idx = p1.iter().position(|&el| el == var).unwrap();
                if !(begin <= idx && idx < begin + len) {
                    over = true;
                    child2[idx] = n;
                }
            }
        }
    }
    let mut next_empty = 0;
    for &n in &p1 {
        if !child2.contains(&n) {
            while child2[next_empty] != MAX {
                next_empty = next_empty + 1;
            }
            child2[next_empty] = n;
        }
    }

    for i in 0..child1.len() {
        if child1[i] > num_customers as i64 {
            child1[i] = 0;
        }
    }
    for i in 0..child2.len() {
        if child2[i] > num_customers as i64 {
            child2[i] = 0;
        }
    }
    (child1, child2)
}

pub fn edge_recombination_crossover(parent1: &Genome, parent2: &Genome, num_customers: usize, rng: &mut ThreadRng) -> (Vec<i64>, Vec<i64>) {
    // Idea : Transform all zeroes into num_customers + 1 to num_customers + num_vehicles, do algo then turn them back to 0
    let mut p1: Vec<i64> = parent1.customer_order.clone();
    let mut p2: Vec<i64> = parent2.customer_order.clone();
    let mut transform = num_customers as i64;
    for i in 0..p1.len() {
        if p1[i] == 0 {
            transform = transform + 1;
            p1[i] = transform;
        }
    }
    transform = num_customers as i64;
    for i in 0..p2.len() {
        if p2[i] == 0 {
            transform = transform + 1;
            p2[i] = transform;
        }
    }
    let mut child1 = Vec::new();
    let mut child2 = Vec::new();

    let mut neighbor_list1: Vec<BTreeSet<i64>> = vec![BTreeSet::new(); p1.len()];
    for (i, &n) in p1.iter().enumerate() {
        let before = if i == 0 {p1.len() - 1} else {i - 1};
        let after = if i == p1.len() - 1 {0} else {i + 1};
        neighbor_list1[(n - 1) as usize].insert(p1[before]);
        neighbor_list1[(n - 1) as usize].insert(p1[after]);
    }
    for (i, &n) in p2.iter().enumerate() {
        let before = if i == 0 {p2.len() - 1} else {i - 1};
        let after = if i == p2.len() - 1 {0} else {i + 1};
        neighbor_list1[(n - 1) as usize].insert(p2[before]);
        neighbor_list1[(n - 1) as usize].insert(p2[after]);
    }
    let mut neighbor_list2 = neighbor_list1.clone();

    let mut head = p1[0];
    let mut inserted = BTreeSet::new();
    for i in 1..=p1.len() {
        inserted.insert(i);
    }
    child1.push(head);
    while child1.len() < p1.len() {
        inserted.remove(&(head as usize));
        for i in 0..neighbor_list1.len() {
            neighbor_list1[i].remove(&head);
        }
        if neighbor_list1[(head - 1) as usize].is_empty() {
            let rand = rng.gen_range(0, inserted.len());
            head = *inserted.iter().nth(rand).unwrap() as i64;
        }
        else {
            let mut min = p1.len();
            let mut candidates = Vec::new();
            for &node in &neighbor_list1[(head - 1) as usize] {
                if neighbor_list1[(node - 1) as usize].len() < min {
                    candidates = vec![node];
                    min = neighbor_list1[(node - 1) as usize].len();
                }
                else if neighbor_list1[(node - 1) as usize].len() == min {
                    candidates.push(node);
                }
            }
            let new_head_idx: usize = if candidates.len() == 1 {0} else {rng.gen_range(0, candidates.len())};
            head = candidates[new_head_idx];
        }
        child1.push(head);
    }
    
    let mut head = p2[0];
    let mut inserted = BTreeSet::new();
    for i in 1..=p2.len() {
        inserted.insert(i);
    }
    child2.push(head);
    while child2.len() < p2.len() {
        inserted.remove(&(head as usize));
        for i in 0..neighbor_list2.len() {
            neighbor_list2[i].remove(&head);
        }
        if neighbor_list2[(head - 1) as usize].is_empty() {
            let rand = rng.gen_range(0, inserted.len());
            head = *inserted.iter().nth(rand).unwrap() as i64;
        }
        else {
            let mut min = p2.len();
            let mut candidates = Vec::new();
            for &node in &neighbor_list2[(head - 1) as usize] {
                if neighbor_list2[(node - 1) as usize].len() < min {
                    candidates = vec![node];
                    min = neighbor_list2[(node - 1) as usize].len();
                }
                else if neighbor_list2[(node - 1) as usize].len() == min {
                    candidates.push(node);
                }
            }
            let new_head_idx: usize = if candidates.len() == 1 {0} else {rng.gen_range(0, candidates.len())};
            head = candidates[new_head_idx];
        }
        child2.push(head);
    }

    for i in 0..child1.len() {
        if child1[i] > num_customers as i64 {
            child1[i] = 0;
        }
    }
    for i in 0..child2.len() {
        if child2[i] > num_customers as i64 {
            child2[i] = 0;
        }
    }
    (child1, child2)
}