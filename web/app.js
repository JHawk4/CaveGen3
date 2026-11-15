// CaveGen3 web page: loads the WebAssembly generator and draws layouts on a canvas.
"use strict";

// ---- Text in both languages ----

const STRINGS = {
  en: {
    subtitle: "Pikmin 2 cave layouts for any seed, generated in your browser.",
    game: "Game",
    cave: "Cave",
    sublevel: "Sublevel",
    seed: "Seed",
    prevSeed: "Previous seed",
    nextSeed: "Next seed",
    seedTitle: "Up to 8 hex digits",
    random: "Random seed",
    generate: "Generate",
    water: "Water",
    zoom: "Zoom",
    fit: "Fit",
    copyLink: "Copy link",
    saveImage: "Save image",
    storyMode: "Story mode",
    challengeMode: "Challenge mode",
    layout: "Layout",
    unitsLine: (n, w, h) => `${n} map units, ${w}×${h} cells`,
    exitHole: (plugged) => `Exit: hole${plugged ? " (plugged)" : ""}`,
    exitGeyser: "Exit: geyser",
    noExit: "No exit (final floor)",
    treasures: (n) => `Treasures (${n})`,
    carriedBy: (name, teki) => `${name} (in ${teki})`,
    teki: (n) => `Teki (${n})`,
    gates: (n) => `Gates (${n})`,
    hp: (n) => `${n} HP`,
    none: "None",
    loading: "Loading…",
    loadingGame: (game) => `Loading ${game} data…`,
    generating: "Generating…",
    generated: (sub, seed, ms) => `${sub}, seed ${seed}. Generated in ${ms} ms.`,
    badSeed: "The seed should be up to 8 hex digits, like 1234ABCD.",
    cantGenerate: (msg) => `Couldn't generate this sublevel: ${msg}`,
    cantStart: (msg) => `Couldn't start: ${msg}. This page needs a recent browser.`,
    linkCopied: "Link copied.",
    floorName: (cave, floor) => `${cave} ${floor}`,
    value: (v) => `${v}`,
    pokos: (v) => `${v} pokos`,
    gate: "Gate",
    ship: "Research Pod",
    hole: "Hole",
    plugged: "plugged",
    geyser: "Geyser",
    onion: "Onion",
    falling: "falls when approached",
    carrying: (t) => `carries ${t}`,
    crashed: "the generator hit a bug and was restarted (please report this sublevel and seed)",
    navGenerator: "Generator",
    navAggregator: "Aggregator",
    navSuggest: "Suggestion box",
    pageTitle: { generator: "CaveGen3", aggregator: "Aggregator · CaveGen3", suggest: "Suggestion box · CaveGen3" },
    comedown: "Comedown only",
    comedownTitle: "Show only what's around the pod when you land",
    comedownSize: "Comedown size (cells)",
    zoomIn: "Zoom in",
    zoomOut: "Zoom out",
    zoomFit: "Fit the map to the window",
    zoomHelp: "Scroll or pinch to zoom, drag to move, double-click to zoom in.",
    suggestTitle: "Suggestion box",
    suggestIntro: "Found a wrong layout, a missing romhack, or have an idea for CaveGen3? Leave a note. Your name is optional.",
    suggestName: "Name (optional)",
    suggestNamePlaceholder: "e.g. your Discord or speedrun.com name",
    suggestMessage: "Suggestion",
    suggestContact: "Contact (optional)",
    suggestContactPlaceholder: "Discord username, so we can follow up",
    suggestSend: "Send",
    suggestSending: "Sending…",
    suggestSent: "Thanks! Your suggestion was sent.",
    suggestFail: "Sorry, it couldn't be sent. Please try again later.",
    suggestBack: "Back to CaveGen3",
    advanced: "Advanced options",
    showOnMap: "Show on the map",
    opt: {
      treasures: "Treasures",
      teki: "Teki",
      plants: "Plants",
      gates: "Gates",
      exits: "Pod and exits",
      spawnpoints: "Spawn points",
      angles: "Facing directions",
      unitScores: "Unit scores",
      spawnScores: "Treasure and hole scores",
      tekiScores: "Teki scores",
      doorScores: "Door scores",
      doorLinks: "Door links",
      doorIds: "Door numbers",
      waypoints: "Waypoints",
      paths: "Carry paths",
      gauge: "Treasure gauge rings",
      grid: "Grid",
      comedownBox: "Comedown box outline",
    },
    judge: "Judge",
    judgeNone: "None",
    judgeHelp: "CaveGen's layout judges. Lower is better; most estimate seconds for a kind of run.",
    judgeNames: {
      score: "score: game's own treasure scores",
      pod: "pod: Pay off Debt",
      at: "at: All Treasures",
      attk: "attk: Challenge Mode score attack",
      cmat: "cmat: Challenge Mode all treasures",
      key: "key: Challenge Mode key and exit",
      colossal: "colossal: Colossal Caverns",
      mapunitcount: "mapunitcount: number of map units",
    },
    judgeLine: (type, score) => `Judge (${type}): ${score}`,
    search: "Seed search",
    searchHelp: "Tests many seeds of this sublevel and lists the best by the judge above, or the ones matching a query.",
    searchCount: "Seeds to test",
    searchMode: "Seeds",
    consecutive: "Counting up from the seed",
    randomSeeds: "Random",
    query: "Query (optional)",
    queryPlaceholder: "e.g. hole carry dist < 1000",
    queryHelp:
      'Uses <a href="https://github.com/mayabyte/caveripper/blob/main/QUERY.md">Caveripper\'s query language</a>. ' +
      "The sublevel can be left out.",
    keep: "Results",
    searchButton: "Search",
    stop: "Stop",
    searching: (done, total) => `Searching… ${done.toLocaleString()} of ${total.toLocaleString()} seeds`,
    searchDone: (tested, matched, s) =>
      `Tested ${tested.toLocaleString()} seeds in ${s} s` + (matched === null ? "." : `; ${matched.toLocaleString()} matched.`),
    searchStopped: (tested) => `Stopped after ${tested.toLocaleString()} seeds.`,
    noMatches: "No seeds matched.",
    needJudgeOrQuery: "Pick a judge or write a query first.",
    rank: (pct) => `top ${pct}%`,
    caveInfo: "CaveInfo",
    caveInfoHelp: "What this sublevel can generate: its limits, and the weights the generator picks from.",
    floorInfo: "Floor",
    maxTeki: "Max teki",
    maxTreasures: "Max treasures",
    maxGates: "Max gates",
    rooms: "Rooms",
    corridorProb: "Corridor chance",
    capProb: "Alcove chance",
    hasGeyser: "Geyser",
    pluggedHole: "Plugged hole",
    finalFloor: "Final floor",
    waterwraith: "Waterwraith timer",
    yes: "Yes",
    no: "No",
    colName: "Name",
    colMin: "Min",
    colWeight: "Weight",
    colGroup: "Group",
    colValue: "Value",
    colHealth: "HP",
    colSize: "Size",
    colDoors: "Doors",
    colType: "Type",
    tekiInfo: "Teki",
    itemInfo: "Treasures",
    gateInfo: "Gates",
    capInfo: "Alcove teki",
    units: "Map units",
    groupNames: { 0: "0 easy", 1: "1 hard", 5: "5 seam", 6: "6 plant", 8: "8 special" },
    unitTypes: { room: "room", hallway: "hallway", cap: "alcove" },
    footer:
      'CaveGen3\'s layouts are checked against <a href="https://github.com/JHawk4/CaveGen">CaveGen</a> and ' +
      '<a href="https://github.com/mayabyte/caveripper">Caveripper</a>. ' +
      '<a href="https://github.com/JHawk4/CaveGen3">Source</a>. ' +
      "Pikmin 2 is &copy; Nintendo; romhack data belongs to its authors.",
    otherLang: "日本語",
    otherLangLabel: "日本語で表示",
    gameNames: {
      pikmin2: "Pikmin 2",
      "251": "Pikmin 251",
      newyear: "Pikmin 2: New Year",
      colossal: "Colossal Caverns",
      "216": "Pikmin 2: 216",
      bikmin: "Bikmin",
    },
  },
  ja: {
    subtitle: "ピクミン2の洞窟のレイアウトを、好きなシードでブラウザ上で生成します。",
    game: "ゲーム",
    cave: "洞窟",
    sublevel: "階",
    seed: "シード",
    prevSeed: "前のシード",
    nextSeed: "次のシード",
    seedTitle: "16進数8桁まで",
    random: "ランダム",
    generate: "生成",
    water: "水",
    zoom: "表示",
    fit: "全体",
    copyLink: "リンクをコピー",
    saveImage: "画像を保存",
    storyMode: "ストーリーモード",
    challengeMode: "チャレンジモード",
    layout: "レイアウト",
    unitsLine: (n, w, h) => `マップユニット ${n}個（${w}×${h}マス）`,
    exitHole: (plugged) => `出口：穴${plugged ? "（ふさがっている）" : ""}`,
    exitGeyser: "出口：間欠泉",
    noExit: "出口なし（最下層）",
    treasures: (n) => `お宝（${n}）`,
    carriedBy: (name, teki) => `${name}（${teki}が持っている）`,
    teki: (n) => `原生生物（${n}）`,
    gates: (n) => `ゲート（${n}）`,
    hp: (n) => `体力 ${n}`,
    none: "なし",
    loading: "読み込み中…",
    loadingGame: (game) => `${game}のデータを読み込み中…`,
    generating: "生成中…",
    generated: (sub, seed, ms) => `${sub}、シード ${seed}。${ms}ミリ秒で生成しました。`,
    badSeed: "シードは 1234ABCD のような16進数8桁までで入力してください。",
    cantGenerate: (msg) => `この階は生成できませんでした：${msg}`,
    cantStart: (msg) => `起動できませんでした：${msg}。新しいブラウザでお試しください。`,
    linkCopied: "リンクをコピーしました。",
    floorName: (cave, floor) => `${cave} 地下${floor}階`,
    value: (v) => `${v}`,
    pokos: (v) => `${v}ポコ`,
    gate: "ゲート",
    ship: "ポッド",
    hole: "穴",
    plugged: "ふさがっている",
    geyser: "間欠泉",
    onion: "オニヨン",
    falling: "近づくと落ちてくる",
    carrying: (t) => `${t}を持っている`,
    crashed: "生成プログラムの不具合で再起動しました（この階とシードを報告してください）",
    navGenerator: "生成",
    navAggregator: "集計",
    navSuggest: "ご意見箱",
    pageTitle: { generator: "CaveGen3", aggregator: "集計 · CaveGen3", suggest: "ご意見箱 · CaveGen3" },
    comedown: "着地時の範囲だけ",
    comedownTitle: "着地したときにポッドの周りに見える範囲だけを表示します",
    comedownSize: "着地時の範囲の広さ（マス）",
    zoomIn: "拡大",
    zoomOut: "縮小",
    zoomFit: "ウィンドウに合わせる",
    zoomHelp: "ホイールかピンチで拡大・縮小、ドラッグで移動、ダブルクリックで拡大。",
    suggestTitle: "ご意見箱",
    suggestIntro: "レイアウトの誤り、未対応の改造版、CaveGen3への要望などがあればお知らせください。名前は任意です。",
    suggestName: "名前（任意）",
    suggestNamePlaceholder: "例：Discordやspeedrun.comの名前",
    suggestMessage: "ご意見",
    suggestContact: "連絡先（任意）",
    suggestContactPlaceholder: "Discordのユーザー名など（返信用）",
    suggestSend: "送信",
    suggestSending: "送信中…",
    suggestSent: "ありがとうございます。送信しました。",
    suggestFail: "送信できませんでした。時間をおいて再度お試しください。",
    suggestBack: "CaveGen3に戻る",
    advanced: "詳細オプション",
    showOnMap: "マップに表示",
    opt: {
      treasures: "お宝",
      teki: "原生生物",
      plants: "植物",
      gates: "ゲート",
      exits: "ポッドと出口",
      spawnpoints: "出現ポイント",
      angles: "向き",
      unitScores: "ユニットのスコア",
      spawnScores: "お宝と穴のスコア",
      tekiScores: "原生生物のスコア",
      doorScores: "ドアのスコア",
      doorLinks: "ドアのつながり",
      doorIds: "ドア番号",
      waypoints: "ウェイポイント",
      paths: "運搬ルート",
      gauge: "お宝ゲージの範囲",
      grid: "グリッド",
      comedownBox: "着地時の範囲の枠",
    },
    judge: "評価",
    judgeNone: "なし",
    judgeHelp: "CaveGenのレイアウト評価。小さいほど良く、多くは走り方ごとの秒数の見積もりです。",
    judgeNames: {
      score: "score：ゲーム内のお宝スコア",
      pod: "pod：借金返済",
      at: "at：お宝全回収",
      attk: "attk：チャレンジモードのスコアアタック",
      cmat: "cmat：チャレンジモードのお宝全回収",
      key: "key：チャレンジモードのカギと出口",
      colossal: "colossal：Colossal Caverns",
      mapunitcount: "mapunitcount：マップユニットの数",
    },
    judgeLine: (type, score) => `評価（${type}）：${score}`,
    search: "シード検索",
    searchHelp: "この階のシードをたくさん試して、上の評価で良いものや、条件に合うものを一覧にします。",
    searchCount: "試すシードの数",
    searchMode: "シード",
    consecutive: "上のシードから順に",
    randomSeeds: "ランダム",
    query: "条件（任意）",
    queryPlaceholder: "例：hole carry dist < 1000",
    queryHelp:
      '<a href="https://github.com/mayabyte/caveripper/blob/main/QUERY.md">Caveripperの検索条件の書き方</a>（英語）を使います。' +
      "階の名前は省略できます。",
    keep: "表示件数",
    searchButton: "検索",
    stop: "停止",
    searching: (done, total) => `検索中… ${done.toLocaleString()} / ${total.toLocaleString()} シード`,
    searchDone: (tested, matched, s) =>
      `${tested.toLocaleString()} シードを${s}秒で調べ` +
      (matched === null ? "ました。" : `、${matched.toLocaleString()} 件が条件に合いました。`),
    searchStopped: (tested) => `${tested.toLocaleString()} シードで停止しました。`,
    noMatches: "条件に合うシードはありませんでした。",
    needJudgeOrQuery: "先に評価を選ぶか、条件を入力してください。",
    rank: (pct) => `上位 ${pct}%`,
    caveInfo: "CaveInfo",
    caveInfoHelp: "この階で生成されうるもの：上限と、生成時に選ばれる重み。",
    floorInfo: "階の設定",
    maxTeki: "原生生物の上限",
    maxTreasures: "お宝の上限",
    maxGates: "ゲートの上限",
    rooms: "部屋の数",
    corridorProb: "通路の確率",
    capProb: "行き止まりの確率",
    hasGeyser: "間欠泉",
    pluggedHole: "ふさがった穴",
    finalFloor: "最下層",
    waterwraith: "アメボウズが落ちてくるまで",
    yes: "あり",
    no: "なし",
    colName: "名前",
    colMin: "最低数",
    colWeight: "重み",
    colGroup: "グループ",
    colValue: "価値",
    colHealth: "体力",
    colSize: "大きさ",
    colDoors: "ドア",
    colType: "種類",
    tekiInfo: "原生生物",
    itemInfo: "お宝",
    gateInfo: "ゲート",
    capInfo: "行き止まりの原生生物",
    units: "マップユニット",
    groupNames: { 0: "0 弱", 1: "1 強", 5: "5 境目", 6: "6 植物", 8: "8 特殊" },
    unitTypes: { room: "部屋", hallway: "通路", cap: "行き止まり" },
    footer:
      'CaveGen3のレイアウトは <a href="https://github.com/JHawk4/CaveGen">CaveGen</a> と ' +
      '<a href="https://github.com/mayabyte/caveripper">Caveripper</a> で検証しています。' +
      '<a href="https://github.com/JHawk4/CaveGen3">ソースコード</a>。' +
      "ピクミン2 &copy; Nintendo。改造版のデータは各作者に帰属します。",
    otherLang: "English",
    otherLangLabel: "Show in English",
    gameNames: {
      pikmin2: "ピクミン2",
      "251": "Pikmin 251",
      newyear: "Pikmin 2: New Year",
      colossal: "Colossal Caverns",
      "216": "Pikmin 2: 216",
      bikmin: "Bikmin",
    },
  },
};

