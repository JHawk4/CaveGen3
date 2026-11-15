// CaveGen3 aggregator page: CaveGen's Aggregator, plus grouping by comedown (what you see when
// you land). Uses the helpers in app.js, which is loaded first.
"use strict";

Object.assign(STRINGS.en, {
  aggIntro:
    "Generates many seeds of a sublevel, groups the ones that look alike, and shows how often each thing spawned at each spot.",
  aggGroupBy: "Group seeds by",
  aggGroupComedown: "Comedown: map units and objects in view",
  aggGroupComedownUnits: "Comedown: map units in view",
  aggGroupLayout: "Whole layout (CaveGen's default)",
  aggGroupRooms: "Rooms only",
  aggGroupFirst: "First room placed",
  aggConsecutive: "Up from the seed",
  aggGameRng: "Game RNG from the seed",
  aggRun: "Aggregate",
  aggShow: "Show",
  aggMinPct: "Hide below",
  aggMaxGroups: "Groups shown",
  aggHelp:
    "Each map is a group of seeds that look alike in any rotation. Icons show what spawned at each spot and how often; " +
    "gray shows how often other map units covered a cell; the dashed outline is the landing camera's view.",
  aggRunning: (done, total) => `Aggregating… ${done.toLocaleString()} of ${total.toLocaleString()} seeds`,
  aggDone: (total, groups, s) => `${total.toLocaleString()} seeds in ${s} s, ${groups.toLocaleString()} groups.`,
  aggStopped: (total) => `Stopped after ${total.toLocaleString()} seeds.`,
  aggDropped: (n) => ` ${n.toLocaleString()} seeds didn't fit in the first 20,000 groups and aren't shown.`,
  aggGroup: (i, pct, n) => `#${i} · ${pct}% (${n.toLocaleString()} seeds)`,
  aggExamples: "Examples:",
  aggNoGroups: "Nothing to show yet.",
  aggCarrying: (t) => ` (carrying ${t})`,
});

Object.assign(STRINGS.ja, {
  aggIntro: "ある階のシードをたくさん生成し、見た目が同じものをまとめて、各地点に何がどれくらいの割合で出たかを表示します。",
  aggGroupBy: "まとめ方",
  aggGroupComedown: "着地時の範囲：見えるマップユニットと物",
  aggGroupComedownUnits: "着地時の範囲：見えるマップユニットのみ",
  aggGroupLayout: "レイアウト全体（CaveGenの標準）",
  aggGroupRooms: "部屋のみ",
  aggGroupFirst: "最初に置かれた部屋",
  aggConsecutive: "シードから+1ずつ",
  aggGameRng: "シードから乱数順に",
  aggRun: "集計",
  aggShow: "表示",
  aggMinPct: "これ未満は非表示",
  aggMaxGroups: "表示するグループ数",
  aggHelp:
    "各マップは、回転を含めて見た目が同じシードのグループです。アイコンは各地点に出たものとその割合、" +
    "灰色はほかのマップユニットがそのマスにあった頻度、点線は着地直後のカメラの範囲です。",
  aggRunning: (done, total) => `集計中… ${done.toLocaleString()} / ${total.toLocaleString()} シード`,
  aggDone: (total, groups, s) => `${total.toLocaleString()} シードを${s}秒で集計、${groups.toLocaleString()} グループ。`,
  aggStopped: (total) => `${total.toLocaleString()} シードで停止しました。`,
  aggDropped: (n) => ` ${n.toLocaleString()} シードは最初の2万グループに入らず、表示していません。`,
  aggGroup: (i, pct, n) => `#${i} · ${pct}%（${n.toLocaleString()} シード）`,
  aggExamples: "例：",
  aggNoGroups: "まだ表示するものがありません。",
  aggCarrying: (t) => `（${t}を持っている）`,
});

const AGG_SETTINGS_KEY = "cavegen3-agg";
let agg = null; // the run in progress or last finished: {sublevel, grouping, result, ...}

/** What an object id from the aggregator ("t:Name|Carry", "i:Name", "g", "h", "y", "s") is. */
function parseObjectId(id, game) {
  if (id.startsWith("t:")) {
    const [name, carry] = id.slice(2).split("|");
    return { kind: "teki", name, carry, game };
  }
  if (id.startsWith("i:")) return { kind: "treasure", name: id.slice(2), game };
  return { kind: { g: "gate", h: "hole", y: "geyser", s: "ship" }[id] || id };
}

