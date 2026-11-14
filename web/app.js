// CaveGen3 web page: loads the WebAssembly generator and draws layouts on a canvas.
"use strict";

const GAME_NAMES = {
  pikmin2: "Pikmin 2",
  "251": "Pikmin 251",
  newyear: "Pikmin 2: New Year",
  colossal: "Colossal Caverns",
  "216": "Pikmin 2: 216",
  bikmin: "Bikmin",
};

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

let wasm = null;
let caves = [];
const loadedGames = new Set();
let current = null; // the last generated layout

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
  setStatus(`Loading ${GAME_NAMES[game] || game} data…`);
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
    images.set(url, new Promise((resolve) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = () => resolve(null);
      img.src = url;
    }));
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
    switch (o.kind) {
      case "ship":
        drawIcon(await loadImage(specialUrl("pod_icon")), x, z, cell * 0.6) || drawFallback("Pod", x, z, cell * 0.6, "#c00");
        break;
      case "hole":
        drawIcon(await loadImage(specialUrl("Cave_icon")), x, z, cell * 0.6);
        if (o.plugged) drawIcon(await loadImage(specialUrl("36px-Clog_icon")), x, z, cell * 0.6);
        break;
      case "geyser":
        drawIcon(await loadImage(specialUrl("Geyser_icon")), x, z, cell * 0.7);
        break;
      case "onion":
        drawIcon(await loadImage(specialUrl(`onion${o.color}`)), x, z, cell * 0.45);
        break;
      case "gate": {
        const img = await loadImage(specialUrl("Gray_bramble_gate_icon"));
        drawIcon(img, x, z, cell * 0.85, o.dir % 2 ? Math.PI / 2 : 0);
        drawBadge(String(Math.round(o.health)), x - cell * 0.3, z + cell * 0.38, cell * 0.24);
        break;
      }
      case "treasure": {
        const img = await firstImage(treasureIconUrls(o.name, o.game));
        drawIcon(img, x, z, cell * 0.55) || drawFallback(o.name, x, z, cell * 0.55, "#c70");
        break;
      }
      case "teki": {
        const size = o.group === 0 ? cell * 0.45 : cell * 0.6;
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
  }
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
  return img ? el("img", { src: img.src, alt: "" }) : el("span", { style: "width:24px" });
}