// Not Object.keys(gameNames): JavaScript lists number-like keys such as "251" first.
const GAME_ORDER = ["pikmin2", "251", "newyear", "colossal", "216", "bikmin"];

// Teki whose icons aren't in the game's enemy icon set (same list as Caveripper).
const SPECIAL_TEKI = {
  gashiba: "Gas_pipe_icon",
  daiodogreen: "daiodogreen",
  ooinu_s: "ooinu_s",
  kareooinu_s: "kareooinu_s",
  kareooinu_l: "kareooinu",
  elechiba: "Electrical_wire_icon",
  hiba: "Fire_geyser_icon",
  bomb: "Bingo_Battle_Bomb_icon",
  egg: "36px-Egg_icon",
  wakame_s: "wakame_s",
  chiyogami: "chiyogami",
  rock: "Roulette_Wheel_boulder",
  panhouse: "ooinu_s",
};

const CELL = 40; // pixels per map grid cell at 100% zoom
const UNIT = 170; // game units per grid cell

const $ = (id) => document.getElementById(id);
const params = new URLSearchParams(location.search);
if (params.has("embed")) document.body.classList.add("embed");

let lang = pickLanguage();
let T = STRINGS[lang];
let names = { treasures: {}, teki: {}, caves: {} };
let wasm = null;
let caves = [];
const loadedGames = new Set();
let current = null; // the last generated layout
let hitTargets = [];
let mapDragged = () => false; // set by attachZoom
let languageHook = null; // a page's own redraw after the language changes
const page = document.body.dataset.page || "generator"; // icons on the map, for tooltips: {x, z, r, label}

function pickLanguage() {
  const fromUrl = params.get("lang");
  if (fromUrl === "ja" || fromUrl === "en") return fromUrl;
  try {
    const saved = localStorage.getItem("cavegen3-lang");
    if (saved === "ja" || saved === "en") return saved;
  } catch {
    // Storage can be unavailable (private windows, embeds); fall through.
  }
  return (navigator.language || "").toLowerCase().startsWith("ja") ? "ja" : "en";
}

// ---- Names ----

/** The in-game name of a treasure or teki, or its internal name if we don't know it. */
function displayName(kind, internal) {
  const entry = names[kind][internal.toLowerCase()];
  return entry ? entry[lang] || entry.en : internal;
}

function caveName(cave) {
  const entry = cave.game === "pikmin2" && names.caves[cave.name];
  return entry ? entry[lang] : cave.name;
}

function gameName(game) {
  return T.gameNames[game] || game;
}

// ---- WebAssembly ----