function objectShownAgg(o) {
  if (o.kind === "treasure") return $("show-treasures").checked;
  if (o.kind === "teki") return $("show-teki").checked || (o.carry && $("show-treasures").checked);
  if (o.kind === "gate") return $("show-gates").checked;
  return $("show-exits").checked;
}

function objectIconUrl(o) {
  switch (o.kind) {
    case "teki":
      return pickUrl(tekiIconUrls(o.name, o.game));
    case "treasure":
      return pickUrl(treasureIconUrls(o.name, o.game));
    case "gate":
      return specialUrl("Gray_bramble_gate_icon");
    case "hole":
      return specialUrl("Cave_icon");
    case "geyser":
      return specialUrl("Geyser_icon");
    case "ship":
      return specialUrl("pod_icon");
  }
  return null;
}

function objectName(o) {
  switch (o.kind) {
    case "teki":
      return displayName("teki", o.name) + (o.carry ? T.aggCarrying(displayName("treasures", o.carry)) : "");
    case "treasure":
      return displayName("treasures", o.name);
    case "gate":
      return T.gate;
    case "hole":
      return T.hole;
    case "geyser":
      return T.geyser;
    case "ship":
      return T.ship;
  }
  return o.kind;
}

function pct(n, total) {
  const p = (100 * n) / total;
  return p >= 10 ? Math.round(p).toString() : p >= 1 ? p.toFixed(1) : p.toPrecision(1);
}

/** What to draw at each spot of a group: objects shown, common enough, likeliest first. */
function groupSpots(group) {
  const game = agg.game;
  const minShare = Number($("agg-min").value) / 100;
  const spots = [];
  for (const [x, z, objs] of group.spots) {
    const items = objs
      .map(([id, n]) => ({ o: parseObjectId(id, game), n }))
      .filter(({ o, n }) => objectShownAgg(o) && n / group.count >= minShare);
    if (items.length) spots.push({ x, z, items });
  }
  // Likeliest last, so it ends up on top where spots crowd together.
  spots.sort((a, b) => a.items[0].n - b.items[0].n);
  return spots;
}

/** The comedown trapezoid's corners in a group's frame. */
function groupComedown(group) {
  if (!group.pod) return null;
  const [x, z, fx, fz] = group.pod;
  const [near, far, nearHalf, farHalf] = agg.result.view;
  const r = [-fz, fx];
  const at = (ahead, side) => [x + fx * ahead + r[0] * side, z + fz * ahead + r[1] * side];
  return [at(near, -nearHalf), at(near, nearHalf), at(far, farHalf), at(far, -farHalf)];
}

/** A group's extent in game units: its units, the cells others covered, its spots and view. */
function groupBounds(group) {
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  const grow = (x, z, r = 0) => {
    x0 = Math.min(x0, x - r);
    z0 = Math.min(z0, z - r);
    x1 = Math.max(x1, x + r);
    z1 = Math.max(z1, z + r);
  };
  for (const u of group.units) {
    grow(u.x - u.w * 85, u.z - u.h * 85);
    grow(u.x + u.w * 85, u.z + u.h * 85);
  }
  for (const [cx, cz] of group.cells) grow(cx * 85, cz * 85, 85);
  for (const [x, z] of group.spots) grow(x, z, 85);
  for (const [x, z] of groupComedown(group) || []) grow(x, z);
  return { x0: x0 - 85, z0: z0 - 85, x1: x1 + 85, z1: z1 + 85 };
}

