# Adding a romhack

CaveGen3 reads each game from its own folder under `data/assets/`. Adding a romhack means
putting its files there in the layout below and listing its caves in
`data/resources/caveinfo_config.txt`. No code changes are needed.

Before adding a romhack to this repository, get the romhack author's permission to
redistribute its files.

## What's needed

```
data/assets/<game>/
  caveinfo/<file>.txt            one file per cave, from user/Mukki/mapunits/caveinfo
  unitfiles/<file>.txt           from user/Mukki/mapunits/units
  mapunits/<unit>/texts/         from user/Mukki/mapunits/arc/<unit>.szs:
    layout.txt                     spawn points (rooms only)
    route.txt                      waypoints
    waterbox.txt                   water
  mapunits/<unit>/arc/texture.png  the unit's radar image, from arc.szs/texture.bti
  teki/<name>.png                optional: enemy icons, from user/Yamashita/enemytex/arc.szs
  treasures/<name>.png           optional: treasure icons, from user/Matoba/resulttex/us/arc.szs
  otakara_config.txt             optional: treasure weights and values,
  item_config.txt                  from user/Abe/Pellet/us/pelletlist_us.szs
```

`<game>` is a short lowercase name for the romhack, such as `251`. Map unit folder names are the
names of the `.szs` archives in `user/Mukki/mapunits/arc`. Anything optional that's missing
falls back to the vanilla game's (`pikmin2`) version.

## Getting the files

You need a built ISO of the romhack: most romhacks ship as a patch, and the patch alone
usually doesn't contain every file.

Pick one of these:

- **From Caveripper.** `caveripper extract <romhack.iso> <game>` writes exactly this layout
  to `~/.config/caveripper/assets/<game>` (Caveripper's README covers installation). Copy
  that folder to `data/assets/<game>`.
- **From the game's own files.** If you have the romhack's files unpacked (an ISO extracted
  with Dolphin, or a romhack that ships replacement files, like Pikmin 216's
  `installer/patch` folder), point this at the folder that holds `user/`:

  ```
  python tools/import_game_files.py <folder> <game> [--base pikmin2]
  ```

  It unpacks the `.szs` archives and decodes the textures with Wiimms SZS Tools (`wszst` and
  `wimgt`, which must be on PATH). `--base pikmin2` copies in vanilla map units the romhack
  uses but doesn't include.

  Bikmin 269 ships as an xdelta patch, which only applies cleanly to the full-size retail
  Pikmin 2 USA dump (1,459,978,240 bytes; convert an `.rvz` with `DolphinTool convert -f iso`).
  Trimmed or rebuilt ISOs move the files, and the patched ISO comes out scrambled.
- **From a CaveGen folder.** If the romhack is already set up for JHawk's CaveGen (see its
  `Adding_New_Romhacks.md`, which covers extracting an ISO with Dolphin and Wiimms SZS Tools),
  convert CaveGen's folder:

  ```
  python tools/import_cavegen.py <CaveGen/files/romhack> <game>
  ```

  This copies `caveinfo`, `units`, `arc`, `enemytex` and `resulttex` into place and renames
  caveinfo files to the case `caveinfo_config.txt` uses. Add the caves to
  `caveinfo_config.txt` first (next section) so the names line up.

The text files are Shift-JIS, as in the game. Don't re-save them as UTF-8.

Then check that every file the data refers to exists with exactly that name:

```
python tools/check_data.py --fix <game>
```

The game ignores case in file names, and romhacks sometimes rely on that (a unit file that
says `SR7_racetrack_toy.txt` for `SR7_Racetrack_toy.txt`). CaveGen3 ignores case too, but
`--fix` renames such files so the data also works with other tools on Linux.

## Listing the caves

Add one line per cave to `data/resources/caveinfo_config.txt`:

```
<game>, <Full Cave Name>, <is challenge mode>, <caveinfo file>, <short name>[, <alias>...]
```

For example:

```
251,        Abandoned Toybox,      false, Abandoned_Toybox.txt,             AT
251,        Frigid Cavern,         true,  ch1_snow_tutorial.txt,            CH1, fgc
```

- Story-mode caves list their story abbreviation first (`AT`). Challenge-mode caves list
  `CH<n>` first, then any short names.
- The caveinfo file name must match the file's case exactly; the web build and Linux are
  case sensitive.
- Short names only need to be unique within the game. Sublevels are written `<short name><floor>`
  (`AT3`), `CH<n>-<floor>` for challenge mode, and `<game>:<name>` when the name is also used
  by another game (`251:SC2`).

## Checking it

```
cargo test --release -p cavegen3 --test parse_vs_reference --test vs_reference
```

Both tests cover every game in `data/assets` that has its caves listed. They check that
CaveGen3 parses and generates the romhack the same way Caveripper does. The web page picks up
new games automatically (see the README for building it locally).

For a stronger check, compare against CaveGen. With the romhack set up in CaveGen, generate
its expected output (CaveGen writes it to `files/expected_output_this.txt` when the
`EXPECTOUT` environment variable is set):

```
EXPECTOUT=1 java -cp <CaveGen> CaveGen none both -num 20 -seed 0 -consecutiveSeeds -<flag> -noImages -noPrints -noStats
```

Gzip it into `testdata/cavegen_expected_<game>.txt.gz` and add a test next to
`matches_cavegen_251_output` in `crates/cavegen3/tests/cavegen_expect.rs`.

## Known gaps

- **New Year's Timeless Realm (CH28)**: its third floor names `3_units_boss_time.txt`, which
  isn't in the hack's files, so the cave can't be generated.
- A few Pikmin 251 and New Year sublevels generate differently in CaveGen and Caveripper;
  CaveGen3 follows Caveripper. They're listed in `crates/cavegen3/tests/cavegen_expect.rs`.