let wasmModule = null;
const bundles = []; // every bundle given to the generator, to rebuild it after a crash

async function fetchGz(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`couldn't load ${url} (${res.status})`);
  const stream = res.body.pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function readResult(ptr) {
  const len = wasm.result_len();
  const text = new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, ptr, len));
  const value = JSON.parse(text);
  if (value && value.error) throw new Error(value.error);
  return value;
}

function withBytes(bytes, fn) {
  const ptr = wasm.alloc(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  try {
    return fn(ptr, bytes.length);
  } finally {
    wasm.dealloc(ptr, bytes.length);
  }
}

function addBundle(bytes) {
  withBytes(bytes, (p, n) => readResult(wasm.add_files(p, n)));
}

async function startWasm() {
  wasmModule = await WebAssembly.compileStreaming(fetch("cavegen3_web.wasm"));
  wasm = (await WebAssembly.instantiate(wasmModule, {})).exports;
}

/**
 * Runs a generator call. A bug in the generator stops its WebAssembly instance for good, so on a
 * crash this starts a fresh one with the same data and reports the error.
 */
async function guarded(fn) {
  try {
    return fn();
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      console.error(e);
      wasm = (await WebAssembly.instantiate(wasmModule, {})).exports;
      for (const b of bundles) addBundle(b);
      throw new Error(T.crashed);
    }
    throw e;
  }
}

async function loadGame(game) {
  if (loadedGames.has(game)) return;
  setStatus(T.loadingGame(gameName(game)));
  const bytes = await fetchGz(`data/${game}.bin.gz`);
  addBundle(bytes);
  bundles.push(bytes);
  loadedGames.add(game);
}

function generate(sublevel, seed, flags) {
  const name = new TextEncoder().encode(sublevel);
  return guarded(() => withBytes(name, (p, n) => readResult(wasm.generate(p, n, seed >>> 0, flags))));
}

/** A tab-separated command for the generator: judge, search or caveinfo. */
function command(...args) {
  const text = new TextEncoder().encode(args.join("\t"));
  return guarded(() => withBytes(text, (p, n) => readResult(wasm.command(p, n))));
}

// ---- Images ----

let available = null; // image paths that exist (data/images.json), so missing ones aren't fetched
const images = new Map(); // url -> Promise<Image or null>
const loaded = new Map(); // url -> Image, once loaded

function loadImage(url) {
  if (!url) return Promise.resolve(null);
  if (!images.has(url)) {
    images.set(
      url,
      new Promise((resolve) => {
        const img = new Image();
        img.onload = () => {
          loaded.set(url, img);
          resolve(img);
        };
        img.onerror = () => resolve(null);
        img.src = url;
      }),
    );
  }
  return images.get(url);
}

/** The first of `urls` that exists. */
function pickUrl(urls) {
  if (!available) return urls[0];
  return urls.find((u) => available.has(u)) || null;
}

async function firstImage(urls) {
  if (available) return loadImage(pickUrl(urls));
  for (const url of urls) {
    const img = await loadImage(url);
    if (img) return img;
  }
  return null;
}

function tekiIconUrls(name, game) {
  let n = name.toLowerCase();
  if (n === "fkabuto") n = "kabuto";
  if (n === "fminihoudai") n = "minihoudai";
  if (SPECIAL_TEKI[n]) return [specialUrl(SPECIAL_TEKI[n])];
  return [`img/${game}/teki/${n}.png`, `img/pikmin2/teki/${n}.png`];
}

function treasureIconUrls(name, game) {
  const n = name.toLowerCase();
  return [`img/${game}/treasures/${n}.png`, `img/pikmin2/treasures/${n}.png`];
}

function unitUrls(name, game) {
  const n = name.toLowerCase();
  return [`img/${game}/mapunits/${n}.png`, `img/pikmin2/mapunits/${n}.png`];
}

function specialUrl(name) {
  return `img/special/${name.toLowerCase()}.png`;
}

// ---- Drawing ----

// Radar images come in two encodings: Caveripper's extractor keeps the shading in the alpha
// channel too (gray v is stored as v,v,v,v), while CaveGen's (used for the romhacks) are opaque.
// Opaque ones are converted to the first kind, so every game's maps look the same.
const unitCanvases = new WeakMap();
function unitImage(img) {
  if (!unitCanvases.has(img)) {
    const c = document.createElement("canvas");
    c.width = img.width;
    c.height = img.height;
    const ctx = c.getContext("2d", { willReadFrequently: true });
    ctx.drawImage(img, 0, 0);
    const data = ctx.getImageData(0, 0, c.width, c.height);
    const d = data.data;
    let opaque = true;
    for (let i = 3; i < d.length; i += 4) {
      if (d[i] < 255) {
        opaque = false;
        break;
      }
    }
    if (opaque) {
      for (let i = 0; i < d.length; i += 4) d[i + 3] = d[i];
      ctx.putImageData(data, 0, 0);
    }
    unitCanvases.set(img, c);
  }
  return unitCanvases.get(img);
}

// ---- Zoom and the part of the map shown ----

let zoom = "fit"; // "fit", or a scale (1 = CELL pixels per grid cell)
let shownScale = 1; // the scale the map was last drawn at

/** The comedown box's width in cells: what you see around the pod when you land. */
function comedownCells() {
  return Number($("comedown-size")?.value) || 5;
}

/** The rectangle of the map to draw, in grid cells: the whole cave, or the comedown box. */
function mapView(layout) {
  const ship = layout.objects.find((o) => o.kind === "ship");
  if ($("comedown")?.checked && ship) {
    const half = comedownCells() / 2;
    return { x0: ship.x / UNIT - half, z0: ship.z / UNIT - half, w: half * 2, h: half * 2, comedown: true };
  }
  // A half-cell margin on each side.
  return { x0: -0.5, z0: -0.5, w: layout.width + 1, h: layout.height + 1, comedown: false };
}

/** The largest scale whose canvas the browser can still draw. */
function maxScale(view) {
  const dpr = window.devicePixelRatio || 1;
  return Math.min(6, 8000 / (Math.max(view.w, view.h) * CELL * dpr));
}

function fitScale(view, el) {
  const box = el.getBoundingClientRect();
  const availW = Math.max(200, box.width - 16);
  const availH = Math.max(200, window.innerHeight * 0.78);
  const s = Math.min(availW / (view.w * CELL), availH / (view.h * CELL), view.comedown ? 4 : 1.5);
  return Math.max(0.05, s);
}

function zoomScale(view) {
  const s = zoom === "fit" ? fitScale(view, $("map-scroll")) : zoom;
  return Math.min(s, maxScale(view));
}

/**
 * Mouse wheel, drag, double-click and pinch zoom for a scrolling map. `scale()` is the scale
 * last drawn at; `zoomTo(scale)` draws at a new one (returning a promise). The point under the
 * cursor (or between the fingers) stays put. Returns a function that tells whether the pointer
 * just dragged, so a click at the end of a drag can be ignored.
 */
function attachZoom(scroller, canvas, scale, zoomTo) {
  let busy = false;
  let pending = 1; // zoom factor waiting while a redraw runs
  let anchor = null;
  const zoomAt = async (factor, clientX, clientY) => {
    pending *= factor;
    anchor = { clientX, clientY };
    if (busy) return;
    busy = true;
    try {
      while (pending !== 1) {
        const f = pending;
        const at = anchor;
        pending = 1;
        const before = canvas.getBoundingClientRect();
        const old = scale();
        const px = (at.clientX - before.left) / old;
        const pz = (at.clientY - before.top) / old;
        await zoomTo(old * f);
        const now = scale();
        const after = canvas.getBoundingClientRect();
        scroller.scrollLeft += after.left + px * now - at.clientX;
        scroller.scrollTop += after.top + pz * now - at.clientY;
      }
    } finally {
      busy = false;
    }
  };
  scroller.addEventListener(
    "wheel",
    (e) => {
      if (e.ctrlKey && !e.deltaY) return;
      e.preventDefault();
      const unit = e.deltaMode === 1 ? 0.05 : e.deltaMode === 2 ? 1 : 0.0015;
      zoomAt(Math.exp(-e.deltaY * unit), e.clientX, e.clientY);
    },
    { passive: false },
  );
  canvas.addEventListener("dblclick", (e) => {
    e.preventDefault();
    zoomAt(e.shiftKey ? 0.5 : 2, e.clientX, e.clientY);
  });

  // Dragging pans; two fingers pinch-zoom.
  const pointers = new Map();
  let dragged = false;
  let start = null;
  let pinch = null;
  canvas.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    canvas.setPointerCapture(e.pointerId);
    dragged = false;
    start = { x: e.clientX, y: e.clientY, left: scroller.scrollLeft, top: scroller.scrollTop };
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      pinch = Math.hypot(a.x - b.x, a.y - b.y);
    }
  });
  canvas.addEventListener("pointermove", (e) => {
    if (!pointers.has(e.pointerId)) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.size === 2 && pinch) {
      const [a, b] = [...pointers.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      if (d > 0 && Math.abs(d / pinch - 1) > 0.04) {
        zoomAt(d / pinch, (a.x + b.x) / 2, (a.y + b.y) / 2);
        pinch = d;
      }
      dragged = true;
      return;
    }
    if (!start) return;
    const dx = e.clientX - start.x;
    const dy = e.clientY - start.y;
    if (Math.abs(dx) + Math.abs(dy) > 4) dragged = true;
    if (dragged) {
      scroller.scrollLeft = start.left - dx;
      scroller.scrollTop = start.top - dy;
      canvas.classList.add("dragging");
    }
  });
  const end = (e) => {
    pointers.delete(e.pointerId);
    if (pointers.size < 2) pinch = null;
    if (!pointers.size) start = null;
    canvas.classList.remove("dragging");
  };
  canvas.addEventListener("pointerup", end);
  canvas.addEventListener("pointercancel", end);
  return () => dragged;
}