async function renderSummary(layout) {
  const box = $("summary");
  box.replaceChildren();

  const treasures = [];
  const teki = new Map();
  let gates = [];
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
  box.append(el("h2", {}, "Layout"));
  const info = el("ul");
  info.append(el("li", {}, `${layout.units.length} map units, ${layout.width}×${layout.height} cells`));
  info.append(el("li", {}, exit ? (exit.kind === "hole" ? `Exit: hole${exit.plugged ? " (plugged)" : ""}` : "Exit: geyser") : "No exit (final floor)"));
  box.append(info);

  box.append(el("h2", {}, `Treasures (${treasures.length})`));
  const tl = el("ul");
  for (const t of treasures) {
    const label = t.by ? `${t.name} (in ${t.by})` : t.name;
    tl.append(el("li", {}, await iconEl(treasureIconUrls(t.name, t.game)), label, el("span", { class: "value" }, t.value ? `${t.value}` : "")));
  }
  if (!treasures.length) tl.append(el("li", { class: "muted" }, "None"));
  box.append(tl);

  const total = [...teki.values()].reduce((a, b) => a + b, 0);
  box.append(el("h2", {}, `Teki (${total})`));
  const kl = el("ul");
  const sorted = [...teki.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  for (const [key, n] of sorted) {
    const [name, game] = key.split("|");
    kl.append(el("li", {}, await iconEl(tekiIconUrls(name, game)), name, el("span", { class: "value" }, `×${n}`)));
  }
  if (!sorted.length) kl.append(el("li", { class: "muted" }, "None"));
  box.append(kl);

  if (gates.length) {
    box.append(el("h2", {}, `Gates (${gates.length})`));
    const gl = el("ul");
    for (const g of gates) gl.append(el("li", {}, `${Math.round(g.health)} HP`));
    box.append(gl);
  }
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
  sel.replaceChildren();
  for (const g of available) sel.append(el("option", { value: g }, GAME_NAMES[g] || g));
}

function fillCaves(game) {
  const sel = $("cave");
  sel.replaceChildren();
  const story = el("optgroup", { label: "Story mode" });
  const challenge = el("optgroup", { label: "Challenge mode" });
  cavesFor(game).forEach((c) => {
    const label = c.challenge ? `${c.short}: ${c.name}` : `${c.name} (${c.short})`;
    (c.challenge ? challenge : story).append(el("option", { value: caves.indexOf(c) }, label));
  });
  if (story.children.length) sel.append(story);
  if (challenge.children.length) sel.append(challenge);
}

function fillFloors(count, selected) {
  const sel = $("floor");
  sel.replaceChildren();
  for (let i = 1; i <= count; i++) sel.append(el("option", { value: i }, String(i)));
  sel.value = String(Math.min(Math.max(1, selected), count));
}

/** Generates from the controls. `floorCount` is refilled when the cave changes. */
async function run({ newCave = false } = {}) {
  const cave = caves[Number($("cave").value)];
  if (!cave) return;
  const seed = parseSeed($("seed").value);
  if (seed === null) {
    setStatus("The seed should be up to 8 hex digits, like 1234ABCD.", true);
    return;
  }
  $("seed").value = formatSeed(seed);
  const floor = newCave ? 1 : Number($("floor").value) || 1;
  try {
    await loadGame(cave.game);
    setStatus("Generating…");
    const start = performance.now();
    const layout = generate(sublevelName(cave, floor), seed);
    const ms = performance.now() - start;
    if (newCave || $("floor").options.length !== layout.floors) fillFloors(layout.floors, floor);
    current = layout;
    $("result").hidden = false;
    $("title").textContent = `${layout.long_name} · ${layout.seed}`;
    await render(layout);
    renderSummary(layout);
    setStatus(`${layout.sublevel}, seed ${layout.seed}. Generated in ${ms.toFixed(1)} ms.`);
    updateUrl(cave, floor, seed);
  } catch (e) {
    console.error(e);
    setStatus(`Couldn't generate this sublevel: ${e.message}`, true);
  }
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

async function init() {
  try {
    const { instance } = await WebAssembly.instantiateStreaming(fetch("cavegen3_web.wasm"), {});
    wasm = instance.exports;
    const resources = await fetchGz("data/resources.bin.gz");
    withBytes(resources, (p, n) => readResult(wasm.add_files(p, n)));
    caves = readResult(wasm.caves());
    const games = (await (await fetch("data/games.txt")).text()).split(/\s+/).filter(Boolean);
    // Not Object.keys(GAME_NAMES): JavaScript lists number-like keys such as "251" first.
    const order = ["pikmin2", "251", "newyear", "colossal", "216", "bikmin"];
    games.sort((a, b) => order.indexOf(a) - order.indexOf(b));
    caves = caves.filter((c) => games.includes(c.game));
    fillGames(games);
  } catch (e) {
    console.error(e);
    setStatus(`Couldn't start: ${e.message}. This page needs a recent browser.`, true);
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
  $("water").addEventListener("change", () => current && render(current));
  $("zoom").addEventListener("change", () => current && render(current));
  let resizeTimer;
  window.addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => current && $("zoom").value === "fit" && render(current), 150);
  });
  $("download").addEventListener("click", () => {
    if (!current) return;
    $("map").toBlob((blob) => {
      const a = el("a", { href: URL.createObjectURL(blob), download: `${current.sublevel}_${current.seed}.png` });
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    });
  });
  $("copy-link").addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(location.href);
      setStatus("Link copied.");
    } catch {
      setStatus(location.href);
    }
  });

  await run();
}

init();
