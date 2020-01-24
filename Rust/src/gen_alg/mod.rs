use rand::prelude::*;
use std::collections::HashSet;
use std::cmp::Ordering;
use rand::seq::SliceRandom;
use std::i64::MAX;
use std::collections::BTreeSet;
mod test;

// General constants
const POP_SIZE: usize = 10_000; // Make sure POP_SIZE and ELITES have the same parity
const ELITES: i64 = 500;
const GENERATIONS: i64 = 1_000;
const POOL_SIZE: i64 = 500;
const CHILDREN: usize = 4;
const REPAIRED: bool = true;

// Mutation and Crossover constants
const PROB_MUTATION: f64 = 0.4;
const FRAC_INSERT: f64 = 7.0/10.0;
const FRAC_SWAP: f64 = 1.0/10.0;
const FRAC_SCRAMBLE: f64 = 2.0/10.0;

const PROB_CROSSOVER: f64 = 0.9;
const FRAC_ORDER1: f64 = 0.1;
const FRAC_PMX: f64 = 0.9;
const _FRAC_EDGE_RECOMB: f64 = 0.0;

// Selection constants
const TOURNAMENT_SIZE: usize = 500; // 1 is random, higher up to pop.len() is higher pressure
const SELECTION_PRESSURE: f64 = 0.9; // Higher = closer to deterministic, should be between 0 and 1

pub fn train(input: String) -> (String, String) {
    let mut rng = thread_rng();
    let mut averages = String::new();
    let mut bests = String::new();

    let mut depots: Vec<Depot> = Vec::new();
    let mut customers: Vec<Customer> = Vec::new();
    let vehicles_per_depot = read_input(&mut depots, &mut customers, input);
    let num_vehicles = vehicles_per_depot as usize * depots.len();
    let cust_per_car = customers.len()/num_vehicles + 1;

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
                let (child1, child2) = gene_pool[p1].crossover(&gene_pool[p2], &depots, &customers, num_vehicles, cust_per_car);
                new_generation.push(child1.mutate(&depots, &customers, cust_per_car));
                new_generation.push(child2.mutate(&depots, &customers, cust_per_car));
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
        println!("Gen {}, Pool : {}, Valid Avg : {}, Avg : {}, Best : {}, Valid : {}", 
                    i + 1, gene_pool.len(), if valid == 0 {0} else {total_v/valid}, total_a/POP_SIZE as i64, best, valid);
        bests.push_str(format!("{} ", best).as_str());
        averages.push_str(format!("{} ", if valid == 0 {0} else {total_v/valid}).as_str());

        pop = new_generation;
    }

    // Then take the best individual, and display it
    return (manage_outputs(pop.pop().unwrap(), &depots, &customers), format!("{}\n{}", bests, averages));
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

// Selection methods
fn fitness_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = thread_rng();
    let mut pool: Vec<Genome> = Vec::new();
    let mut fitness_cdf: Vec<f64> = Vec::new();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }

    // make list of increasing fitness
    let mut acc = 0.0;
    for g in old_pop.iter() {
        acc += g.fitness;
        fitness_cdf.push(acc);
    }
    let fit_total = acc;

    while pool.len() < POOL_SIZE as usize {
        let mut rand: f64 = rng.gen();
        rand = rand * fit_total;
        pool.push(old_pop[find(rand, &fitness_cdf)].clone());
    }
    
    return pool;
}