/** The zoom buttons' label. */
function showZoomLabel() {
  const label = $("zoom-label");
  if (label) label.textContent = `${Math.round(shownScale * 100)}%`;
  $("zoom-fit")?.classList.toggle("on", zoom === "fit");
}

/** Tooltip text for an object on the map. */
function objectLabel(o) {
  switch (o.kind) {
    case "treasure": {
      const name = displayName("treasures", o.name);
      return o.value ? `${name} · ${T.pokos(o.value)}` : name;
    }
    case "teki": {
      const parts = [displayName("teki", o.name)];
      if (o.count > 1) parts[0] += ` ×${o.count}`;
      if (o.carrying) parts.push(T.carrying(displayName("treasures", o.carrying)));
      if (o.falling) parts.push(T.falling);
      return parts.join(" · ");
    }
    case "gate":
      return `${T.gate} · ${T.hp(Math.round(o.health))}`;
    case "hole":
      return o.plugged ? `${T.hole} · ${T.plugged}` : T.hole;
    case "geyser":
      return T.geyser;
    case "ship":
      return T.ship;
    case "onion":
      return T.onion;
  }
  return "";
}

// Map options. `details` ones need the generator's extra layout data.
const MAP_OPTIONS = [
  { key: "treasures", on: true },
  { key: "teki", on: true },
  { key: "plants", on: true },
  { key: "gates", on: true },
  { key: "exits", on: true },
  { key: "spawnpoints", details: true },
  { key: "angles", details: true },
  { key: "unitScores", details: true },
  { key: "spawnScores", details: true },
  { key: "tekiScores", details: true },
  { key: "doorScores", details: true },
  { key: "doorLinks", details: true },
  { key: "doorIds", details: true },
  { key: "waypoints", details: true },
  { key: "paths", details: true },
  { key: "gauge" },
  { key: "grid" },
  { key: "comedownBox" },
];

function mapOptions() {
  const o = { water: $("water").checked };
  for (const { key } of MAP_OPTIONS) o[key] = $(`opt-${key}`).checked;
  return o;
}

function needsDetails() {
  return MAP_OPTIONS.some((m) => m.details && $(`opt-${m.key}`).checked);
}

// CaveGen's colors for spawn point groups 0-9, and treasure gauge rings.
const SPAWN_COLORS = ["#ff82dc", "#dc0000", "#dc7800", "#000", "#008c3c", "#505050", "#8ce164", "#009696", "#a000a0", "#0000ff"];
const GAUGE_TICKS = [0.5, 1, 2, 3, 4, 5, 6, 7, 8];
const GAUGE_COLORS = ["#a0ffff", "#ffffa0", "#b4b4ff", "#ffb4b4", "#dca0ff", "#ffdc8c", "#a0d296", "#be9678", "#0000ff"];

function objectShown(o, opts) {
  switch (o.kind) {
    case "treasure":
      return opts.treasures;
    case "teki":
      return o.group === 6 ? opts.plants : opts.teki;
    case "gate":
      return opts.gates;
    default:
      return opts.exits;
  }
}

/** The image each map unit and object needs, so they can all load at once. */
function imagesFor(layout) {
  const urls = new Set([specialUrl("falling_icon")]);
  for (const u of layout.units) urls.add(pickUrl(unitUrls(u.name, u.game)));
  for (const o of layout.objects) {
    if (o.kind === "treasure") urls.add(pickUrl(treasureIconUrls(o.name, o.game)));
    if (o.kind === "teki") {
      urls.add(pickUrl(tekiIconUrls(o.name, o.game)));
      if (o.carrying) urls.add(pickUrl(treasureIconUrls(o.carrying, o.game)));
    }
  }
  for (const s of ["pod_icon", "Cave_icon", "36px-Clog_icon", "Geyser_icon", "Gray_bramble_gate_icon"]) urls.add(specialUrl(s));
  for (let i = 0; i < 6; i++) urls.add(specialUrl(`onion${i}`));
  urls.delete(null);
  return [...urls];
}

/** A loaded image for the first of `urls` that exists, or null. Call after `imagesFor` loads. */
function imageOf(urls) {
  if (available) return loaded.get(pickUrl(urls)) || null;
  for (const u of urls) if (loaded.has(u)) return loaded.get(u);
  return null;
}

