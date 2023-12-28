use std::{cmp::Ordering, time::Instant, sync::Mutex};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug)]
struct Pair {
    pub winner: usize,
    pub loser: usize,
    pub victory_ratio: usize
}

impl Pair {
    fn new_from_tuple(preference_diff: (usize, usize), par: (usize, usize)) -> Option<Self> {
        //println!(
            //"Preference diff for {} and {} => {}",
           // par.0,
            //par.1,
            //preference_diff.0 as i32 - preference_diff.1 as i32
        //);
        if preference_diff.0 as i32 - preference_diff.1 as i32 != 0 as i32 {
            let mut winner = par.0;
            let mut loser = par.1;

            if preference_diff.0 < preference_diff.1 {
                loser = par.0;
                winner = par.1;
            }
            let victory_ratio = if winner == par.0 {
                preference_diff.0 - preference_diff.1
            }
            else {
                preference_diff.1 - preference_diff.0
            };

            return Some(Self { winner, loser, victory_ratio });
        }

        None
    }
}

fn main() {
    let start = Instant::now();
    // max number of candidates

    // constant of candidates
    let candidates: Vec<String> = vec!["Chizaram", "Charlie", "Bob", "Alice"]
        .into_iter()
        .map(|f| f.to_string())
        .collect();
    let mut pairs: Vec<Pair>;
    let mut pair_count = 0;
    //let votes
    let votes = || {
        let mut rev = candidates.to_owned();
        rev.reverse();
        vec![
            candidates.to_owned(),
            candidates.to_owned(),
            rev.clone(),
            rev,
            vec!["Chizaram", "Bob", "Charlie", "Alice"]
                .into_iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
        ]
    };

    let mut preferences: Vec<Vec<usize>> = vec![vec![0; candidates.len()]; candidates.len()];

    let mut locked = vec![vec![false; candidates.len()]; candidates.len()];

    // query for votes
    let min_s = Instant::now();
    let voted = Mutex::new(votes());
    let gen_v = 10000;
    (0..gen_v).into_par_iter().for_each(|_| {
        let mut voted = voted.lock().unwrap();
        voted.extend(votes())
    });
    let voted = voted.lock().unwrap().to_owned();
    println!("Generating {gen_v} votes took {} seconds", (Instant::now() - min_s).as_secs_f64());
    for vote in voted {
        //init active voters ranks
        let mut ranks: Vec<usize> = vec![0; candidates.len()];

        //query and vote for each rank
        for (cand_in, cand) in vote.into_iter().enumerate() {
            // if not vote break
            if !votem(cand_in, cand, &mut ranks, &candidates) {
                //println!("Invalid vote");
                return;
            }
        }
        //println!("Ranks: {:?}", ranks);
        record_preferences(ranks, &mut preferences);
        //println!("\n Pref: {preferences:?}");
    }

    pairs = add_pairs(&mut preferences, &mut pair_count);

    lock_pair(&mut pairs, &mut locked);

    
    //println!("Pairs : {:?}", pairs);

    print!("\n\n *************** Winner ************* \n\n");
    print_winner(&locked, &candidates);
    println!("Generated winner in approximately {} milliseconds or {} seconds ", (Instant::now() - start).as_millis(), (Instant::now() - start).as_secs_f64());
}

/// Given the current voters virtual rank , the name voted candidate, the candidates vectir and the cand rank index <br>
/// This function update the vRank with the candidate found index <br>
/// And returns true or false on failure to update
fn votem(cand_in: usize, cand: String, ranks: &mut Vec<usize>, candidates: &Vec<String>) -> bool {
    for (ind, candid) in candidates.iter().enumerate() {
        if &cand == candid {
            ranks[cand_in] = ind;
            return true;
        }
    }

    false
}

/// Update the preference array based on the current voter's ranks <br>
/// <b> Remember that preferences arrays if preferences [candidate - index]over[candidate i];
fn record_preferences(ranks: Vec<usize>, preferences: &mut Vec<Vec<usize>>) {
    let mut ranks = ranks.iter();
    while let Some(active_rank) = ranks.next() {
        // set the active rank to be prefered over all the rest
        //println!("rank {:?}", ranks);
        let _ = &ranks.clone().for_each(|c| {
            preferences[*active_rank][*c] += 1;
        });
    }
}


/// - Add each pair of candidates to pairs array if one candidate is preferred over the other <br><hr>
/// - Update global variable pair_count to be the total number of pairs
/// - Also update the pair count for each pair generation
fn add_pairs(preferences: &mut Vec<Vec<usize>>, pair_count: &mut usize) -> Vec<Pair> {
    // for each row by the index of the candidate running the row
    let mut all_pairs = Vec::new();
    let cands_len_range = 0..preferences[0].len();

    // check if there is a winner and if create a losser and winner
    for active_ind in cands_len_range {
        //println!("Index actively searching => {active_ind}");
        // loop for the rest for slice excepting current node
        let uncalc = preferences[0][active_ind..].len();
        let mut recursive_cand_in = active_ind;
        for _ in 0..uncalc { 
            let pref_val = preferences[active_ind][recursive_cand_in];
            let other_pref_val = preferences[recursive_cand_in][active_ind];

            if let Some(e) = Pair::new_from_tuple((pref_val, other_pref_val), (active_ind, recursive_cand_in)) {
                all_pairs.push(e);
                *pair_count += 1;
            }
            recursive_cand_in +=1 ;
        }
        
        
    }

    all_pairs.sort_by(|a, b| a.victory_ratio.partial_cmp(&b.victory_ratio).unwrap_or_else(|| Ordering::Equal));
    all_pairs.reverse();
    return all_pairs;
}


/// Lock Pair function
fn lock_pair(pairs: &mut Vec<Pair>, lock_graph: &mut Vec<Vec<bool>>) {
    for pair in pairs {
        lock_graph[pair.winner][pair.loser] = true;
    }
}

fn print_winner(locked_graph: &Vec<Vec<bool>>, _candidates: &Vec<String>) {
    'header: for f in 0..locked_graph[0].len() {
        for x in 0..locked_graph[f].len()
        {
            let val = locked_graph[f][x];
            if f == x {
                continue;
            }
            if val == false {
                continue 'header;
            }

            if x == locked_graph[f].len()-1 {
                // found winner just //print
                println!("                {}            ", _candidates[f])
            }
        }
    }
}


mod Unused {
    use crate::Pair;

    /// - Add each pair of candidates to pairs array if one candidate is preferred over the other <br><hr>
/// - Update global variable pair_count to be the total number of pairs
/// - Also update the pair count for each pair generation
///
#[allow(unused)]
fn add_pair(preferences: &mut Vec<Vec<usize>>, pair_count: &mut usize) -> Vec<Pair> {
    // for each row by the index of the candidate running the row
    let mut all_pairs = Vec::new();

    // check if there is a winner and if create a losser and winner
    for (cand, row) in preferences.iter().enumerate() {
        for (over_cand, col) in row.iter().enumerate() {
            let pref_val = preferences[cand][over_cand];
            let other_pref_val = preferences[over_cand][cand];
            if let Some(e) = Pair::new_from_tuple((pref_val, other_pref_val), (cand, over_cand)) {
                all_pairs.push(e);
                *pair_count += 1;
            }
        }
    }

    return all_pairs;
}

}