use std::env;
use std::error;
use std::error::Error;
use std::fs;
use std::fmt;
use std::io;
use std::io::Read;
use std::io::BufReader;
use std::num::Wrapping;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct InputError;

impl Error for InputError{}

impl fmt::Display for InputError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "usage: ./sha256 (= reads stdin)\nor\nusage: ./sha256 -s <string>\nor\nusage: ./sha256 <filename>")
	}
}

const H0: u32 = 0x6a09e667;
const H1: u32 = 0xbb67ae85;
const H2: u32 = 0x3c6ef372;
const H3: u32 = 0xa54ff53a;
const H4: u32 = 0x510e527f;
const H5: u32 = 0x9b05688c;
const H6: u32 = 0x1f83d9ab;
const H7: u32 = 0x5be0cd19;

const K: [u32; 64] = [0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
	0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
	0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
	0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
	0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
	0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
	0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2];

fn s0_func(x: u32, r1: u32, r2: u32, s: u32) -> u32 {
	x.rotate_right(r1) ^ x.rotate_right(r2) ^ (x >> s)
}

fn s1_func(x: u32, r1: u32, r2: u32, s: u32) -> u32 {
	x.rotate_right(r1) ^ x.rotate_right(r2) ^ (x >> s)
}

// fn print_bytes(s: &str) {
// 	let bytes = s.as_bytes();

// 	for byte in bytes.iter() {
// 		println!("{byte:x}");
// 	}
// }

// fn print_bytes(s: &Vec<u8>) {

// 	for byte in s.iter() {
// 		print!("{byte:x} ");
// 	}
// 	println!()
// }

fn make_padded_vec(input: &str) -> Vec<u8> {

	let og_len_in_bits = Wrapping(input.len() as u64 * 8);
	let mut input_vec: Vec<u8> = Vec::new();
	input_vec.extend(input.as_bytes());
	input_vec.push(128_u8);

	while (input_vec.len() * 8) % 512 != 448 {
		input_vec.push(0_u8);
	}
	let og_len_in_bytes : [u8; 8] = og_len_in_bits.0.to_be_bytes();
	let mut og_len_in_bytes = og_len_in_bytes.to_vec();
	input_vec.append(&mut og_len_in_bytes);

	input_vec
}

fn hash_loop(input: Vec<u8>, hash: &mut [u32; 8]) {

	for chunk in input.as_slice().chunks(64) {
		
		let mut w: [u32; 64] = [0_u32; 64];
		const BASE: i32 = 2;
		
		for i in 0..16 {
			let n1: u32 = (chunk[4 * i] as u32).wrapping_mul(BASE.pow(24) as u32);
			let n2: u32 = (chunk[4 * i + 1] as u32).wrapping_mul(BASE.pow(16) as u32);
			let n3: u32 = (chunk[4 * i + 2] as u32).wrapping_mul(BASE.pow(8) as u32);
			let n4: u32 = chunk[4 * i + 3] as u32;
			let n: u32 = n1.wrapping_add(n2)
						.wrapping_add(n3)
						.wrapping_add(n4);
			w[i] = n;
		}

		for i in 16..64 {
			let s0: u32 = s0_func(w[i - 15], 7, 18, 3);
			let s1: u32 = s1_func(w[i - 2], 17, 19, 10);
			w[i] = w[i - 16]
				.wrapping_add(s0)
				.wrapping_add(w[i - 7])
				.wrapping_add(s1);
		}

		let mut a: u32 = hash[0];
		let mut b: u32 = hash[1];
		let mut c: u32 = hash[2];
		let mut d: u32 = hash[3];
		let mut e: u32 = hash[4];
		let mut f: u32 = hash[5];
		let mut g: u32 = hash[6];
		let mut h: u32 = hash[7];

		for i in 0..64 {
			let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
			let ch = (e & f) ^ (!e & g);
			let temp1 = h.wrapping_add(s1)
							.wrapping_add(ch)
							.wrapping_add(K[i])
							.wrapping_add(w[i]);
			let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
			let maj = (a & b) ^ (a & c) ^ (b & c);
			let temp2 = s0.wrapping_add(maj);

			h = g;
			g = f;
			f = e;
			e = d.wrapping_add(temp1);
			d = c;
			c = b;
			b = a;
			a = temp1.wrapping_add(temp2);
		}

		hash[0] = hash[0].wrapping_add(a);
		hash[1] = hash[1].wrapping_add(b);
		hash[2] = hash[2].wrapping_add(c);
		hash[3] = hash[3].wrapping_add(d);
		hash[4] = hash[4].wrapping_add(e);
		hash[5] = hash[5].wrapping_add(f);
		hash[6] = hash[6].wrapping_add(g);
		hash[7] = hash[7].wrapping_add(h);
	}
}

fn read_from_stdin() -> Result<String>
{
	let mut out = String::new();

	let result = io::stdin().read_to_string(&mut out);
	
	match result {
		Ok(_) => Ok(out),
		Err(error) => Err(Box::new(error))
	}
}

fn get_input(args: &Vec<String>) -> Result<String> {
	match args.len() {
		1 => read_from_stdin(),
		2 => {
			if args[1] == "-s" {				
				return Err(Box::new(InputError));
			}
			let infile = fs::File::open(args[1].clone())?;
			
			let mut buf_reader = BufReader::new(infile);
			let mut input = String::new();
			let result = buf_reader.read_to_string(&mut input);
			match result {
				Ok(_) => Ok(input),
				Err(error) => Err(Box::new(error)),
			}
		},
		3 => {
			if args[1] != "-s" {
				return Err(Box::new(InputError));
			}
			Ok(String::from(args[2].clone()))
		},
		_ => { 
			Err(Box::new(InputError))			
		},
	}
}
fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();
	
	let input = get_input(&args).unwrap_or_else(|error| {
		eprintln!("{error}");
		panic!("{error:?}");
	});
	
	let mut hash: [u32; 8] = [H0, H1, H2, H3, H4, H5, H6, H7];
	
	let padded: Vec<u8> = make_padded_vec(&input);

	hash_loop(padded, &mut hash);
	
	println!("{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}", 
		hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]);

	Ok(())
}