/** Draws one group's map into its own canvas at `scale`, for the card's viewer. */
async function drawGroupContent(card, scale) {
  const { group } = card;
  const game = agg.game;
  const spots = groupSpots(group);
  const urls = new Set();
  for (const u of group.units) urls.add(pickUrl(unitUrls(u.name, u.game)));
  for (const s of spots) {
    for (const { o } of s.items) {
      urls.add(objectIconUrl(o));
      if (o.carry) urls.add(pickUrl(treasureIconUrls(o.carry, game)));
    }
  }
  urls.delete(null);
  await Promise.all([...urls].map(loadImage));

  const b = groupBounds(group);
  const cell = CELL * scale;
  const dpr = window.devicePixelRatio || 1;
  const px = (v) => (v / UNIT) * cell;
  const w = px(b.x1 - b.x0);
  const h = px(b.z1 - b.z0);
  const canvas = card.content;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  canvas.cssW = w;
  canvas.cssH = h;
  const ctx = canvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = "#0f0f0f";
  ctx.fillRect(0, 0, w, h);
  ctx.translate(px(-b.x0), px(-b.z0));

  // How often other units covered each cell.
  for (const [cx, cz, n] of group.cells) {
    ctx.fillStyle = `rgba(200, 200, 200, ${0.08 + 0.45 * (n / group.count)})`;
    ctx.fillRect(px(cx * 85 - 85), px(cz * 85 - 85), cell, cell);
  }

  // The group's own map units.
  ctx.imageSmoothingEnabled = false;
  for (const u of group.units) {
    const img = imageOf(unitUrls(u.name, u.game));
    ctx.save();
    ctx.translate(px(u.x), px(u.z));
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

  // What the camera sees when you land.
  const view = groupComedown(group);
  if (view) {
    ctx.strokeStyle = "#ffe14d";
    ctx.lineWidth = Math.max(1.5, cell / 16);
    ctx.setLineDash([cell / 4, cell / 6]);
    ctx.beginPath();
    for (const [x, z] of view) ctx.lineTo(px(x), px(z));
    ctx.closePath();
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // Each spot's objects, stacked, sized by how often they spawned there (as CaveGen does).
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  card.hits = [];
  for (const s of spots) {
    const sizes = s.items.map(({ n }) => cell * 0.55 * (0.5 + 0.5 * (n / group.count)));
    let y = px(s.z) - sizes.reduce((a, c) => a + c, 0) / 2;
    const x = px(s.x);
    s.items.forEach(({ o, n }, i) => {
      const size = sizes[i];
      const cy = y + size / 2;
      const img = loaded.get(objectIconUrl(o));
      if (img) {
        const r = Math.min(size / img.width, size / img.height);
        ctx.drawImage(img, x - (img.width * r) / 2, cy - (img.height * r) / 2, img.width * r, img.height * r);
      } else {
        ctx.fillStyle = o.kind === "treasure" ? "#c70" : "#7a2a8a";
        ctx.beginPath();
        ctx.arc(x, cy, size / 2.5, 0, Math.PI * 2);
        ctx.fill();
      }
      if (o.carry) {
        const t = loaded.get(pickUrl(treasureIconUrls(o.carry, game)));
        if (t) {
          const r = Math.min((size * 0.6) / t.width, (size * 0.6) / t.height);
          ctx.drawImage(t, x + size * 0.1, cy + size * 0.05, t.width * r, t.height * r);
        }
      }
      if (cell >= 14) {
        const text = `${pct(n, group.count)}%`;
        const fs = Math.max(9, cell * 0.24);
        ctx.font = `bold ${fs}px system-ui, sans-serif`;
        ctx.textAlign = "left";
        ctx.textBaseline = "middle";
        ctx.lineJoin = "round";
        ctx.lineWidth = Math.max(2, fs / 4);
        ctx.strokeStyle = "#000";
        ctx.strokeText(text, x + size / 2 + 1, cy);
        ctx.fillStyle = o.kind === "treasure" || o.carry ? "#ffd75a" : "#fff";
        ctx.fillText(text, x + size / 2 + 1, cy);
      }
      y += size;
    });
    const all = group.spots.find(([sx, sz]) => sx === s.x && sz === s.z)[2];
    const label = all
      .map(([id, n]) => ({ o: parseObjectId(id, game), n }))
      .filter(({ o }) => objectShownAgg(o))
      .map(({ o, n }) => `${objectName(o)}: ${pct(n, group.count)}%`)
      .join("\n");
    card.hits.push({ x: x + px(-b.x0), z: px(s.z) + px(-b.z0), r: Math.max(cell * 0.4, 10), label });
  }
  return canvas;
}

function groupCard(group, index) {
  const total = agg.result.total;
  const canvas = el("canvas", { class: "agg-map" });
  const pane = el("div", { class: "agg-pane" }, canvas);
  const examples = el("span", { class: "agg-examples" }, T.aggExamples);
  for (const seed of group.seeds) {
    const p = new URLSearchParams({ sublevel: agg.sublevelParam, seed: formatSeed(seed) });
    if (agg.grouping.startsWith("comedown")) p.set("comedown", "1");
    examples.append(" ", el("a", { href: `./?${p}`, class: "seed-link" }, formatSeed(seed)));
  }
  const card = { group, content: document.createElement("canvas"), hits: [] };
  card.box = el(
    "section",
    { class: "agg-card" },
    el("h3", {}, T.aggGroup(index + 1, pct(group.count, total), group.count)),
    examples,
    pane,
  );
  card.viewer = createViewer(pane, canvas, {
    size: () => {
      const b = groupBounds(group);
      const view = { w: (b.x1 - b.x0) / UNIT, h: (b.z1 - b.z0) / UNIT };
      return { w: view.w * CELL, h: view.h * CELL, max: maxScale(view), maxFit: 1.5 };
    },
    draw: (scale) => drawGroupContent(card, scale),
    onChange: () => ($("tooltip").hidden = true),
  });
  const tip = $("tooltip");
  pane.addEventListener("mousemove", (e) => {
    const at = card.viewer.toContent(e.clientX, e.clientY);
    const k = card.viewer.cscale / card.viewer.scale;
    const hit = card.hits.findLast((t) => Math.hypot(t.x - at.x, t.z - at.y) <= Math.max(t.r, 8 * k));
    if (!hit) {
      tip.hidden = true;
      return;
    }
    tip.textContent = hit.label;
    tip.hidden = false;
    tip.style.left = `${Math.min(e.pageX + 12, document.documentElement.scrollWidth - tip.offsetWidth - 8)}px`;
    tip.style.top = `${e.pageY + 14}px`;
  });
  pane.addEventListener("mouseleave", () => (tip.hidden = true));
  return card;
}

let cards = [];
async function showResults() {
  const box = $("agg-results");
  if (!agg || !agg.result) return;
  for (const c of cards) c.viewer.destroy();
  if (!agg.result.groups.length) {
    cards = [];
    box.replaceChildren(el("p", { class: "muted" }, T.aggNoGroups));
    return;
  }
  cards = agg.result.groups.map((g, i) => groupCard(g, i));
  box.replaceChildren(...cards.map((c) => c.box));
  for (const c of cards) await c.viewer.refresh(true);
}

async function redrawCards() {
  for (const c of cards) await c.viewer.refresh();
}

/**
 * Which map units look the same turned (period 1: a quarter turn, 2: a half turn, 4: never),
 * judged from their radar images, as "name=period;...". The aggregator also checks their shape.
 */
async function imagePeriods(sublevel) {
  const ci = await command("caveinfo", sublevel);
  const out = [];
  const imgs = await Promise.all(ci.units.map((u) => loadImage(pickUrl(unitUrls(u.name, u.game)))));
  ci.units.forEach((u, i) => {
    const img = imgs[i];
    if (!img) return;
    const c = unitImage(img);
    const d = c.getContext("2d", { willReadFrequently: true }).getImageData(0, 0, c.width, c.height).data;
    const [W, H] = [c.width, c.height];
    const v = (x, y) => d[(y * W + x) * 4 + 3];
    const mismatch = (map) => {
      let bad = 0;
      for (let y = 0; y < H; y++) {
        for (let x = 0; x < W; x++) {
          const [x2, y2] = map(x, y);
          if (Math.abs(v(x, y) - v(x2, y2)) > 40) bad++;
        }
      }
      return bad / (W * H);
    };
    let period = 4;
    if (W === H && mismatch((x, y) => [y, W - 1 - x]) < 0.03) period = 1;
    else if (mismatch((x, y) => [W - 1 - x, H - 1 - y]) < 0.03) period = 2;
    out.push(`${u.name}=${period}`);
  });
  return out.join(";");
}

function stopAgg() {
  if (agg) agg.stopped = true;
  $("agg-go").textContent = T.aggRun;
}

async function runAgg() {
  if (agg && !agg.done) {
    stopAgg();
    return;
  }
  const sel = selected();
  if (!sel) return;
  const grouping = $("agg-group").value;
  const total = Math.max(1, Math.min(1e7, Math.floor(Number($("agg-count").value) || 0)));
  const mode = $("agg-mode").value;
  const startAt = mode === "r" ? randomSeed() : (parseSeed($("seed").value) ?? 0);
  const sublevel = sublevelName(sel.cave, sel.floor);
  const run = {
    sublevel,
    sublevelParam: `${sel.cave.game}:${sel.cave.short}-${sel.floor}`,
    game: sel.cave.game,
    grouping,
    done: false,
    stopped: false,
    result: null,
  };
  agg = run;
  $("agg-go").textContent = T.stop;
  const t0 = performance.now();
  try {
    await loadGame(sel.cave.game);
    const shape = comedownShape();
    const periods = await imagePeriods(sublevel);
    await command("agg_start", sublevel, grouping, shape.near, shape.far, shape.nearHalf, shape.farHalf, periods);
    let doneSeeds = 0;
    let chunk = 50;
    let lastShown = performance.now();
    const max = Number($("agg-max").value) || 20;
    while (doneSeeds < total && !run.stopped) {
      const n = Math.min(chunk, total - doneSeeds);
      const t = performance.now();
      await command("agg_run", startAt + doneSeeds, n, mode);
      doneSeeds += n;
      chunk = Math.max(10, Math.min(100000, Math.round((n * 80) / Math.max(1, performance.now() - t))));
      setStatus(T.aggRunning(doneSeeds, total));
      // Show the groups so far every couple of seconds on long runs.
      if (performance.now() - lastShown > 2500) {
        run.result = await command("agg_result", max);
        showResults(); // not awaited: drawing waits for images, the aggregation needn't
        lastShown = performance.now();
      }
      await yieldToBrowser();
    }
    run.result = await command("agg_result", max);
    await showResults();
    const secs = ((performance.now() - t0) / 1000).toFixed(1);
    let text = run.stopped ? T.aggStopped(run.result.total) : T.aggDone(run.result.total, run.result.groups_total, secs);
    if (run.result.dropped) text += T.aggDropped(run.result.dropped);
    setStatus(text);
  } catch (e) {
    console.error(e);
    setStatus(e.message, true);
  } finally {
    run.done = true;
    if (agg === run) $("agg-go").textContent = T.aggRun;
  }
}

/** The floor list for the selected cave, from generating one layout. */
async function refreshFloors(floor = 1) {
  const cave = caves[Number($("cave").value)];
  if (!cave) return;
  try {
    await loadGame(cave.game);
    const layout = await generate(sublevelName(cave, floor), 0, 0);
    fillFloors(layout.floors, floor);
    setStatus("");
  } catch (e) {
    setStatus(T.cantGenerate(e.message), true);
  }
}

function saveAggSettings() {
  try {
    const s = {};
    for (const id of ["agg-group", "agg-count", "agg-mode", "agg-min", "agg-max"]) s[id] = $(id).value;
    for (const id of ["show-treasures", "show-teki", "show-gates", "show-exits"]) s[id] = $(id).checked;
    localStorage.setItem(AGG_SETTINGS_KEY, JSON.stringify(s));
    const shared = JSON.parse(localStorage.getItem(SETTINGS_KEY) || "{}");
    shared.comedownScreen = $("comedown-screen").value;
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(shared));
  } catch {
    // Not saved; the choices still apply to this page.
  }
}

function loadAggSettings() {
  try {
    const s = JSON.parse(localStorage.getItem(AGG_SETTINGS_KEY) || "{}");
    for (const [id, v] of Object.entries(s)) {
      const node = $(id);
      if (!node) continue;
      if (typeof v === "boolean") node.checked = v;
      else node.value = v;
    }
    const shared = JSON.parse(localStorage.getItem(SETTINGS_KEY) || "{}");
    if (shared.comedownScreen) $("comedown-screen").value = shared.comedownScreen;
  } catch {
    // Nothing saved, or storage is unavailable.
  }
}

function updateFieldVisibility() {
  $("comedown-size-field").hidden = !$("agg-group").value.startsWith("comedown");
  $("seed-field").hidden = $("agg-mode").value === "r";
}

async function initAggregator() {
  loadAggSettings();
  applyLanguage();
  setupHeader();
  updateFieldVisibility();
  languageHook = async () => {
    if (agg && agg.result) await showResults();
  };
  setStatus(T.loading);
  try {
    await startCommon();
  } catch (e) {
    console.error(e);
    setStatus(T.cantStart(e.message), true);
    return;
  }
  selectStart();
  const fromUrl = params.get("sublevel") && selectFromParam(params.get("sublevel"));
  await refreshFloors(fromUrl ? fromUrl.floor : 1);

  $("game").addEventListener("change", () => {
    fillCaves($("game").value);
    $("cave").selectedIndex = 0;
    refreshFloors();
  });
  $("cave").addEventListener("change", () => refreshFloors());
  $("controls").addEventListener("submit", (e) => {
    e.preventDefault();
    runAgg();
  });
  for (const id of ["agg-group", "agg-count", "agg-mode", "comedown-screen", "agg-max"]) {
    $(id).addEventListener("change", () => {
      saveAggSettings();
      updateFieldVisibility();
    });
  }
  for (const id of ["show-treasures", "show-teki", "show-gates", "show-exits", "agg-min"]) {
    $(id).addEventListener("change", () => {
      saveAggSettings();
      redrawCards();
    });
  }
}
