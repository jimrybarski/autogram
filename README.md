# Autogram

This is a simple command line tool to produce autograms, which are sentences that describe their inventory of
characters. Take this example from [Wikipedia](https://en.wikipedia.org/wiki/Autogram):

> This sentence employs two a’s, two c’s, two d’s, twenty-eight e’s, five f’s, three g’s, eight h’s, eleven i’s, three l’s, two m’s, thirteen n’s, nine o’s, two p’s, five r’s, twenty-five s’s, twenty-three t’s, six v’s, ten w’s, two x’s, five y’s, and one z.

## Inspiration

In the mid- to late-nineties, I remember finding a website where dozens of autograms were listed, with hundreds of
solutions to each. Most notably, it claimed that the list contained *all possible solutions*. Two decades later, it
occurred to me that it would be a fun problem to solve. I can't find that original website anymore, but if you happen to
be the author, PLEASE reach out. I'd love to know how your solution worked. Especially since it must have been radically
more performant given how slowly my own program runs, and how much slower computers were back then.

I also did this partly to learn Rust better.

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

This is currently quite slow - the only time I've gotten it to produce a result, it took around 12 hours. I
suspect that for any given preamble, there are hundreds of solutions. However, unlike genetic algorithms or random
searches, this is guaranteed to find all possible solutions (with counts up to 99 at least).

## Bar Trivia Team Name

This is the one solution I've found. Sadly, we did not win the prize for best team name:

> This bar trivia team name has seven a's, two b's, one c, two d's, thirty-two e's, five f's, one g, five h's, ten i's, one j, one k, two l's, three m's, twenty-two n's, seventeen o's, one p, one q, five r's, twenty-two s's, twenty-two t's, one u, nine v's, eleven w's, one x, five y's and one z.
