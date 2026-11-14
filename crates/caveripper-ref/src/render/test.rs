use paste::paste;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{assets::fs_asset_manager::FsAssetManager, layout::Layout, render::*};

macro_rules! test_render {
    ($($name: literal),+) => {
        paste!{
            $(
                #[test]
                fn [<test_render_layouts_ $name>] () {
                    let mgr = FsAssetManager::init().unwrap();
                    let helper = RenderHelper::new(&mgr);

                    let num_layouts = 4;
                    let caveinfos = mgr.caveinfos_from_cave($name.replace('_', ":").as_str()).unwrap();

                    caveinfos.into_par_iter().panic_fuse().for_each(|caveinfo| {
                        let mut rng: SmallRng = SeedableRng::seed_from_u64(0x12345678);
                        for _ in 0..num_layouts {
                            let seed: u32 = rng.r#gen();
                            let layout = Layout::generate(seed, &caveinfo);
                            if let Err(cause) = render_layout(&layout, &helper, LayoutRenderOptions::default()) {
                                panic!("({}, {:#010X}) {}", $name, seed, cause);
                            }
                        }
                    });
                }

                #[test]
                fn [<test_render_caveinfo_ $name>] () {
                    let mgr = FsAssetManager::init().unwrap();
                    let helper = RenderHelper::new(&mgr);
                    let caveinfos = mgr.caveinfos_from_cave($name.replace('_', ":").as_str()).unwrap();
                    caveinfos.into_par_iter().panic_fuse().for_each(|caveinfo| {
                        render_caveinfo(&caveinfo, &helper, CaveinfoRenderOptions::default()).unwrap();
                    });
                }
            )*
        }
    }
}

test_render!(
    "ec",
    "scx",
    "fc",
    "hob",
    "wfg",
    "bk",
    "sh",
    "cos",
    "gk",
    "sr",
    "smc",
    "coc",
    "hoh",
    "dd",
    "exc",
    "nt",
    "ltb",
    "cg",
    "gh",
    "hh",
    "ba",
    "rc",
    "tg",
    "twg",
    "cc",
    "cm",
    "cr",
    "dn",
    "ca",
    "sp",
    "tct",
    "ht",
    "csn",
    "gb",
    "rg",
    "sl",
    "hg",
    "ad",
    "str",
    "bg",
    "cop",
    "bd",
    "snr",
    "er"
    // CaveGen3: Pikmin 251, New Year, 216 and Bikmin are left out. 216 and Bikmin aren't
    // included, and the others' data comes from CaveGen, which has no
    // treasure configs, so this renderer can't look up their custom treasures.
);
