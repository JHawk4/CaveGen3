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
let hitTargets = []; // icons on the map, for tooltips: {x, z, r, label}

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

async function loadGame(game) {
  if (loadedGames.has(game)) return;
  setStatus(T.loadingGame(gameName(game)));
  const bytes = await fetchGz(`data/${game}.bin.gz`);
  withBytes(bytes, (p, n) => readResult(wasm.add_files(p, n)));
  loadedGames.add(game);
}

function generate(sublevel, seed) {
  const name = new TextEncoder().encode(sublevel);
  return withBytes(name, (p, n) => readResult(wasm.generate(p, n, seed >>> 0)));
}

// ---- Images ----

const images = new Map();
function loadImage(url) {
  if (!images.has(url)) {
    images.set(
      url,
      new Promise((resolve) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = () => resolve(null);
        img.src = url;
      }),
    );
  }
  return images.get(url);
}

/** The first of `urls` that loads, or null. */
async function firstImage(urls) {
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

function specialUrl(name) {
  return `img/special/${name.toLowerCase()}.png`;
}

// ---- Drawing ----

// Radar images come in two encodings: Caveripper's extractor stores the shading in the alpha
// channel (and they look dark), while CaveGen's are opaque grayscale. Both get brightened, by
// different amounts, so they end up looking alike.
const brightnessCache = new WeakMap();
function unitBrightness(img) {
  if (!brightnessCache.has(img)) {
    const c = document.createElement("canvas");
    c.width = img.width;
    c.height = img.height;
    const ctx = c.getContext("2d", { willReadFrequently: true });
    ctx.drawImage(img, 0, 0);
    const d = ctx.getImageData(0, 0, c.width, c.height).data;
    let translucent = false;
    for (let i = 3; i < d.length; i += 4) {
      if (d[i] < 255) {
        translucent = true;
        break;
      }
    }
    brightnessCache.set(img, translucent ? "brightness(1.6)" : "brightness(1.15)");
  }
  return brightnessCache.get(img);
}

function zoomScale(layout) {
  const z = $("zoom").value;
  if (z !== "fit") return Number(z);
  const box = $("map-scroll").getBoundingClientRect();
  const availW = Math.max(200, box.width - 16);
  const availH = Math.max(200, window.innerHeight * 0.78);
  // +1 for the half-cell margin on each side.
  const s = Math.min(availW / ((layout.width + 1) * CELL), availH / ((layout.height + 1) * CELL), 1.5);
  return Math.max(0.25, s);
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

async function render(layout) {
  const canvas = $("map");
  const scale = zoomScale(layout);
  const cell = CELL * scale;
  const dpr = window.devicePixelRatio || 1;
  const pad = cell * 0.5;
  const w = layout.width * cell + pad * 2;
  const h = layout.height * cell + pad * 2;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  canvas.style.width = `${Math.round(w)}px`;
  canvas.style.height = `${Math.round(h)}px`;
  hitTargets = [];

  const ctx = canvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = "#0f0f0f";
  ctx.fillRect(0, 0, w, h);
  ctx.translate(pad, pad);

  const toPx = (v) => (v / UNIT) * cell;

  // Map units, rotated clockwise in quarter turns like the game's radar images.
  const unitImgs = await Promise.all(
    layout.units.map((u) => {
      const n = u.name.toLowerCase();
      return firstImage([`img/${u.game}/mapunits/${n}.png`, `img/pikmin2/mapunits/${n}.png`]);
    }),
  );
  ctx.imageSmoothingEnabled = false;
  layout.units.forEach((u, i) => {
    const img = unitImgs[i];
    const cx = (u.x + u.w / 2) * cell;
    const cz = (u.z + u.h / 2) * cell;
    ctx.save();
    ctx.translate(cx, cz);
    if (img) {
      ctx.rotate((u.rot * Math.PI) / 2);
      const [w0, h0] = u.rot % 2 ? [u.h, u.w] : [u.w, u.h];
      ctx.filter = unitBrightness(img);
      ctx.drawImage(img, (-w0 * cell) / 2, (-h0 * cell) / 2, w0 * cell, h0 * cell);
      ctx.filter = "none";
    } else {
      ctx.fillStyle = "#333";
      ctx.fillRect((-u.w * cell) / 2, (-u.h * cell) / 2, u.w * cell, u.h * cell);
    }
    ctx.restore();
  });

  if ($("water").checked) {
    ctx.fillStyle = "rgba(0, 100, 230, 0.25)";
    for (const u of layout.units) {
      const cx = (u.x + u.w / 2) * cell;
      const cz = (u.z + u.h / 2) * cell;
      for (const [x1, z1, x2, z2] of u.water) {
        ctx.fillRect(cx + toPx(x1), cz + toPx(z1), toPx(x2 - x1), toPx(z2 - z1));
      }
    }
  }

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
  const drawFallback = (label, x, z, size, color) => {
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(x, z, size / 2.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#fff";
    ctx.font = `bold ${Math.max(8, size / 3.2)}px system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(label.slice(0, 3), x, z);
  };
  const drawBadge = (text, x, z, size) => {
    ctx.font = `bold ${Math.max(9, size)}px system-ui, sans-serif`;
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.lineWidth = 3;
    ctx.strokeStyle = "#000";
    ctx.strokeText(text, x, z);
    ctx.fillStyle = "#fff";
    ctx.fillText(text, x, z);
  };

  // Exits and the ship first, then gates, treasures and teki on top.
  const order = { hole: 0, geyser: 0, ship: 1, onion: 1, gate: 2, treasure: 3, teki: 4 };
  const objects = [...layout.objects].sort((a, b) => order[a.kind] - order[b.kind]);
  const falling = await loadImage(specialUrl("falling_icon"));

  for (const o of objects) {
    const x = toPx(o.x);
    const z = toPx(o.z);
    let size = cell * 0.6;
    switch (o.kind) {
      case "ship":
        drawIcon(await loadImage(specialUrl("pod_icon")), x, z, size) || drawFallback("Pod", x, z, size, "#c00");
        break;
      case "hole":
        drawIcon(await loadImage(specialUrl("Cave_icon")), x, z, size);
        if (o.plugged) drawIcon(await loadImage(specialUrl("36px-Clog_icon")), x, z, size);
        break;
      case "geyser":
        size = cell * 0.7;
        drawIcon(await loadImage(specialUrl("Geyser_icon")), x, z, size);
        break;
      case "onion":
        size = cell * 0.45;
        drawIcon(await loadImage(specialUrl(`onion${o.color}`)), x, z, size);
        break;
      case "gate": {
        size = cell * 0.85;
        const img = await loadImage(specialUrl("Gray_bramble_gate_icon"));
        drawIcon(img, x, z, size, o.dir % 2 ? Math.PI / 2 : 0);
        drawBadge(String(Math.round(o.health)), x - cell * 0.3, z + cell * 0.38, cell * 0.24);
        break;
      }
      case "treasure": {
        size = cell * 0.55;
        const img = await firstImage(treasureIconUrls(o.name, o.game));
        drawIcon(img, x, z, size) || drawFallback(o.name, x, z, size, "#c70");
        break;
      }
      case "teki": {
        size = o.group === 0 ? cell * 0.45 : cell * 0.6;
        const img = await firstImage(tekiIconUrls(o.name, o.game));
        drawIcon(img, x, z, size) || drawFallback(o.name, x, z, size, "#7a2a8a");
        if (o.carrying) {
          const t = await firstImage(treasureIconUrls(o.carrying, o.game));
          drawIcon(t, x + size * 0.35, z + size * 0.35, size * 0.6);
        }
        if (o.falling) drawIcon(falling, x - size * 0.4, z - size * 0.4, size * 0.4);
        if (o.count > 1) drawBadge(`×${o.count}`, x + size * 0.3, z - size * 0.35, size * 0.38);
        break;
      }
    }
    hitTargets.push({ x: x + pad, z: z + pad, r: Math.max(size / 2, 8), label: objectLabel(o) });
  }
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
  canvas.addEventListener("click", (e) => show(e.clientX, e.clientY));
}

// ---- Summary ----

function el(tag, attrs = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  for (const c of children) e.append(c);
  return e;
}

async function iconEl(urls) {
  const img = await firstImage(urls);
  return img ? el("img", { src: img.src, alt: "" }) : el("span", { class: "icon-gap" });
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

function fillGames(available) {
  const sel = $("game");
  const value = sel.value;
  sel.replaceChildren();
  for (const g of available) sel.append(el("option", { value: g }, gameName(g)));
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
  try {
    await loadGame(cave.game);
    setStatus(T.generating);
    const start = performance.now();
    const layout = generate(sublevelName(cave, floor), seed);
    const ms = performance.now() - start;
    if (newCave || $("floor").options.length !== layout.floors) fillFloors(layout.floors, floor);
    current = { layout, cave, floor, ms };
    $("result").hidden = false;
    $("tooltip").hidden = true;
    await showCurrent();
    updateUrl(cave, floor, seed);
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

/** Puts the page's fixed text in the current language. */
function applyLanguage() {
  document.documentElement.lang = lang;
  for (const node of document.querySelectorAll("[data-i18n]")) node.textContent = T[node.dataset.i18n];
  for (const node of document.querySelectorAll("[data-i18n-title]")) {
    node.title = T[node.dataset.i18nTitle];
    node.setAttribute("aria-label", T[node.dataset.i18nTitle]);
  }
  $("seed").title = T.seedTitle;
  $("footer-text").innerHTML = T.footer;
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
}

async function init() {
  applyLanguage();
  $("lang-toggle").addEventListener("click", () => setLanguage(lang === "ja" ? "en" : "ja"));
  setStatus(T.loading);
  try {
    const [{ instance }, resources, gamesText, namesJson] = await Promise.all([
      WebAssembly.instantiateStreaming(fetch("cavegen3_web.wasm"), {}),
      fetchGz("data/resources.bin.gz"),
      fetch("data/games.txt").then((r) => r.text()),
      fetch("data/names.json").then((r) => (r.ok ? r.json() : null)),
    ]);
    wasm = instance.exports;
    if (namesJson) names = namesJson;
    withBytes(resources, (p, n) => readResult(wasm.add_files(p, n)));
    caves = readResult(wasm.caves());
    const games = gamesText.split(/\s+/).filter(Boolean);
    games.sort((a, b) => GAME_ORDER.indexOf(a) - GAME_ORDER.indexOf(b));
    caves = caves.filter((c) => games.includes(c.game));
    fillGames(games);
  } catch (e) {
    console.error(e);
    setStatus(T.cantStart(e.message), true);
    return;
  }

  const fromUrl = params.get("sublevel") && selectFromParam(params.get("sublevel"));
  const seed = params.get("seed") && parseSeed(params.get("seed"));
  const start = fromUrl || { cave: caves[0], floor: 1 };
  $("game").value = start.cave.game;
  fillCaves(start.cave.game);
  $("cave").value = String(caves.indexOf(start.cave));
  fillFloors(start.floor, start.floor);
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
  $("zoom").addEventListener("change", () => current && render(current.layout));
  let resizeTimer;
  window.addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => current && $("zoom").value === "fit" && render(current.layout), 150);
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

init();
