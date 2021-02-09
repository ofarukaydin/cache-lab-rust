extern crate cc;
extern crate getopts;

use getopts::Options;
use std::env;
use std::fs;
#[derive(Debug, Clone)]
struct CacheLine {
    valid_bit: i32,
    tag_bit: u64,
    block_bit: String,
    used_counter: i32,
}

struct CacheParameter {
    s: i32,
    S: i64,
    b: i32,
    B: i32,
    E: i32,
    hit: i32,
    miss: i32,
    eviction: i32,
}
#[derive(Debug, Clone)]
struct CacheSet {
    lines: Vec<CacheLine>,
}
#[derive(Debug)]
struct Cache {
    sets: Vec<CacheSet>,
}

fn check_hit(line: &CacheLine, tag: u64) -> bool {
    if line.valid_bit == 1 && line.tag_bit == tag {
        return true;
    }
    false
}

fn check_full(set: &CacheSet) -> bool {
    for line in &set.lines {
        if line.valid_bit == 0 {
            return false;
        };
    }
    true
}

fn find_index(set: &mut CacheSet) -> &mut CacheLine {
    for i in 0..set.lines.len() {
        if set.lines[i].valid_bit == 0 {
            return &mut set.lines[i];
        }
    }
    &mut set.lines[0]
}

fn find_evict(set: &mut CacheSet) -> &mut CacheLine {
    let mut min = set.lines[0].used_counter;
    let mut index = 0;

    for i in 0..set.lines.len() {
        if min > set.lines[i].used_counter {
            index = i;
            min = set.lines[i].used_counter;
        }
    }
    &mut set.lines[index]
}

fn find_max(set: &mut CacheSet) -> i32 {
    let mut max = set.lines[0].used_counter;

    for i in 0..set.lines.len() {
        if set.lines[i].used_counter > max {
            max = set.lines[i].used_counter;
        }
    }
    max
}

fn make_cache(S: i64, E: i32) -> Cache {
    let mut cache = Cache { sets: vec![] };
    let mut cache_set = CacheSet { lines: vec![] };
    for _ in 0..E {
        cache_set.lines.push(CacheLine {
            block_bit: String::from(""),
            used_counter: 0,
            tag_bit: 0,
            valid_bit: 0,
        });
    }
    for _ in 0..S {
        cache.sets.push(cache_set.clone())
    }
    cache
}

fn simulate(cache: &mut Cache, cache_property: &mut CacheParameter, address: u64) {
    let tag_size = 64 - (cache_property.b + cache_property.s);
    let tag = address >> (cache_property.b + cache_property.s);
    let set_index = (address << tag_size) >> (tag_size + cache_property.b);
    let mut cache_set = &mut cache.sets[set_index as usize];

    let mut hit = false;

    for i in 0..cache_property.E {
        let current_line = &cache_set.lines[i as usize];

        if check_hit(current_line, tag) {
            cache_property.hit += 1;
            hit = true;
            let max = find_max(&mut cache_set);
            cache_set.lines[i as usize].used_counter = max + 1
        }
    }

    let max = find_max(&mut cache_set);

    if !hit && check_full(&cache_set) {
        let mut evict = find_evict(&mut cache_set);

        cache_property.miss += 1;
        cache_property.eviction += 1;

        evict.tag_bit = tag;
        evict.used_counter = max + 1;
    } else if !hit {
        let mut line = find_index(&mut cache_set);

        cache_property.miss += 1;
        line.tag_bit = tag;
        line.valid_bit = 1;
        line.used_counter = max + 1;
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt(
        "s",
        "",
        "Number of set index bits (S = 2s is the number of sets)",
        "Set Index",
    );
    opts.optopt(
        "E",
        "",
        "Associativity (number of lines per set)",
        "Associativity",
    );
    opts.optopt(
        "b",
        "",
        "Number of block bits (B = 2b is the block size)",
        "Block bits",
    );
    opts.optopt("t", "", "Name of the valgrind trace to replay", "Tracefile");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let set_bits: i32 = matches.opt_str("s").unwrap().parse().unwrap();
    let associativity: i32 = matches.opt_str("E").unwrap().parse().unwrap();
    let block_bits: i32 = matches.opt_str("b").unwrap().parse().unwrap();
    let trace = matches.opt_str("t").unwrap();

    let block_size = 1 << block_bits;
    let number_of_sets = 1 << set_bits;

    let mut cache_property = CacheParameter {
        B: block_size,
        S: number_of_sets,
        s: set_bits,
        b: block_bits,
        E: associativity,
        eviction: 0,
        hit: 0,
        miss: 0,
    };

    let mut cache = make_cache(number_of_sets, associativity);

    let contents = fs::read_to_string(trace).expect("Something went wrong reading the file");

    for line in contents
        .lines()
        .filter(|line| !line.starts_with('I'))
        .map(|line| line.trim_start())
    {
        let address_str = line.split(' ').collect::<Vec<_>>()[1]
            .split(',')
            .collect::<Vec<_>>()[0];
        let address_hex = u64::from_str_radix(address_str, 16).unwrap();
        match &line[0..1] {
            "M" => {
                simulate(&mut cache, &mut cache_property, address_hex);
                simulate(&mut cache, &mut cache_property, address_hex)
            }
            "L" => simulate(&mut cache, &mut cache_property, address_hex),
            "S" => simulate(&mut cache, &mut cache_property, address_hex),
            _ => println!("Should not happen"),
        }
    }

    print!(
        "hits:{} misses:{} evictions:{}",
        cache_property.hit, cache_property.miss, cache_property.eviction
    );
}
