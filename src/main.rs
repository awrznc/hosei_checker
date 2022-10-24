use std::{collections::{HashMap, HashSet}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
struct Hosei {
    base: f64,
    first: f64,
    multi: f64,
    bonus: f64,
    repeat: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Waza {
    id: String,
    dm: u64,
    hs: Option<Hosei>,
}

impl Waza {
    pub fn new(waza: &Waza) -> Waza{
        Waza {
            id: waza.id.to_owned(),
            dm: waza.dm,
            hs: Some(Hosei {
                base: 1.0,
                first: 1.0,
                multi: 1.0,
                bonus: 1.0,
                repeat: 1.0,
            })
        }
    }
}

struct HoseiChecker {
    target: Vec<Vec<Waza>>,
    result: HashMap<String, Waza>,
}

impl HoseiChecker {
    fn get_base_damages(combos: Vec<Vec<Waza>>) -> (Vec<Vec<Waza>>, HashMap<String, Waza>) {
        let mut commands = HashMap::new();
        let commands_in_combo = combos.iter().fold( HashSet::new(), |mut cic, combo| {
            if combo.len() <= 1 { return cic; }
            let mut combo_iter = combo.iter();
            let first_waza = combo_iter.next().unwrap();
            commands.insert( first_waza.id.to_owned(), Waza::new(first_waza) );
            combo_iter.for_each( |waza| { cic.insert(waza.id.to_string()); } );
            return cic;
        } );

        // check if there is a "base damage"
        // note: for simplicity of calculation
        assert!( commands_in_combo.iter().all( |x| commands.contains_key(x) ) );
        return (combos, commands);
    }

    fn load_yaml(path: &str) -> Vec<Vec<Waza>> {
        let file = std::fs::File::open(path).unwrap();
        return serde_yaml::from_reader(file).unwrap();
    }

    pub fn new(path: &str) -> HoseiChecker {
        let combos: Vec<Vec<Waza>> = HoseiChecker::load_yaml(path);
        let (combos, commands) = HoseiChecker::get_base_damages(combos);
        let mut hosei_checher = HoseiChecker {
            target: combos,
            result: commands,
        };

        // calculate hosei
        hosei_checher.calculate();

        return hosei_checher;
    }

    fn sieve_of_eratosthenes(max_number: usize) -> Vec<u64> {
        let mut nums = vec![true; max_number];
        nums[0] = false;
        nums[1] = false;

        let max = (max_number as f64).sqrt() as usize;
        for i in 2..max {
            if nums[i] == false { continue; }
            nums[i] = true;
            for j in i..(max_number/i) {
                nums[j*i] = false;
            }
        }

        return nums.iter().enumerate().filter_map(|(i, num)| {
            if *num { Some(i as u64) } else { None }
        } ).collect();
    }


    // single digit only
    fn prime_factorization(&self, value :u64) -> Vec<u64>{
        let mut elements: Vec<u64> = Vec::new();
        let mut base = value;

        for prime in HoseiChecker::sieve_of_eratosthenes(value as usize) {
            'div: loop {
                if base == 0 || base % prime != 0 { break 'div; }
                elements.push(prime);
                base = base / prime;
            }
        }

        return elements;
    }

    fn calculate(&mut self) {

        let mut candidate_for_base_hosei: Vec<Vec<(u64,u64)>> = Vec::new();

        // get candidate for base hosei from first waza and second waza
        self.target.iter().for_each(|combo| {
            let before_waza = combo.first().unwrap();
            let before_damage = before_waza.dm;

            let waza = &combo[1];
            let current_damage = waza.dm - before_damage;
            let hosei: f64 = current_damage as f64 / before_damage as f64;

            // get the elements of the hosei value by prime factorization
            let elements = self.prime_factorization((hosei * 10000.0) as u64);

            // truth table like search
            let mut pairs: Vec<(u64,u64)> = Vec::new();
            let len = (2 as usize).pow(elements.len() as u32);
            for position in 0..len {
                let mut a: u64 = 1;
                let mut b: u64 = 1;
                for i in 0..elements.len() {
                    if (position) & (1 << i) == 0 {
                        a = a * elements[i];
                    } else {
                        b = b * elements[i];
                    }
                }

                if a != 1 && b != 1 && pairs.iter().any(|pare| (pare.0 == a && pare.1 == b) || (pare.0 == b && pare.1 == a)) == false {
                    pairs.push((a, b));
                }
            }

            candidate_for_base_hosei.push(pairs);
        });

        // get most frequent value
        let mut hosei_counter: HashMap<u64, u64> = HashMap::new();
        candidate_for_base_hosei.iter().for_each(|v| {
            v.iter().for_each(|pair| {
                let (a, b) = pair;
                let a_value: u64 = match hosei_counter.get(a) {
                    Some(value) => *value + 1,
                    None => 1,
                };
                let b_value: u64 = match hosei_counter.get(b) {
                    Some(value) => *value + 1,
                    None => 1,
                };
                hosei_counter.insert(*a, a_value);
                hosei_counter.insert(*b, b_value);
            });
        });

        let base_hosei = hosei_counter.iter().max_by_key(|(_,b)| **b ).unwrap();
        for (_, waza) in self.result.iter_mut() {
            (*waza).hs.as_mut().unwrap().base = *(base_hosei.0) as f64 / 100.0;
        }

    }

    pub fn print(&self) {
        self.result.iter().for_each(|v| println!("{} - dm: {}, base_hs: {}", v.0, v.1.dm, v.1.hs.unwrap().base))
    }
}

fn main() {

    // load and set
    let hc = HoseiChecker::new("combo.yaml");
    println!("INPUTS: {:#?}", hc.target);

    // print result
    hc.print();
}
