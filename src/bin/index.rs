use clap::Parser;
use std::{path::PathBuf, fs::File, io::{BufReader, BufRead, Write}, convert::TryInto};

#[derive(Debug,Parser)]
struct Args
{
    /// Input file from which to read nicknames
    #[clap(short,long)]
    input: PathBuf,

    /// Output file in which to write nicknames and indices
    #[clap(short,long)]
    output: PathBuf,

    /// Hash key
    #[clap(short,long)]
    key: String,
}

fn main()
{
    let args = Args::parse();

    let infile = File::open(args.input).unwrap();
    let mut outfile = File::create(args.output).unwrap();

    let key = base64::decode(args.key).unwrap();
    let key: [u8; 16] = key.try_into().unwrap();
    let hasher = magnetite::NickHasher::new(&key);

    let reader = BufReader::new(infile);

    for nick in reader.lines()
    {
        let nick = nick.unwrap();

        let index = hasher.index_for(&nick);

        outfile.write_fmt(format_args!("{} {}\n", index, nick)).unwrap();
    }
}