fn tournament_selection(mut old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut pool = Vec::new();
    let mut rng = thread_rng();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }

    for _ in 0..POOL_SIZE {
        
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
    let mut rng = thread_rng();
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

// Misc
fn find(elem: f64, list: &Vec<f64>) -> usize {
    let mut low = 0;
    let mut high = list.len() - 1;
    loop {
        let middle = (low+high)/2;
        if middle == low {
            return high
        }
        if elem < list[middle] {
            high = middle;
        }
        else {
            low = middle;
        }
    }
}

impl Genome {
    // Mutation functions 

    fn mutate(mut self, depots: &Vec<Depot>, customers: &Vec<Customer>, cust_per_car: usize) -> Genome {
        let mut rng = thread_rng();
        let l = self.customer_order.len();
        let mutat: f64 = rng.gen();

        if mutat < PROB_MUTATION {
            let mutat: f64 = rng.gen();
            if mutat < FRAC_INSERT {
                let src: usize = rng.gen_range(0, l);
                let dst: usize = rng.gen_range(0, l);
                let elem = self.customer_order.remove(src);
                self.customer_order.insert(dst, elem);
            }
            else if mutat < FRAC_INSERT + FRAC_SWAP {
                let src: usize = rng.gen_range(0, l);
                let dst: usize = rng.gen_range(0, l);
                let tmp = self.customer_order[src];
                self.customer_order[src] = self.customer_order[dst];
                self.customer_order[dst] = tmp;
            }
            else if mutat < FRAC_INSERT + FRAC_SWAP + FRAC_SCRAMBLE {
                let len: usize = rng.gen_range(0, l);
                let begin: usize = rng.gen_range(0, l - len);
                let mut sub: Vec<i64> = self.customer_order.iter().skip(begin).take(len).map(|&e| e).collect();
                sub.shuffle(&mut rng);
                for (i, &e) in sub.iter().enumerate() {
                    self.customer_order[i + begin] = e;
                }
            }
        }
        Self::generate(self.customer_order, depots, customers, cust_per_car)
    }

    // Crossover functions 

    pub fn crossover(&self, parent2: &Genome, depots: &Vec<Depot>, customers: &Vec<Customer>, total_vehicles: usize, cust_per_car: usize) -> (Genome, Genome) {
        let child1: Vec<i64>;
        let child2: Vec<i64>;
        let mut rng = thread_rng();
        let cross: f64 = rng.gen();

        if cross < FRAC_ORDER1 {
            child1 = self.order_1_crossover(parent2, total_vehicles);
            child2 = parent2.order_1_crossover(self, total_vehicles);
        }
        else if cross < FRAC_ORDER1 + FRAC_PMX {
            let children = parent2.partially_mapped_crossover(self, customers.len());
            child1 = children.0;
            child2 = children.1;
        }
        else { // (if cross < FRAC_ORDER1 + FRAC_PMX + FRAC_EDGE_RECOMB)
            let children = parent2.edge_recombination_crossover(self, customers.len());
            child1 = children.0;
            child2 = children.1;
        }
        (Self::generate(child1, depots, customers, cust_per_car), Self::generate(child2, depots, customers, cust_per_car))
    }   

    pub fn order_1_crossover(&self, parent2: &Genome, total_vehicles: usize) -> Vec<i64> {
        let mut rng = thread_rng();

        let mut child = Vec::new();

        let len = rng.gen_range(0, self.customer_order.len());
        let begin = rng.gen_range(0, self.customer_order.len() - len);
        let mut zero_count = 0;
        
        for &n in self.customer_order.iter().skip(begin).take(len) {
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

    pub fn partially_mapped_crossover(&self, parent2: &Genome, num_customers: usize) -> (Vec<i64>, Vec<i64>) {
        // Idea : Transform all zeroes into num_customers + 1 to num_customers + num_vehicles, do algo then turn them back to 0
        let mut p1: Vec<i64> = self.customer_order.clone();
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

        let mut rng = thread_rng();

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

    pub fn edge_recombination_crossover(&self, parent2: &Genome, num_customers: usize) -> (Vec<i64>, Vec<i64>) {
        // Idea : Transform all zeroes into num_customers + 1 to num_customers + num_vehicles, do algo then turn them back to 0
        let mut p1: Vec<i64> = self.customer_order.clone();
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

        let mut rng = thread_rng();

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
                let dist = depots[depot].euclid_dist(x, y);
                duration = duration + dist;
                total_distance = total_distance + dist;
                if (depots[depot].max_duration != 0 && duration > depots[depot].max_duration) || load > depots[depot].max_load {
                    total_distance = total_distance * depots.len() as i64;
                    valid = false;
                }
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
                        let dist = cust.euclid_dist(x, y);
                        load = load + cust.demand;
                        duration = duration + cust.duration + dist;
                        total_distance = total_distance + dist;
                        x = cust.x;
                        y = cust.y;
                    }
                }
            }
        }

        // Check limits
        let dist = depots[depot].euclid_dist(x, y);
        duration = duration + dist;
        total_distance = total_distance + dist;
        if (depots[depot].max_duration != 0 && duration > depots[depot].max_duration) || load > depots[depot].max_load {
            total_distance = total_distance * depots.len() as i64;
            valid = false;
        }
        (total_distance, valid)
    }

    fn repair(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>, cust_per_car: usize) -> Vec<i64> {
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

        let mut x = depots[0].x;
        let mut y = depots[0].y;

        let mut acceptable_dist = depots[depot].max_duration/cust_per_car as i64 + 1;

        let mut i = 0;
        while i < customer_order.len() {
            if customer_order[i] == 0 {
                // Initialize new vehicle :
                vehicle = vehicle + 1;
                if vehicle >= depots[depot].vehicles {
                    vehicle = 0;
                    depot = depot + 1;
                    acceptable_dist = depots[depot].max_duration/cust_per_car as i64;
                }
                x = depots[depot].x;
                y = depots[depot].y;
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
                        let dist = cust.euclid_dist(x, y);
                        load = load + cust.demand;
                        duration = duration + cust.duration + dist;
                        x = cust.x;
                        y = cust.y;
                    }
                }
                if depots[depot].max_duration != 0 && duration + depots[depot].euclid_dist(x, y) > depots[depot].max_duration {
                    let (_, best_idx, best_dist) = customer_order.iter().skip(i).fold((i, i, MAX), |(curr_idx, idx, min_dist), &x| {
                        if x == 0 {
                            (curr_idx + 1, idx, min_dist)
                        }
                        else {
                            let cust = customers.get(x as usize - 1).unwrap();
                            let distance = cust.euclid_dist(prev_x, prev_y) + cust.euclid_dist(depots[depot].x, depots[depot].y);
                            if distance < min_dist {
                                (curr_idx + 1, curr_idx, distance)
                            }
                            else {
                                (curr_idx + 1, idx, min_dist)
                            }
                        }
                    });
                    let cust = customers.get(customer_order[best_idx] as usize - 1).unwrap();
                    let new_dur = prev_dur + cust.duration + cust.euclid_dist(prev_x, prev_y);
                    let new_load = prev_load + cust.demand;
                    if !((depots[depot].max_duration != 0 && new_dur + depots[depot].euclid_dist(cust.x, cust.y) > depots[depot].max_duration) || new_load > depots[depot].max_load) {
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
                else if load > depots[depot].max_load {
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
                let dist = depots[depot].euclid_dist(x, y);
                duration = duration + dist;
                total_distance = total_distance + dist;
                if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
                    return (String::from("Invalid duration"), None)
                }
                if load > depots[depot].max_load {
                    return (String::from("Invalid load"), None)
                }
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
                        let dist = cust.euclid_dist(x, y);
                        load = load + cust.demand;
                        duration = duration + dist;
                        total_distance = total_distance + dist;
                        x = cust.x;
                        y = cust.y;
                        cus_list.push_str(format!("{} ", c).as_str());
                    }
                }
            }
        }
        // Check limits
        let dist = depots[depot].euclid_dist(x, y);
        duration = duration + dist;
        total_distance = total_distance + dist;
        if depots[depot].max_duration != 0 && duration > depots[depot].max_duration {
            return (String::from("Invalid duration"), None)
        }
        if load > depots[depot].max_load {
            return (String::from("Invalid load"), None)
        }
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
        Self::generate(customer_list, depots, customers, total_vehicles)
    }

    fn fitness(total_distance: i64) -> f64 {
        if total_distance == 0 {
            0.0
        }
        else {
            1.0/total_distance as f64
        }
    }

    fn generate(mut customer_order: Vec<i64>, depots: &Vec<Depot>, customers: &Vec<Customer>, cust_per_car: usize) -> Genome {
        if REPAIRED {
            customer_order = Self::repair(customer_order, depots, customers, cust_per_car);
            customer_order.reverse();
            customer_order = Self::repair(customer_order, depots, customers, cust_per_car);
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

    fn swap_next_zero(i: usize, mut customer_order: Vec<i64>) -> (Vec<i64>, bool) {
        match customer_order.iter().skip(i).position(|&e| e == 0) {
            None => return (customer_order, false),
            Some(a) => {
                customer_order[i + a] = customer_order[i];
                customer_order[i] = 0;
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