async function render(layout) {
  const opts = mapOptions();
  if (available) await Promise.all(imagesFor(layout).map(loadImage));
  else await Promise.all(layout.units.map((u) => firstImage(unitUrls(u.name, u.game))));

  const canvas = $("map");
  const view = mapView(layout);
  const scale = zoomScale(view);
  shownScale = scale;
  showZoomLabel();
  const cell = CELL * scale;
  const dpr = window.devicePixelRatio || 1;
  const w = view.w * cell;
  const h = view.h * cell;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  canvas.style.width = `${Math.round(w)}px`;
  canvas.style.height = `${Math.round(h)}px`;
  hitTargets = [];
  const offX = -view.x0 * cell;
  const offZ = -view.z0 * cell;

  const ctx = canvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = "#0f0f0f";
  ctx.fillRect(0, 0, w, h);
  ctx.translate(offX, offZ);
  $("result").classList.toggle("comedown-view", view.comedown);

  const toPx = (v) => (v / UNIT) * cell;

  // Map units, rotated clockwise in quarter turns like the game's radar images.
  ctx.imageSmoothingEnabled = false;
  for (const u of layout.units) {
    const img = imageOf(unitUrls(u.name, u.game));
    const cx = (u.x + u.w / 2) * cell;
    const cz = (u.z + u.h / 2) * cell;
    ctx.save();
    ctx.translate(cx, cz);
    if (img) {
      ctx.rotate((u.rot * Math.PI) / 2);
      const [w0, h0] = u.rot % 2 ? [u.h, u.w] : [u.w, u.h];
      ctx.filter = "brightness(1.6)";
      ctx.drawImage(unitImage(img), (-w0 * cell) / 2, (-h0 * cell) / 2, w0 * cell, h0 * cell);
      ctx.filter = "none";
    } else {
      ctx.fillStyle = "#333";
      ctx.fillRect((-u.w * cell) / 2, (-u.h * cell) / 2, u.w * cell, u.h * cell);
    }
    ctx.restore();
  }

  if (opts.water) {
    ctx.fillStyle = "rgba(0, 100, 230, 0.25)";
    for (const u of layout.units) {
      const cx = (u.x + u.w / 2) * cell;
      const cz = (u.z + u.h / 2) * cell;
      for (const [x1, z1, x2, z2] of u.water) {
        ctx.fillRect(cx + toPx(x1), cz + toPx(z1), toPx(x2 - x1), toPx(z2 - z1));
      }
    }
  }

  if (opts.grid) {
    ctx.strokeStyle = "rgba(255, 255, 255, 0.12)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let x = 0; x <= layout.width; x++) {
      ctx.moveTo(x * cell, 0);
      ctx.lineTo(x * cell, layout.height * cell);
    }
    for (let z = 0; z <= layout.height; z++) {
      ctx.moveTo(0, z * cell);
      ctx.lineTo(layout.width * cell, z * cell);
    }
    ctx.stroke();
  }

  const label = (text, x, z, color, size = Math.max(9, cell * 0.28), align = "center") => {
    ctx.font = `bold ${size}px system-ui, sans-serif`;
    ctx.textAlign = align;
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = Math.max(2, size / 4);
    ctx.strokeStyle = "#000";
    ctx.strokeText(text, x, z);
    ctx.fillStyle = color;
    ctx.fillText(text, x, z);
  };
  const thin = Math.max(1, cell / 30);

  const carried = layout.objects.filter((o) => o.kind === "treasure" || (o.kind === "teki" && o.carrying));
  if (opts.gauge) {
    ctx.lineWidth = thin;
    for (const o of carried) {
      GAUGE_TICKS.forEach((tick, i) => {
        const r = (1 - Math.sqrt((tick - 0.5) / 9.5)) * 900;
        ctx.strokeStyle = GAUGE_COLORS[i];
        ctx.beginPath();
        ctx.arc(toPx(o.x), toPx(o.z), toPx(r), 0, Math.PI * 2);
        ctx.stroke();
      });
    }
  }

  const doors = layout.doors || [];
  if (opts.doorLinks) {
    ctx.strokeStyle = "rgba(255, 0, 255, 0.45)";
    ctx.lineWidth = thin;
    ctx.beginPath();
    doors.forEach((d) => {
      for (const j of d.links) {
        ctx.moveTo(toPx(d.x), toPx(d.z));
        ctx.lineTo(toPx(doors[j].x), toPx(doors[j].z));
      }
    });
    ctx.stroke();
  }

  if (opts.waypoints && layout.waypoints) {
    const wps = layout.waypoints;
    ctx.lineWidth = thin;
    ctx.strokeStyle = "rgba(90, 170, 255, 0.8)";
    ctx.fillStyle = "rgba(90, 170, 255, 0.15)";
    for (const wp of wps) {
      ctx.beginPath();
      ctx.arc(toPx(wp.x), toPx(wp.z), Math.max(2, toPx(wp.r)), 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    }
    ctx.strokeStyle = "rgba(120, 200, 255, 0.95)";
    for (const wp of wps) {
      if (wp.next === null) continue;
      const n = wps[wp.next];
      arrow(ctx, toPx(wp.x), toPx(wp.z), toPx(n.x), toPx(n.z), Math.max(4, cell * 0.12));
    }
  }

  if (opts.paths) {
    ctx.lineWidth = Math.max(1.5, cell / 18);
    ctx.strokeStyle = "rgba(255, 200, 40, 0.9)";
    for (const o of carried) {
      if (!o.path || o.path.length < 4) continue;
      ctx.beginPath();
      ctx.moveTo(toPx(o.path[0]), toPx(o.path[1]));
      for (let i = 2; i < o.path.length; i += 2) ctx.lineTo(toPx(o.path[i]), toPx(o.path[i + 1]));
      ctx.stroke();
    }
  }

  const spawnpoints = layout.spawnpoints || [];
  if (opts.spawnpoints) {
    for (const sp of spawnpoints) {
      const x = toPx(sp.x);
      const z = toPx(sp.z);
      ctx.globalAlpha = sp.used ? 1 : 0.7;
      if (sp.group === 0) {
        ctx.strokeStyle = SPAWN_COLORS[0];
        ctx.lineWidth = thin;
        ctx.beginPath();
        ctx.arc(x, z, Math.max(3, toPx(sp.r)), 0, Math.PI * 2);
        ctx.stroke();
      }
      ctx.fillStyle = SPAWN_COLORS[sp.group] || "#fff";
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(x, z, Math.max(2.5, cell * 0.09), 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      if (opts.angles) facing(ctx, x, z, sp.angle, cell * 0.35, "#fff");
    }
    ctx.globalAlpha = 1;
  }

  // Exits and the ship first, then gates, treasures and teki on top.
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  const drawIcon = (img, x, z, size, rot = 0) => {
    if (!img) return false;
    ctx.save();
    ctx.translate(x, z);
    if (rot) ctx.rotate(rot);
    const r = Math.min(size / img.width, size / img.height);
    ctx.drawImage(img, (-img.width * r) / 2, (-img.height * r) / 2, img.width * r, img.height * r);
    ctx.restore();
    return true;
  };
  const drawFallback = (text, x, z, size, color) => {
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(x, z, size / 2.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#fff";
    ctx.font = `bold ${Math.max(8, size / 3.2)}px system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(text.slice(0, 3), x, z);
  };
  const special = (name) => loaded.get(specialUrl(name)) || null;

  const order = { hole: 0, geyser: 0, ship: 1, onion: 1, gate: 2, treasure: 3, teki: 4 };
  const objects = layout.objects.filter((o) => objectShown(o, opts)).sort((a, b) => order[a.kind] - order[b.kind]);
  const falling = special("falling_icon");
  for (const o of objects) {
    const x = toPx(o.x);
    const z = toPx(o.z);
    let size = cell * 0.6;
    switch (o.kind) {
      case "ship":
        drawIcon(special("pod_icon"), x, z, size) || drawFallback("Pod", x, z, size, "#c00");
        break;
      case "hole":
        drawIcon(special("Cave_icon"), x, z, size);
        if (o.plugged) drawIcon(special("36px-Clog_icon"), x, z, size);
        break;
      case "geyser":
        size = cell * 0.7;
        drawIcon(special("Geyser_icon"), x, z, size);
        break;
      case "onion":
        size = cell * 0.45;
        drawIcon(special(`onion${o.color}`), x, z, size);
        break;
      case "gate": {
        size = cell * 0.85;
        drawIcon(special("Gray_bramble_gate_icon"), x, z, size, o.dir % 2 ? Math.PI / 2 : 0);
        // Zoomed far out (Colossal Caverns), the numbers would hide the map; the tooltip has them.
        if (cell >= 20) label(String(Math.round(o.health)), x - cell * 0.3, z + cell * 0.38, "#fff", Math.max(9, cell * 0.24), "left");
        break;
      }
      case "treasure": {
        size = cell * 0.55;
        drawIcon(imageOf(treasureIconUrls(o.name, o.game)), x, z, size) || drawFallback(o.name, x, z, size, "#c70");
        break;
      }
      case "teki": {
        size = o.group === 0 ? cell * 0.45 : cell * 0.6;
        drawIcon(imageOf(tekiIconUrls(o.name, o.game)), x, z, size) || drawFallback(o.name, x, z, size, "#7a2a8a");
        if (o.carrying) drawIcon(imageOf(treasureIconUrls(o.carrying, o.game)), x + size * 0.35, z + size * 0.35, size * 0.6);
        if (o.falling) drawIcon(falling, x - size * 0.4, z - size * 0.4, size * 0.4);
        if (o.count > 1) label(`×${o.count}`, x + size * 0.3, z - size * 0.35, "#fff", Math.max(9, size * 0.38), "left");
        break;
      }
    }
    if (opts.angles && o.angle !== undefined && o.kind !== "gate") facing(ctx, x, z, o.angle, cell * 0.4, "#ffe14d");
    const hx = x + offX;
    const hz = z + offZ;
    if (hx >= 0 && hz >= 0 && hx <= w && hz <= h) hitTargets.push({ x: hx, z: hz, r: Math.max(size / 2, 8), label: objectLabel(o) });
  }

  // Numbers on top of everything.
  if (opts.unitScores || opts.tekiScores) {
    for (const u of layout.units) {
      const cx = (u.x + u.w / 2) * cell;
      const cz = (u.z + u.h / 2) * cell;
      if (opts.unitScores && u.score >= 0) label(String(u.score), cx, cz, "#fff");
      if (opts.tekiScores && u.teki_score > 0) label(String(u.teki_score), cx, cz + cell * 0.3, "#ff5a5a");
    }
  }
  if (opts.spawnScores) {
    for (const sp of spawnpoints) {
      const x = toPx(sp.x) + cell * 0.15;
      const z = toPx(sp.z);
      if ((sp.group === 2 || sp.group === 9) && sp.treasure_score > -1) label(String(sp.treasure_score), x, z + cell * 0.12, "#ffaa00", undefined, "left");
      if ((sp.group === 4 || sp.group === 9) && sp.hole_score > -1) label(String(sp.hole_score), x, z - cell * 0.12, "#3cff3c", undefined, "left");
    }
  }
  if (opts.comedownBox && !view.comedown) {
    const ship = layout.objects.find((o) => o.kind === "ship");
    if (ship) {
      const half = (comedownCells() / 2) * cell;
      ctx.strokeStyle = "#ffe14d";
      ctx.lineWidth = Math.max(1.5, cell / 16);
      ctx.setLineDash([cell / 4, cell / 6]);
      ctx.strokeRect(toPx(ship.x) - half, toPx(ship.z) - half, half * 2, half * 2);
      ctx.setLineDash([]);
    }
  }
  if (opts.doorScores || opts.doorIds) {
    let unit = -1;
    let first = 0;
    doors.forEach((d, i) => {
      if (d.unit !== unit) {
        unit = d.unit;
        first = i;
      }
      // Nudge labels into the unit, away from the seam.
      const [dx, dz] = [[0, 1], [-1, 0], [0, -1], [1, 0]][d.dir];
      const x = toPx(d.x) + dx * cell * 0.22;
      const z = toPx(d.z) + dz * cell * 0.22;
      if (opts.doorIds) label(String(i - first), x, z, "#b48cff");
      if (opts.doorScores && d.score >= 0) label(String(d.score), x, z + (opts.doorIds ? cell * 0.25 : 0), "#5ad2ff");
    });
  }
}

/** A line with an arrowhead at its end. */
function arrow(ctx, x1, z1, x2, z2, head) {
  const a = Math.atan2(z2 - z1, x2 - x1);
  ctx.beginPath();
  ctx.moveTo(x1, z1);
  ctx.lineTo(x2, z2);
  ctx.moveTo(x2, z2);
  ctx.lineTo(x2 - head * Math.cos(a - 0.4), z2 - head * Math.sin(a - 0.4));
  ctx.moveTo(x2, z2);
  ctx.lineTo(x2 - head * Math.cos(a + 0.4), z2 - head * Math.sin(a + 0.4));
  ctx.stroke();
}

/** An arrow showing a facing angle in degrees (0 = +z, like the game). */
function facing(ctx, x, z, degrees, len, color) {
  const a = (degrees * Math.PI) / 180;
  ctx.strokeStyle = "#000";
  ctx.lineWidth = 3;
  arrow(ctx, x, z, x + len * Math.sin(a), z + len * Math.cos(a), len * 0.35);
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.5;
  arrow(ctx, x, z, x + len * Math.sin(a), z + len * Math.cos(a), len * 0.35);
}

// ---- Map tooltips ----

function setupTooltips() {
  const canvas = $("map");
  const tip = $("tooltip");
  const show = (clientX, clientY) => {
    const rect = canvas.getBoundingClientRect();
    const x = clientX - rect.left;
    const z = clientY - rect.top;
    let best = null;
    let bestDist = Infinity;
    for (const t of hitTargets) {
      const d = Math.hypot(t.x - x, t.z - z);
      if (d <= t.r && d < bestDist) {
        best = t;
        bestDist = d;
      }
    }
    if (!best) {
      tip.hidden = true;
      return;
    }
    tip.textContent = best.label;
    tip.hidden = false;
    const wrap = $("map-scroll").getBoundingClientRect();
    const left = Math.min(clientX - wrap.left + 12, wrap.width - tip.offsetWidth - 4);
    tip.style.left = `${Math.max(4, left) + $("map-scroll").scrollLeft}px`;
    tip.style.top = `${clientY - wrap.top + 14 + $("map-scroll").scrollTop}px`;
  };
  canvas.addEventListener("mousemove", (e) => show(e.clientX, e.clientY));
  canvas.addEventListener("mouseleave", () => ($("tooltip").hidden = true));
  canvas.addEventListener("click", (e) => !mapDragged() && show(e.clientX, e.clientY));
}

// ---- Summary ----

function el(tag, attrs = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  for (const c of children) e.append(c);
  return e;
}

async function iconEl(urls) {
  // With the image list the browser can load icons on its own, all at once.
  const url = available ? pickUrl(urls) : (await firstImage(urls))?.src;
  return url ? el("img", { src: url, alt: "", loading: "lazy" }) : el("span", { class: "icon-gap" });
}

async function renderSummary(layout) {
  const box = $("summary");
  const parts = [];

  const treasures = [];
  const teki = new Map();
  const gates = [];
  for (const o of layout.objects) {
    if (o.kind === "treasure") treasures.push({ name: o.name, game: o.game, value: o.value, by: null });
    if (o.kind === "teki") {
      const key = `${o.name}|${o.game}`;
      teki.set(key, (teki.get(key) || 0) + (o.count || 1));
      if (o.carrying) treasures.push({ name: o.carrying, game: o.game, value: null, by: o.name });
    }
    if (o.kind === "gate") gates.push(o);
  }

  const exit = layout.objects.find((o) => o.kind === "hole" || o.kind === "geyser");
  parts.push(el("h2", {}, T.layout));
  const info = el("ul");
  info.append(el("li", {}, T.unitsLine(layout.units.length, layout.width, layout.height)));
  info.append(el("li", {}, exit ? (exit.kind === "hole" ? T.exitHole(exit.plugged) : T.exitGeyser) : T.noExit));
  if (current && current.judge) info.append(el("li", { class: "judge-line" }, T.judgeLine(current.judge.type, current.judge.score)));
  parts.push(info);

  parts.push(el("h2", {}, T.treasures(treasures.length)));
  const tl = el("ul");
  for (const t of treasures) {
    const name = displayName("treasures", t.name);
    const label = t.by ? T.carriedBy(name, displayName("teki", t.by)) : name;
    const li = el("li", { title: t.name }, await iconEl(treasureIconUrls(t.name, t.game)), el("span", {}, label));
    li.append(el("span", { class: "value" }, t.value ? T.value(t.value) : ""));
    tl.append(li);
  }
  if (!treasures.length) tl.append(el("li", { class: "muted" }, T.none));
  parts.push(tl);

  const total = [...teki.values()].reduce((a, b) => a + b, 0);
  parts.push(el("h2", {}, T.teki(total)));
  const kl = el("ul");
  const sorted = [...teki.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  for (const [key, n] of sorted) {
    const [name, game] = key.split("|");
    kl.append(
      el(
        "li",
        { title: name },
        await iconEl(tekiIconUrls(name, game)),
        el("span", {}, displayName("teki", name)),
        el("span", { class: "value" }, `×${n}`),
      ),
    );
  }
  if (!sorted.length) kl.append(el("li", { class: "muted" }, T.none));
  parts.push(kl);

  if (gates.length) {
    parts.push(el("h2", {}, T.gates(gates.length)));
    const gl = el("ul");
    for (const g of gates) gl.append(el("li", {}, T.hp(Math.round(g.health))));
    parts.push(gl);
  }
  box.replaceChildren(...parts);
}

// ---- Controls ----

function setStatus(text, isError = false) {
  const s = $("status");
  s.textContent = text;
  s.classList.toggle("error", isError);
}

function cavesFor(game) {
  return caves.filter((c) => c.game === game);
}

function sublevelName(cave, floor) {
  if (cave.name === "Colossal Caverns") return `${cave.game}:colossal`;
  return `${cave.game}:${cave.short}-${floor}`;
}

function parseSeed(text) {
  const t = text.trim().replace(/^0x/i, "");
  if (!/^[0-9a-f]{1,8}$/i.test(t)) return null;
  return parseInt(t, 16) >>> 0;
}

function formatSeed(seed) {
  return (seed >>> 0).toString(16).toUpperCase().padStart(8, "0");
}

function randomSeed() {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

function fillGames(games) {
  const sel = $("game");
  const value = sel.value;
  sel.replaceChildren();
  for (const g of games) sel.append(el("option", { value: g }, gameName(g)));
  if (value) sel.value = value;
}

function fillCaves(game) {
  const sel = $("cave");
  const value = sel.value;
  sel.replaceChildren();
  const story = el("optgroup", { label: T.storyMode });
  const challenge = el("optgroup", { label: T.challengeMode });
  cavesFor(game).forEach((c) => {
    const name = caveName(c);
    const label = c.challenge ? `${c.short}: ${name}` : `${name} (${c.short})`;
    (c.challenge ? challenge : story).append(el("option", { value: caves.indexOf(c) }, label));
  });
  if (story.children.length) sel.append(story);
  if (challenge.children.length) sel.append(challenge);
  if (value && [...sel.options].some((o) => o.value === value)) sel.value = value;
}

function fillFloors(count, selected) {
  const sel = $("floor");
  sel.replaceChildren();
  for (let i = 1; i <= count; i++) sel.append(el("option", { value: i }, String(i)));
  sel.value = String(Math.min(Math.max(1, selected), count));
}

/** The cave and floor the controls show. */
function selected() {
  const cave = caves[Number($("cave").value)];
  return cave && { cave, floor: Number($("floor").value) || 1 };
}

/** Generates from the controls. The floor list is refilled when the cave changes. */
async function run({ newCave = false } = {}) {
  const cave = caves[Number($("cave").value)];
  if (!cave) return;
  const seed = parseSeed($("seed").value);
  if (seed === null) {
    setStatus(T.badSeed, true);
    return;
  }
  $("seed").value = formatSeed(seed);
  const floor = newCave ? 1 : Number($("floor").value) || 1;
  const sublevel = sublevelName(cave, floor);
  if (!current || current.sublevel !== sublevel) stopSearch(true);
  try {
    await loadGame(cave.game);
    setStatus(T.generating);
    const start = performance.now();
    const layout = await generate(sublevel, seed, needsDetails() ? 1 : 0);
    const ms = performance.now() - start;
    if (newCave || $("floor").options.length !== layout.floors) fillFloors(layout.floors, floor);
    const judgeType = $("judge").value;
    const judge = judgeType ? { type: judgeType, score: (await command("judge", sublevel, judgeType, seed)).score } : null;
    current = { layout, cave, floor, ms, sublevel, judge };
    $("result").hidden = false;
    $("tooltip").hidden = true;
    await showCurrent();
    updateUrl(cave, floor, seed);
    if ($("caveinfo-box").open) await showCaveInfo();
  } catch (e) {
    console.error(e);
    setStatus(T.cantGenerate(e.message), true);
  }
}

/** Draws the current layout and its text in the current language. */
async function showCurrent() {
  if (!current) return;
  const { layout, cave, floor, ms } = current;
  const title = layout.floors > 1 || cave.name !== "Colossal Caverns" ? T.floorName(caveName(cave), floor) : caveName(cave);
  $("title").textContent = `${title} · ${layout.seed}`;
  await render(layout);
  await renderSummary(layout);
  setStatus(T.generated(layout.sublevel, layout.seed, ms.toFixed(1)));
}

function updateUrl(cave, floor, seed) {
  const p = new URLSearchParams(location.search);
  p.set("sublevel", `${cave.game}:${cave.short}-${floor}`);
  p.set("seed", formatSeed(seed));
  history.replaceState(null, "", `?${p.toString()}`);
}

/** Selects the cave and floor named by a "game:short-floor" (or any "SCx6"-style) string. */
function selectFromParam(text) {
  let game = null;
  let rest = text;
  if (text.includes(":")) [game, rest] = text.split(":", 2);
  const m = rest.match(/^(.*?)-?(\d+)$/);
  const short = (m ? m[1] : rest).toLowerCase();
  const floor = m ? Number(m[2]) : 1;
  const cave = caves.find(
    (c) =>
      (!game || c.game === game.toLowerCase()) &&
      (c.short.toLowerCase() === short || c.aliases.some((a) => a.toLowerCase() === short) || c.name.toLowerCase() === short),
  );
  if (!cave) return null;
  return { cave, floor };
}

// ---- Advanced options ----

const JUDGES = ["score", "pod", "at", "attk", "cmat", "key", "colossal", "mapunitcount"];
const SETTINGS_KEY = "cavegen3-options";

function buildAdvanced() {
  const box = $("map-options");
  for (const { key, on } of MAP_OPTIONS) {
    const input = el("input", { type: "checkbox", id: `opt-${key}` });
    input.checked = !!on;
    box.append(el("label", { class: "check" }, input, el("span", { "data-i18n-opt": key })));
  }
  fillJudges();
  // Saved choices (map options and judge) from last time.
  try {
    const saved = JSON.parse(localStorage.getItem(SETTINGS_KEY) || "{}");
    for (const { key } of MAP_OPTIONS) if (key in (saved.map || {})) $(`opt-${key}`).checked = !!saved.map[key];
    if (JUDGES.includes(saved.judge)) $("judge").value = saved.judge;
    if (saved.comedown) $("comedown").checked = true;
    if (saved.comedownSize) $("comedown-size").value = String(saved.comedownSize);
    if (saved.open) $("advanced").open = true;
  } catch {
    // Nothing saved, or storage is unavailable.
  }
}

function saveSettings() {
  const map = {};
  for (const { key } of MAP_OPTIONS) map[key] = $(`opt-${key}`).checked;
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify({
        map,
        judge: $("judge").value,
        open: $("advanced").open,
        comedown: $("comedown").checked,
        comedownSize: comedownCells(),
      }));
  } catch {
    // Not saved; the choices still apply to this page.
  }
}

function fillJudges() {
  const sel = $("judge");
  const value = sel.value;
  sel.replaceChildren(el("option", { value: "" }, T.judgeNone));
  for (const j of JUDGES) sel.append(el("option", { value: j }, T.judgeNames[j]));
  sel.value = value;
}

/**
 * Lets the browser draw and handle input between slices of a long job. A message is used rather
 * than setTimeout, which browsers slow to once a second or less in background tabs.
 */
function yieldToBrowser() {
  return new Promise((resolve) => {
    const ch = new MessageChannel();
    ch.port1.onmessage = () => resolve();
    ch.port2.postMessage(0);
  });
}

// Seed search runs a slice of seeds at a time so the page stays responsive.
let search = null; // {token, sublevel, best, scores, ...} while running or after

function stopSearch(clear = false) {
  if (search) search.stopped = true;
  if (clear) {
    search = null;
    $("search-results").replaceChildren();
    $("search-status").textContent = "";
  }
  $("search-go").textContent = T.searchButton;
}

async function startSearch() {
  if (search && !search.done) {
    stopSearch();
    return;
  }
  const sel = selected();
  if (!sel) return;
  const judge = $("judge").value;
  const query = $("query").value.trim();
  const status = $("search-status");
  status.classList.remove("error");
  if (!judge && !query) {
    status.textContent = T.needJudgeOrQuery;
    status.classList.add("error");
    return;
  }
  const total = Math.max(1, Math.min(1e8, Math.floor(Number($("search-count").value) || 0)));
  const keep = Math.max(1, Math.min(500, Math.floor(Number($("search-keep").value) || 20)));
  const mode = $("search-mode").value;
  const seed = parseSeed($("seed").value) ?? 0;
  const startAt = mode === "c" ? seed : randomSeed();
  const s = {
    sublevel: sublevelName(sel.cave, sel.floor),
    sel,
    judge,
    query,
    keep,
    total,
    tested: 0,
    matched: 0,
    best: [],
    scores: [],
    stopped: false,
    done: false,
    t0: performance.now(),
  };
  search = s;
  $("search-go").textContent = T.stop;
  try {
    await loadGame(sel.cave.game);
    let chunk = 50;
    while (s.tested < total && !s.stopped) {
      const n = Math.min(chunk, total - s.tested);
      const t = performance.now();
      const r = await command("search", s.sublevel, judge || "-", query || "-", startAt + s.tested, n, mode, keep);
      s.tested += n;
      s.matched += r.matched;
      if (judge) {
        s.best = s.best.concat(r.best).sort((a, b) => a[1] - b[1]).slice(0, keep);
        for (const v of r.scores) s.scores.push(v);
      } else {
        s.best = s.best.concat(r.best).slice(0, keep);
        if (s.best.length >= keep) break;
      }
      // Aim for about 60 ms of work per slice.
      chunk = Math.max(10, Math.min(200000, Math.round((n * 60) / Math.max(1, performance.now() - t))));
      status.textContent = T.searching(s.tested, total);
      showSearchResults(s, false);
      await yieldToBrowser();
    }
    const secs = ((performance.now() - s.t0) / 1000).toFixed(1);
    status.textContent = s.stopped ? T.searchStopped(s.tested) : T.searchDone(s.tested, query ? s.matched : null, secs);
  } catch (e) {
    console.error(e);
    status.textContent = e.message;
    status.classList.add("error");
    s.failed = true;
  } finally {
    s.done = true;
    if (search === s) {
      $("search-go").textContent = T.searchButton;
      if (s.failed) $("search-results").replaceChildren();
      else showSearchResults(s, true);
    }
  }
}

function showSearchResults(s, final) {
  const list = $("search-results");
  if (!s.best.length) {
    list.replaceChildren(final ? el("li", { class: "muted" }, T.noMatches) : "");
    return;
  }
  // Rank among the seeds tested: the share of them that scored at least as well.
  let sorted = null;
  if (final && s.judge && s.scores.length) sorted = Float64Array.from(s.scores).sort();
  const rank = (score) => {
    let lo = 0;
    let hi = sorted.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (sorted[mid] <= score) lo = mid + 1;
      else hi = mid;
    }
    const pct = (100 * lo) / sorted.length;
    return pct < 1 ? pct.toPrecision(2) : pct.toFixed(1);
  };
  const items = s.best.map(([seed, score]) => {
    const btn = el("button", { type: "button", class: "seed-link" }, formatSeed(seed));
    btn.addEventListener("click", () => openSearchResult(s, seed));
    const li = el("li", {}, btn);
    if (s.judge) {
      li.append(el("span", { class: "value" }, String(score)));
      if (sorted) li.append(el("span", { class: "muted rank" }, T.rank(rank(score))));
    }
    return li;
  });
  list.replaceChildren(...items);
}

async function openSearchResult(s, seed) {
  const now = selected();
  if (!now || now.cave !== s.sel.cave || now.floor !== s.sel.floor) {
    $("game").value = s.sel.cave.game;
    fillCaves(s.sel.cave.game);
    $("cave").value = String(caves.indexOf(s.sel.cave));
    fillFloors(s.sel.floor, s.sel.floor);
  }
  $("seed").value = formatSeed(seed);
  await run();
  $("result").scrollIntoView({ behavior: "smooth", block: "start" });
}

// ---- CaveInfo report ----

async function showCaveInfo() {
  const sel = selected();
  const box = $("caveinfo-body");
  if (!sel) return;
  let ci;
  try {
    await loadGame(sel.cave.game);
    ci = await command("caveinfo", sublevelName(sel.cave, sel.floor));
  } catch (e) {
    box.replaceChildren(el("p", { class: "error" }, e.message));
    return;
  }
  const yesNo = (b) => (b ? T.yes : T.no);
  const fl = ci.floor;
  const pct = (p) => `${Math.round(p * 100)}%`;
  const facts = [
    [T.maxTeki, fl.max_teki],
    [T.maxTreasures, fl.max_treasures],
    [T.maxGates, fl.max_gates],
    [T.rooms, fl.rooms],
    [T.corridorProb, pct(fl.corridor_probability)],
    [T.capProb, pct(fl.cap_probability)],
    [T.hasGeyser, yesNo(fl.geyser)],
    [T.pluggedHole, yesNo(fl.plugged)],
    [T.finalFloor, yesNo(fl.final)],
  ];
  if (fl.waterwraith > 0) facts.push([T.waterwraith, `${fl.waterwraith} s`]);
  const factList = el("dl", { class: "facts" });
  for (const [k, v] of facts) factList.append(el("div", {}, el("dt", {}, k), el("dd", {}, String(v))));

  const table = (headers, rows) => {
    const t = el("table");
    t.append(el("thead", {}, el("tr", {}, ...headers.map((h) => el("th", {}, h)))));
    const body = el("tbody");
    for (const r of rows) body.append(el("tr", {}, ...r.map((c) => el("td", {}, c))));
    t.append(body);
    return el("div", { class: "table-wrap" }, t);
  };
  const nameCell = async (kind, item) => {
    const urls = kind === "teki" ? tekiIconUrls(item.name, item.game) : treasureIconUrls(item.name, item.game);
    const parts = [await iconEl(urls), el("span", {}, displayName(kind, item.name))];
    if (item.carrying) parts.push(el("span", { class: "muted" }, ` (${T.carrying(displayName("treasures", item.carrying))})`));
    if (item.falling) parts.push(el("span", { class: "muted" }, ` (${T.falling})`));
    return el("span", { class: "name-cell", title: item.name }, ...parts);
  };
  const groupName = (g) => T.groupNames[g] || String(g);

  const sections = [el("h3", {}, T.floorInfo), factList];
  if (ci.teki.length) {
    const rows = [];
    for (const t of ci.teki) rows.push([await nameCell("teki", t), String(t.min), String(t.weight), groupName(t.group)]);
    sections.push(el("h3", {}, T.tekiInfo), table([T.colName, T.colMin, T.colWeight, T.colGroup], rows));
  }
  if (ci.items.length) {
    const rows = [];
    for (const t of ci.items) rows.push([await nameCell("treasures", t), String(t.min), String(t.weight), t.value ? String(t.value) : ""]);
    sections.push(el("h3", {}, T.itemInfo), table([T.colName, T.colMin, T.colWeight, T.colValue], rows));
  }
  if (ci.gates.length) {
    const rows = ci.gates.map((g) => [T.hp(Math.round(g.health)), String(g.weight)]);
    sections.push(el("h3", {}, T.gateInfo), table([T.colHealth, T.colWeight], rows));
  }
  if (ci.caps.length) {
    const rows = [];
    for (const t of ci.caps) rows.push([await nameCell("teki", t), String(t.min), String(t.weight), String(t.group)]);
    sections.push(el("h3", {}, T.capInfo), table([T.colName, T.colMin, T.colWeight, T.colGroup], rows));
  }
  const units = el("ul", { class: "unit-list" });
  for (const u of ci.units) {
    const url = available ? pickUrl(unitUrls(u.name, u.game)) : unitUrls(u.name, u.game)[0];
    const img = url ? el("img", { src: url, alt: "", loading: "lazy", style: `aspect-ratio: ${u.w} / ${u.h}` }) : el("span");
    units.append(
      el(
        "li",
        { title: u.name },
        el("span", { class: "unit-thumb" }, img),
        el("span", { class: "unit-name" }, u.name),
        el("span", { class: "muted" }, `${u.w}×${u.h} · ${T.unitTypes[u.type] || u.type} · ${T.colDoors} ${u.doors}`),
      ),
    );
  }
  sections.push(el("h3", {}, `${T.units} (${ci.units.length})`), units);
  box.replaceChildren(...sections);
}

// ---- Language ----

/** Puts the page's fixed text in the current language. */
function applyLanguage() {
  document.documentElement.lang = lang;
  for (const node of document.querySelectorAll("[data-i18n]")) node.textContent = T[node.dataset.i18n];
  for (const node of document.querySelectorAll("[data-i18n-html]")) node.innerHTML = T[node.dataset.i18nHtml];
  for (const node of document.querySelectorAll("[data-i18n-opt]")) node.textContent = T.opt[node.dataset.i18nOpt];
  for (const node of document.querySelectorAll("[data-i18n-title]")) {
    node.title = T[node.dataset.i18nTitle];
    node.setAttribute("aria-label", T[node.dataset.i18nTitle]);
  }
  for (const node of document.querySelectorAll("[data-i18n-placeholder]")) node.placeholder = T[node.dataset.i18nPlaceholder];
  if ($("seed")) $("seed").title = T.seedTitle;
  if ($("footer-text")) $("footer-text").innerHTML = T.footer;
  if ($("search-go")) $("search-go").textContent = search && !search.done ? T.stop : T.searchButton;
  if ($("judge")) fillJudges();
  if (T.pageTitle[page]) document.title = T.pageTitle[page];
  const toggle = $("lang-toggle");
  toggle.textContent = T.otherLang;
  toggle.setAttribute("aria-label", T.otherLangLabel);
  toggle.lang = lang === "ja" ? "en" : "ja";
}

async function setLanguage(next) {
  lang = next;
  T = STRINGS[lang];
  try {
    localStorage.setItem("cavegen3-lang", lang);
  } catch {
    // Not saved; the choice still applies to this page.
  }
  const p = new URLSearchParams(location.search);
  p.set("lang", lang);
  history.replaceState(null, "", `?${p.toString()}`);
  applyLanguage();
  if (caves.length) {
    fillGames([...$("game").options].map((o) => o.value));
    fillCaves($("game").value);
  }
  await showCurrent();
  if (search) showSearchResults(search, search.done);
  if ($("caveinfo-box")?.open) await showCaveInfo();
  if (languageHook) await languageHook();
}

/** Language switching and the suggestion-box link, on every page. */
function setupHeader() {
  $("lang-toggle").addEventListener("click", () => setLanguage(lang === "ja" ? "en" : "ja"));
  // The suggestion box records which page a suggestion came from.
  for (const a of document.querySelectorAll("a.nav-suggest")) {
    a.addEventListener("click", () => {
      const here = (location.pathname.split("/").pop() || "index.html") + location.search;
      a.href = `suggest.html?from=${encodeURIComponent(here)}`;
    });
  }
}

// ---- Suggestion box ----

// Suggestions from all of the Pikmin sites go to one Google Form; its hidden "page" field says
// which site and page each came from (here "cavegen3/<lang>/<page>"). The form's fields are
// named in suggest.html.
function initSuggest() {
  applyLanguage();
  setupHeader();
  const form = $("suggest");
  const status = $("suggest-status");
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    const from = new URLSearchParams(location.search).get("from") || "";
    $("suggest-page").value = `cavegen3/${lang}/${from}`;
    const button = form.querySelector("button");
    button.disabled = true;
    status.textContent = T.suggestSending;
    try {
      // Google Forms doesn't allow reading the reply, so a sent request counts as success.
      await fetch(form.action, { method: "POST", body: new FormData(form), mode: "no-cors" });
      form.reset();
      status.textContent = T.suggestSent;
    } catch {
      status.textContent = T.suggestFail;
    } finally {
      button.disabled = false;
    }
  });
}

/** Loads the generator, the cave list, in-game names and the image list, and fills the game menu. */
async function startCommon() {
  const [, resources, gamesText, namesJson, imageList] = await Promise.all([
    startWasm(),
    fetchGz("data/resources.bin.gz"),
    fetch("data/games.txt").then((r) => r.text()),
    fetch("data/names.json").then((r) => (r.ok ? r.json() : null)),
    fetch("data/images.json").then((r) => (r.ok ? r.json() : null)),
  ]);
  if (namesJson) names = namesJson;
  if (imageList) available = new Set(imageList);
  addBundle(resources);
  bundles.push(resources);
  caves = readResult(wasm.caves());
  const games = gamesText.split(/\s+/).filter(Boolean);
  games.sort((a, b) => GAME_ORDER.indexOf(a) - GAME_ORDER.indexOf(b));
  caves = caves.filter((c) => games.includes(c.game));
  fillGames(games);
}

/** Selects the cave and floor from the link (`?sublevel=`), or the first cave. */
function selectStart() {
  const fromUrl = params.get("sublevel") && selectFromParam(params.get("sublevel"));
  const start = fromUrl || { cave: caves[0], floor: 1 };
  $("game").value = start.cave.game;
  fillCaves(start.cave.game);
  $("cave").value = String(caves.indexOf(start.cave));
  fillFloors(start.floor, start.floor);
}

async function init() {
  buildAdvanced();
  applyLanguage();
  setupHeader();
  setStatus(T.loading);
  try {
    await startCommon();
  } catch (e) {
    console.error(e);
    setStatus(T.cantStart(e.message), true);
    return;
  }

  selectStart();
  const seed = params.get("seed") && parseSeed(params.get("seed"));
  if (params.get("comedown") === "1") $("comedown").checked = true;
  $("seed").value = formatSeed(seed ?? randomSeed());

  $("game").addEventListener("change", () => {
    fillCaves($("game").value);
    $("cave").selectedIndex = 0;
    run({ newCave: true });
  });
  $("cave").addEventListener("change", () => run({ newCave: true }));
  $("floor").addEventListener("change", () => run());
  $("controls").addEventListener("submit", (e) => {
    e.preventDefault();
    run();
  });
  $("random").addEventListener("click", () => {
    $("seed").value = formatSeed(randomSeed());
    run();
  });
  const step = (d) => {
    const s = parseSeed($("seed").value);
    if (s === null) return;
    $("seed").value = formatSeed(s + d);
    run();
  };
  $("prev").addEventListener("click", () => step(-1));
  $("next").addEventListener("click", () => step(1));
  $("water").addEventListener("change", () => current && render(current.layout));
  const redraw = () => current && render(current.layout);
  const zoomTo = (s) => {
    zoom = Math.max(0.05, s);
    return current ? render(current.layout) : Promise.resolve();
  };
  mapDragged = attachZoom($("map-scroll"), $("map"), () => shownScale, zoomTo);
  const center = () => {
    const r = $("map-scroll").getBoundingClientRect();
    return [r.left + r.width / 2, r.top + r.height / 2];
  };
  // The buttons zoom about the middle of what's shown, like the wheel does about the cursor.
  const zoomStep = (f) => $("map-scroll").dispatchEvent(new WheelEvent("wheel", { deltaY: -Math.log(f) / 0.0015, clientX: center()[0], clientY: center()[1], cancelable: true }));
  $("zoom-in").addEventListener("click", () => zoomStep(1.5));
  $("zoom-out").addEventListener("click", () => zoomStep(1 / 1.5));
  $("zoom-fit").addEventListener("click", () => {
    zoom = "fit";
    redraw();
  });
  $("comedown").addEventListener("change", () => {
    zoom = "fit";
    saveSettings();
    const p = new URLSearchParams(location.search);
    if ($("comedown").checked) p.set("comedown", "1");
    else p.delete("comedown");
    history.replaceState(null, "", `?${p.toString()}`);
    redraw();
  });
  $("comedown-size").addEventListener("change", () => {
    saveSettings();
    redraw();
  });
  for (const { key, details } of MAP_OPTIONS) {
    $(`opt-${key}`).addEventListener("change", () => {
      saveSettings();
      if (!current) return;
      // Turning on an option that needs the extra layout data generates the layout again.
      if (details && !current.layout.spawnpoints && $(`opt-${key}`).checked) run();
      else render(current.layout);
    });
  }
  $("judge").addEventListener("change", () => {
    saveSettings();
    run();
  });
  $("advanced").addEventListener("toggle", saveSettings);
  $("search-go").addEventListener("click", startSearch);
  $("query").addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      startSearch();
    }
  });
  $("caveinfo-box").addEventListener("toggle", () => $("caveinfo-box").open && showCaveInfo());
  let resizeTimer;
  window.addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => current && zoom === "fit" && render(current.layout), 150);
  });
  $("download").addEventListener("click", () => {
    if (!current) return;
    $("map").toBlob((blob) => {
      const a = el("a", { href: URL.createObjectURL(blob), download: `${current.layout.sublevel}_${current.layout.seed}.png` });
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    });
  });
  $("copy-link").addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(location.href);
      setStatus(T.linkCopied);
    } catch {
      setStatus(location.href);
    }
  });
  setupTooltips();

  await run();
}

// Each page's own setup; the aggregator's is in aggregator.js, loaded after this file.
window.addEventListener("DOMContentLoaded", () => {
  if (page === "aggregator") initAggregator();
  else if (page === "suggest") initSuggest();
  else init();
});
