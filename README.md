# Autogram

This is a simple command line tool to produce autograms, which are sentences that describe their inventory of
characters. Take this example from [Wikipedia](https://en.wikipedia.org/wiki/Autogram):

> This sentence employs two a’s, two c’s, two d’s, twenty-eight e’s, five f’s, three g’s, eight h’s, eleven i’s, three l’s, two m’s, thirteen n’s, nine o’s, two p’s, five r’s, twenty-five s’s, twenty-three t’s, six v’s, ten w’s, two x’s, five y’s, and one z.

The search process is completely deterministic, so it is guaranteed to find all possible solutions (with counts up to 99
at least).

## Inspiration

In the mid- to late-nineties, I found [this site](https://selfreferentialsentences.blogspot.com/), which listed hundreds of
autograms (thanks to Matthias Belz for helping me find it again). Although I didn't find it again until after I wrote
this library, I did decide to try my hand at a solution just for fun. This library will find all possible solutions with
counts up to 99, and while it is extremely slow, it should find something in a day or two. I also did this partly to learn Rust better.

## Installation

You will need a [Rust](https://www.rust-lang.org/en-US/) compiler installed.

```
git clone https://github.com/jimrybarski/autogram.git
cd autogram
cargo build --release
```

## Usage

Simple invoke the binary with the "preamble" (the fixed part of the sentence at the beginning) as space-separated
arguments.

`./target/release/autogram this sentence has`

This is currently quite slow - the only time I've gotten it to produce a result, it took around 12 hours on a four-core
i5 laptop. I suspect that for any given preamble, there are hundreds of solutions.

## Bar Trivia Team Name

This is the one solution I've found. Sadly, we did not win the prize for best team name:

> This bar trivia team name has seven a's, two b's, one c, two d's, thirty-two e's, five f's, one g, five h's, ten i's, one j, one k, two l's, three m's, twenty-two n's, seventeen o's, one p, one q, five r's, twenty-two s's, twenty-two t's, one u, nine v's, eleven w's, one x, five y's and one z.
