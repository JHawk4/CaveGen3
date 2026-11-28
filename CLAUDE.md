# CaveGen3: notes for agents

Read `README.md` first. It covers what the project does, the web page, the CLI, the repo layout
and what each test checks. This file only adds what isn't there.

## Rules

- This repo and its site are public. Never add personal details about the maintainer: no real
  name or surname, emails or local paths, in code, docs, commit messages or data. Credit
  CaveGen as "JHawk's CaveGen", linking https://github.com/JHawk4/CaveGen.
- Commit and push to `main` when a piece of work is done and the tests pass; pushing to `main`
  deploys the site (`.github/workflows/pages.yml`). Creating repos and changing a repo's
  visibility or settings need the maintainer's OK.
- Stable Rust only (`rust-toolchain.toml`).
- Don't run `cargo fmt` on the whole workspace: the code isn't rustfmt-clean, and it would
  rewrite every file, `caveripper-ref` included. Format only what you touch, by hand
  (`rustfmt.toml`: 140 columns).
- Generation must keep matching CaveGen (`testdata/`) and Caveripper (`crates/caveripper-ref`,
  kept only as a test reference). Check behavior questions against the pikmin2 decomp
  (github.com/projectPiki/pikmin2) and CaveGen's Java source, not guesses.
- Keep the README's tables and feature lists current with each change.

## Where things are

- `crates/cavegen3`: data parsing (`parse.rs`, `data.rs`), `caveinfo.rs`, `generate.rs`,
  `layout.rs` (`Layout`, spawn order, `count`), waypoints, judges, queries, search.
- `crates/cavegen3-web`: the JSON API shared by the site and the CLI. `Web::query(&self)` takes
  judge, search and caveinfo commands and is thread-safe; `Web::command(&mut self)` adds the
  aggregator (`aggregate.rs`). The wasm exports are raw `extern "C"` functions (no wasm-bindgen).
- `crates/cavegen3-cli`: the `cavegen3` binary (`main.rs`, clap). `render.rs` draws everything
  with tiny-skia, mirroring `web/app.js`'s canvas drawing. When you change a map option, change
  both.
- `web/`: plain HTML, CSS and JS, no build step. `tools/build_site.py` assembles `site/` (the
  wasm, data bundles, icons, pretty tiles).
- Sublevel names: `BK-4`, `251:AT-2`, and Colossal variants as suffixes
  (`colossal+ultra+extra+5onion`).
- Data tools (Python): `tools/import_cavegen.py`, `import_game_files.py`, `extract_names.py`,
  `check_data.py` (runs in CI).
- Pretty view images are in `data/resources/pretty/*.webp`. They are made from CaveGen's pictures by
  `cargo run --release -p cavegen3-cli --example clean_pretty` (tiles) and
  `--example pretty_backdrops` (toy and grass floor backdrops); `pretty_profile` is a
  diagnostic for tuning them. Convert the PNGs with ffmpeg as the examples' doc comments say.

## Building and checking

- `cargo test --release`: everything; the generator tests are heavy, so run in release.
  Narrow with `-p cavegen3-cli` or `--test cavegen_expect` while iterating.
- The site, locally:
  ```
  cargo build --release -p cavegen3-web --target wasm32-unknown-unknown
  python tools/build_site.py site
  ```
  Then serve `site/` with `python -m http.server -d site` (`.claude/launch.json` has this as
  the `site` preview, through the Windows `py` launcher).
  Check UI changes in a browser, not only by reading the code: CSS can override `hidden`, for
  example (`style.css` has a `[hidden]{display:none !important}` rule for that).
- PNG output: `target/release/cavegen3 <sublevel> <seed> --show ... -o <dir>`; look at the result.
- On Windows Git Bash, `python` may be an old version; use `py -3.13`.

## Gotchas

- Seeds with `--seeds g` (game RNG) must continue the RNG across parallel chunks
  (`slice_start` in the CLI, `nextChunkStart` in JS); a test covers it.
- `all_units251.txt` has two units missing their closing `}`; the parser treats a bare `{` as a
  new section on purpose. Lands of Torture has the opposite (two sections with no `{`); the game
  treats braces as whitespace, so `parse::sections` reads text that a `}` closes as a section.
- The game reads cave files as token streams, and romhacks lean on it. layout.txt is read as
  tokens (count, then 8 per spawn point), and a caveinfo floor is its FloorInfo plus the
  sections after it in order (Hallow's Haunt has a floor with no ItemInfo and one with two
  CapInfos). Numbers are read like sscanf: the longest prefix that parses ("3-20.05" is 3).
- Treasure order matters for spawning: keep caveinfo order wherever treasures are listed.
- 12 romhack sublevels once differed between CaveGen and Caveripper; 10 were CaveGen3 bugs, fixed
  against the decomp. New Year CH-1, CH-2 and CH-4 remain different, because CaveGen treats the short
  name "CH" as challenge mode. They're listed in `crates/cavegen3/tests/cavegen_expect.rs`.
