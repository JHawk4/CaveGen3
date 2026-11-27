// CaveGen3 web page: loads the WebAssembly generator and draws layouts on a canvas.
"use strict";

// ---- Text in both languages ----

const STRINGS = {
  en: {
    game: "Game",
    cave: "Cave",
    sublevel: "Sublevel",
    seed: "Seed",
    prevSeed: "Previous seed",
    nextSeed: "Next seed",
    seedTitle: "Up to 8 hex digits",
    seedTitleSwitch: "Up to 16 hex digits (Switch)",
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
    generatedIn: (ms) => `Generated in ${ms} ms`,
    badSeed: "The seed should be up to 8 hex digits, like 1234ABCD.",
    badSeedSwitch: "A Switch seed should be up to 16 hex digits, like 306306F66F0443C4.",
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
    colossalOptions: "Colossal Caverns options",
    colUltra: "Ultra randomizer",
    colExtra: "Extra units",
    col5: "5 Onions",
    viewPod: "Landing view only (pod)",
    viewOlimar: "Landing view only (Olimar)",
    viewLouie: "Landing view only (Louie)",
    viewComedown: "Comedown view only",
    screen: "Screen",
    seedStep: "‹ › buttons",
    seedStepPlus: "±1",
    seedStepGame: "Game RNG",
    gameRngSeeds: "Game RNG from the seed",
    zoomIn: "Zoom in",
    zoomOut: "Zoom out",
    zoomFit: "Fit the map to the window",
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
    secObjects: "Objects",
    secView: "View",
    secGeneration: "Generation details",
    secScores: "Scores",
    secSeeds: "Seeds",
    platform: "Game version",
    platformGc: "GameCube / Wii",
    platformSwitch: "Switch",
    opt: {
      treasures: "Treasures",
      hideBuried: "Hide fully buried treasures",
      teki: "Teki",
      plants: "Plants",
      gates: "Gates",
      gateHealth: "Gate health",
      exits: "Pod and exits",
      comedownVisible: "Comedown-visible only",
      spawnpoints: "Spawn points",
      spawnOrder: "Spawn order",
      pathDists: "Carry distances",
      podAngle: "Pod facing",
      angles: "Facing directions",
      unitScores: "Unit scores",
      spawnScores: "Treasure and hole scores",
      holeOdds: "Hole odds",
      tekiScores: "Teki scores",
      doorScores: "Door scores",
      doorLinks: "Door links",
      doorIds: "Door numbers",
      waypoints: "Waypoints",
      paths: "Carry paths",
      gauge: "Treasure gauge",
      grid: "Grid",
      landingBox: "Landing outline (pod)",
      landingBoxOlimar: "Landing outline (Olimar)",
      landingBoxLouie: "Landing outline (Louie)",
      comedownBox: "Comedown outline",
      quickGlance: "QuickGlance",
      pretty: "Pretty",
    },
    judge: "Judge",
    judgeNone: "None",
    judgeNames: {
      score: "score: game's own treasure scores",
      pod: "pod: Pay off Debt",
      judge2: "judge2: Pay off Debt, refit",
      at: "at: All Treasures",
      attk: "attk: Challenge Mode score attack",
      cmat: "cmat: Challenge Mode all treasures",
      key: "key: Challenge Mode key and exit",
      colossal: "colossal: Colossal Caverns",
      mapunitcount: "mapunitcount: number of map units",
    },
    judgeTips: {
      score: "The game's own unit and treasure scores, as CaveGen judges them",
      pod: "Estimated seconds to collect this sublevel's treasures in a Pay off Debt run",
      judge2: "Pay off Debt, refit against timed runs: expected net seconds, letting the runner skip treasures that aren't worth the trip",
      at: "Estimated seconds to collect every treasure in an All Treasures run",
      attk: "Challenge Mode score attack estimate",
      cmat: "Challenge Mode all-treasures estimate",
      key: "Challenge Mode estimate for getting the key and reaching the exit",
      colossal: "Colossal Caverns estimate",
      mapunitcount: "The number of map units; fewer usually means a faster floor",
    },
    judgeLine: (type, score) => `Judge (${type}): ${score}`,
    likelySkipped: (list) => `Likely skipped: ${list}`,
    search: "Seed search",
    searchCount: "Seeds to test",
    searchMode: "Seeds",
    consecutive: "Up from the seed",
    randomSeeds: "Random",
    query: "Query (optional)",
    queryPlaceholder: "e.g. hole carry dist < 1000",
    keep: "Show best",
    searchButton: "Search",
    stop: "Stop",
    searching: (done, total) => `Searching… ${done.toLocaleString()} of ${total.toLocaleString()} seeds`,
    searchDone: (tested, matched, s) =>
      `Tested ${tested.toLocaleString()} seeds in ${s} s` + (matched === null ? "." : `; ${matched.toLocaleString()} matched.`),
    searchStopped: (tested) => `Stopped after ${tested.toLocaleString()} seeds.`,
    noMatches: "No seeds matched.",
    needJudgeOrQuery: "Pick a judge, write a query, or name something to count first.",
    countWhat: "Count (optional)",
    matchedShare: (matched, tested) => `${matched.toLocaleString()} of ${tested.toLocaleString()} seeds matched (${((100 * matched) / Math.max(1, tested)).toFixed(2)}%).`,
    countTitle: (what, tested) => `How many ${what} (${tested.toLocaleString()} seeds)`,
    rank: (pct) => `top ${pct}%`,
    querySyntax: "syntax",
    tips: {
      colUltra: "Shuffle the map unit lists much more before placing them (CaveGen's -ultraRandomizer)",
      colExtra: "Also use Pikmin 251's map units (CaveGen's -extraUnits)",
      col5: "Place five Onions instead of three (CaveGen's -5onion)",
      viewPod: "Show only what's on screen when you land after pressing L, which turns the camera the way the pod faces. Turned as on screen",
      viewOlimar: "Show only what's on screen when you land as Olimar without pressing L: the camera faces the same way on every floor. Turned as on screen",
      viewLouie: "Show only what's on screen when you land as Louie without pressing L: the camera faces the same way on every floor. Turned as on screen",
      viewComedown: "Show only what's on screen as the pod comes down, while the sublevel's name is up, turned as on screen",
      water: "Shade the water",
      screen: "The game's screen setting, for the landing and comedown views. A 16:9 screen shows more to the sides",
      seedStep: "±1, or the seed before/after it in the game's RNG (CaveGen's -consecutiveSeeds)",
      platform:
        "Which release's random number generator makes the layouts. GameCube, Wii and Wii U share one with 8-digit seeds; " +
        "the Switch release (Pikmin 2 HD) uses a 64-bit one, so its seeds have 16 hex digits. " +
        "The Switch generator was worked out from gameplay video and its layouts haven't been checked against the game yet",
      judge: "Rate each layout with one of CaveGen's judges. Lower is better; most estimate the seconds a kind of run takes",
      search: "Test many seeds of this sublevel, then list the best by the judge, or the ones matching the query",
      searchCount: "How many seeds to generate and check",
      searchMode: "Up from the seed (+1 each), random, or each next seed in the game's RNG",
      keep: "How many of the best seeds to list",
      countWhat: "Tally how many of something each tested seed has, like CaveGen's -count: a teki or treasure by internal name (minihoudai, ahiru), or teki, treasure, gate or onion. Searching then tests every seed",
      query: "Keep only seeds matching this condition, in Caveripper's query language, e.g. hole carry dist < 1000. The sublevel can be left out",
      caveInfo: "What this sublevel can generate: its limits, and the weights the generator picks teki, treasures and map units by",
      opt: {
        treasures: "Treasure icons",
        hideBuried: "Hide treasures that start completely under the ground (as CaveGen's -drawNoBuriedItems)",
        teki: "Teki icons, with what they carry and whether they fall when approached. Plants aren't teki here, wherever they spawned; Candypop Buds, Pellet Posies and hazards are",
        plants: "Plant icons: Clovers, Glowcaps, Glowstems, Figworts, Shoots, Margarets and the like, also when the cave put one in a teki spot",
        gates: "Gates",
        gateHealth: "Each gate's HP, written on the gate",
        exits: "The pod, the hole or geyser, and Onions",
        comedownVisible:
          "Only what the game draws as the pod comes down: treasures (a buried one shows as its mound of dirt), gates, the hole or geyser, the pod, " +
          "plants, and fire, electric and gas hazards. The cave intro doesn't draw other teki, so no enemies, eggs or Candypop Buds, nor the treasures they carry. " +
          "Only the floor near the pod is lit",
        spawnpoints: "Every spawn point, marked by what can spawn there as in Caveripper: pink rings for easy teki, red dots for hard teki, a duck for treasures, a hole for the exit, a leaf for plants, a star for special teki, the ship for the pod. Faded ones are empty",
        angles: "Arrows for the way each object and spawn point faces",
        spawnOrder: "The order things were placed in: map units (outlined), then the pod, exits, teki, treasures, alcove teki and gates (CaveGen's -drawSpawnOrder)",
        pathDists: "How far each treasure is carried to the pod, in tens of units along its path (CaveGen's -drawPathDists)",
        podAngle: "An arrow for the way the pod faces, which is the way the camera looks when you land (CaveGen's -drawPodAngle)",
        unitScores: "Each map unit's score, which the generator uses to place the hole and treasures",
        spawnScores: "The scores used to pick the treasure spots (orange) and the hole (green)",
        holeOdds: "For Challenge Mode, where the seed is unknown: the chance of the hole (or geyser) being at each alcove and dead end, from the game's weights (√unit score + 10) and what was placed after it: spots holding something, teki that keep away from it, treasures and cap teki that passed an alcove up, and gates in front of alcoves",
        tekiScores: "The teki score in each map unit (red)",
        doorScores: "Each door's score (blue)",
        doorLinks: "Lines between linked doors in the same map unit",
        doorIds: "Each door's number within its map unit",
        waypoints: "The carry waypoints, with arrows toward the pod",
        paths: "The way each treasure is carried back to the pod",
        gauge: "Shades the floor by how high the treasure gauge's needle would be there: yellow while the needle is below 0.3, orange from 0.3 and red from 0.8, deeper each step inward (steps at 0.05, 0.1, 0.2 … 0.9)",
        grid: "The map's grid cells",
        landingBox: "An outline of what the camera shows when you land after pressing L, which turns the camera the way the pod faces (white)",
        landingBoxOlimar: "An outline of what the camera shows when you land as Olimar without pressing L (red)",
        landingBoxLouie: "An outline of what the camera shows when you land as Louie without pressing L (blue)",
        comedownBox: "An outline of what the camera shows as the pod comes down, while the sublevel's name is on screen (yellow)",
        quickGlance: "Colored discs behind what matters, to read a layout at a glance: the pod (red), treasures and teki carrying them (orange), the exit (green), Gatling Groinks (pink), Candypop Buds and Onions (CaveGen's QuickGlance)",
        pretty: "Draws the vanilla map units as pictures of the actual floor instead of radar images (CaveGen's Pretty)",
      },
    },
    caveInfo: "CaveInfo",
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
    ciScore: (n) => `Score: ${n}`,
    ciCarryTip: "Pikmin needed to carry it / most that can carry it",
    ciGroups: { special: "Special", hard: "Hard", easy: "Easy", plant: "Plant", seam: "Seam", other: "Other", cap: "Alcove", treasure: "Treasure" },
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
      newyearkaizo: "Pikmin 2: New Year Kaizo",
      colossal: "Colossal Caverns",
      "216": "Pikmin 216",
      bikmin: "Bikmin 269",
      torture: "Lands of Torture",
    },
  },
  ja: {
    game: "ゲーム",
    cave: "洞窟",
    sublevel: "階",
    seed: "シード",
    prevSeed: "前のシード",
    nextSeed: "次のシード",
    seedTitle: "16進数8桁まで",
    seedTitleSwitch: "16進数16桁まで（Switch版）",
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
    generatedIn: (ms) => `${ms}ミリ秒で生成`,
    badSeed: "シードは 1234ABCD のような16進数8桁までで入力してください。",
    badSeedSwitch: "Switch版のシードは 306306F66F0443C4 のような16進数16桁までで入力してください。",
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
    colossalOptions: "Colossal Caverns のオプション",
    colUltra: "超ランダム",
    colExtra: "追加ユニット",
    col5: "オニヨン5個",
    viewPod: "着地時の範囲だけ（ポッド）",
    viewOlimar: "着地時の範囲だけ（オリマー）",
    viewLouie: "着地時の範囲だけ（ルーイー）",
    viewComedown: "降下中の範囲だけ",
    screen: "画面",
    seedStep: "‹ › ボタン",
    seedStepPlus: "±1",
    seedStepGame: "ゲームの乱数",
    gameRngSeeds: "シードから乱数順に",
    zoomIn: "拡大",
    zoomOut: "縮小",
    zoomFit: "ウィンドウに合わせる",
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
    secObjects: "表示するもの",
    secView: "表示方法",
    secGeneration: "生成の詳細",
    secScores: "スコア",
    secSeeds: "シード",
    platform: "ゲームの版",
    platformGc: "ゲームキューブ / Wii",
    platformSwitch: "Switch",
    opt: {
      treasures: "お宝",
      hideBuried: "完全に埋まったお宝を隠す",
      teki: "原生生物",
      plants: "植物",
      gates: "ゲート",
      gateHealth: "ゲートの体力",
      exits: "ポッドと出口",
      comedownVisible: "降下中に見えるものだけ",
      spawnpoints: "出現ポイント",
      spawnOrder: "生成順",
      pathDists: "運搬距離",
      podAngle: "ポッドの向き",
      angles: "向き",
      unitScores: "ユニットのスコア",
      spawnScores: "お宝と穴のスコア",
      holeOdds: "穴の確率",
      tekiScores: "原生生物のスコア",
      doorScores: "ドアのスコア",
      doorLinks: "ドアのつながり",
      doorIds: "ドア番号",
      waypoints: "ウェイポイント",
      paths: "運搬ルート",
      gauge: "お宝ゲージ",
      grid: "グリッド",
      landingBox: "着地時の範囲の枠（ポッド）",
      landingBoxOlimar: "着地時の範囲の枠（オリマー）",
      landingBoxLouie: "着地時の範囲の枠（ルーイー）",
      comedownBox: "降下中の範囲の枠",
      quickGlance: "クイックグランス",
      pretty: "きれいな表示",
    },
    judge: "評価",
    judgeNone: "なし",
    judgeNames: {
      score: "score：ゲーム内のお宝スコア",
      pod: "pod：借金返済",
      judge2: "judge2：借金返済（再調整版）",
      at: "at：お宝全回収",
      attk: "attk：チャレンジモードのスコアアタック",
      cmat: "cmat：チャレンジモードのお宝全回収",
      key: "key：チャレンジモードのカギと出口",
      colossal: "colossal：Colossal Caverns",
      mapunitcount: "mapunitcount：マップユニットの数",
    },
    judgeTips: {
      score: "ゲーム内のユニットとお宝のスコア（CaveGenの評価）",
      pod: "借金返済で、この階のお宝を集めるまでの見積もり秒数",
      judge2: "実際の借金返済の走りに合わせて調整した見積もり秒数。割に合わないお宝は取らずに進む場合も考えます",
      at: "お宝全回収で、すべてのお宝を集めるまでの見積もり秒数",
      attk: "チャレンジモードのスコアアタックの見積もり",
      cmat: "チャレンジモードのお宝全回収の見積もり",
      key: "チャレンジモードでカギを取って出口に着くまでの見積もり",
      colossal: "Colossal Caverns の見積もり",
      mapunitcount: "マップユニットの数（少ないほど速いことが多い）",
    },
    judgeLine: (type, score) => `評価（${type}）：${score}`,
    likelySkipped: (list) => `取らない可能性が高いお宝：${list}`,
    search: "シード検索",
    searchCount: "試すシードの数",
    searchMode: "シード",
    consecutive: "シードから+1ずつ",
    randomSeeds: "ランダム",
    query: "条件（任意）",
    queryPlaceholder: "例：hole carry dist < 1000",
    keep: "上位を表示",
    searchButton: "検索",
    stop: "停止",
    searching: (done, total) => `検索中… ${done.toLocaleString()} / ${total.toLocaleString()} シード`,
    searchDone: (tested, matched, s) =>
      `${tested.toLocaleString()} シードを${s}秒で調べ` +
      (matched === null ? "ました。" : `、${matched.toLocaleString()} 件が条件に合いました。`),
    searchStopped: (tested) => `${tested.toLocaleString()} シードで停止しました。`,
    noMatches: "条件に合うシードはありませんでした。",
    needJudgeOrQuery: "先に評価を選ぶか、条件か数えるものを入力してください。",
    countWhat: "数える（任意）",
    matchedShare: (matched, tested) => `${tested.toLocaleString()} シード中 ${matched.toLocaleString()} が条件に合いました（${((100 * matched) / Math.max(1, tested)).toFixed(2)}%）。`,
    countTitle: (what, tested) => `${what} の数（${tested.toLocaleString()} シード）`,
    rank: (pct) => `上位 ${pct}%`,
    querySyntax: "書き方",
    tips: {
      colUltra: "マップユニットの並びを配置前にもっと大きくかき混ぜます（CaveGenの -ultraRandomizer）",
      colExtra: "Pikmin 251 のマップユニットも使います（CaveGenの -extraUnits）",
      col5: "オニヨンを3個ではなく5個置きます（CaveGenの -5onion）",
      viewPod: "Lボタンでカメラをポッドの向きに合わせたときに、着地時に画面に映る範囲だけを、画面と同じ向きで表示します",
      viewOlimar: "オリマーでLボタンを押さずに着地したときに画面に映る範囲だけを、画面と同じ向きで表示します。カメラはどの階でも同じ向きです",
      viewLouie: "ルーイーでLボタンを押さずに着地したときに画面に映る範囲だけを、画面と同じ向きで表示します。カメラはどの階でも同じ向きです",
      viewComedown: "ポッドが降りてくる間（階層名が表示されている間）に画面に映る範囲だけを、画面と同じ向きで表示します",
      water: "水の部分に色をつけます",
      screen: "ゲームの画面設定（着地時と降下中の範囲用）。16:9のほうが左右が広く映ります",
      seedStep: "±1、またはゲームの乱数で前後のシード（CaveGenの -consecutiveSeeds）",
      platform:
        "レイアウトを作る乱数の種類。ゲームキューブ・Wii・Wii Uは共通で、シードは8桁です。" +
        "Switch版（Pikmin 2 HD）は64ビットの乱数を使うため、シードは16進数16桁です。" +
        "Switch版の乱数はプレイ動画から割り出したもので、レイアウトはまだ実機と照合していません",
      judge: "CaveGenの評価で各レイアウトを採点します。小さいほど良く、多くは走り方ごとの秒数の見積もりです",
      search: "この階のシードをたくさん試して、評価の良いものや条件に合うものを一覧にします",
      searchCount: "生成して調べるシードの数",
      searchMode: "シードから+1ずつ、ランダム、またはゲームの乱数で次々のシード",
      keep: "一覧に出す上位のシードの数",
      countWhat: "調べた各シードに何がいくつあるかを数えます（CaveGenの -count）。内部名（minihoudai, ahiru など）か、teki・treasure・gate・onion。指定するとすべてのシードを調べます",
      query: "Caveripperの検索条件に合うシードだけを残します（例：hole carry dist < 1000）。階の名前は省略できます",
      caveInfo: "この階で生成されうるもの：上限と、原生生物・お宝・マップユニットが選ばれる重み",
      opt: {
        treasures: "お宝のアイコン",
        hideBuried: "最初から完全に地中に埋まっているお宝を隠します（CaveGenの -drawNoBuriedItems）",
        teki: "原生生物のアイコン（持っているお宝、近づくと落ちてくるかも表示）。植物はどこに出ても含みません。ポンガシグサ、ペレット草、火・電気・ガスの仕掛けは含みます",
        plants: "植物のアイコン（クローバー、ヒカリキノコ、ヒカリエノキ、モモフグリ、若芽、マーガレットなど）。原生生物の地点に出た植物も含みます",
        gates: "ゲート",
        gateHealth: "各ゲートの体力をゲートの上に表示します",
        exits: "ポッド、穴か間欠泉、オニヨン",
        comedownVisible:
          "ポッドが降りてくる間にゲームが描くものだけを表示します：お宝（埋まったお宝は土の盛り上がりとして）、ゲート、穴か間欠泉、ポッド、植物、火・電気・毒ガスの噴出口。" +
          "洞窟に入るときのムービーではそれ以外の原生生物が描かれないため、敵・タマゴ・ポンガシグサと、それらが持つお宝は表示しません。明るいのはポッドの近くの床だけです",
        spawnpoints: "すべての出現ポイント（Caveripperと同じ記号：ピンクの輪は弱い原生生物、赤い点は強い原生生物、アヒルはお宝、穴は出口、葉は植物、星は特殊、船はポッド。薄いものは空）",
        angles: "各オブジェクトと出現ポイントの向きの矢印",
        spawnOrder: "置かれた順番：マップユニット（枠）、ポッド、出口、原生生物、お宝、行き止まりの原生生物、ゲート（CaveGenの -drawSpawnOrder）",
        pathDists: "各お宝がポッドまで運ばれる道のりの長さ（10単位ごと、CaveGenの -drawPathDists）",
        podAngle: "ポッドの向きの矢印。着地したときにカメラが向く方向です（CaveGenの -drawPodAngle）",
        unitScores: "各マップユニットのスコア（穴とお宝の配置に使われます）",
        spawnScores: "お宝（オレンジ）と穴（緑）の場所選びに使われるスコア",
        holeOdds: "チャレンジモードなどシードが分からないとき：各くぼみと行き止まりに穴（または間欠泉）がある確率。ゲームの重み（√ユニットスコア＋10）と、その後に置かれたもの（何かがある場所、離れて置かれる敵、そのくぼみを避けたお宝や小部屋の敵、くぼみの前のゲート）から計算します",
        tekiScores: "各マップユニットの原生生物スコア（赤）",
        doorScores: "各ドアのスコア（青）",
        doorLinks: "同じマップユニット内でつながっているドアどうしの線",
        doorIds: "マップユニット内のドア番号",
        waypoints: "運搬のウェイポイントと、ポッドへ向かう矢印",
        paths: "各お宝がポッドまで運ばれる道すじ",
        gauge: "その場所でお宝ゲージの針がどこまで上がるかで床を塗り分けます。針が0.3未満は黄色、0.3からオレンジ、0.8から赤で、内側ほど濃くなります（0.05, 0.1, 0.2 … 0.9 ごと）",
        grid: "マップのマス目",
        landingBox: "Lボタンでカメラをポッドの向きに合わせたときに、着地時にカメラに映る範囲の枠（白）",
        landingBoxOlimar: "オリマーでLボタンを押さずに着地したときにカメラに映る範囲の枠（赤）",
        landingBoxLouie: "ルーイーでLボタンを押さずに着地したときにカメラに映る範囲の枠（青）",
        comedownBox: "ポッドが降りてくる間（階層名が表示されている間）にカメラに映る範囲の枠（黄）",
        quickGlance: "大事なものの下に色の円を描いて、ひと目で読めるようにします：ポッド（赤）、お宝とお宝を持つ原生生物（オレンジ）、出口（緑）、ガトリングモドキ（ピンク）、ポンガシグサ、オニヨン（CaveGenの QuickGlance）",
        pretty: "元のゲームのマップユニットを、レーダー画像ではなく実際の床の絵で描きます（CaveGenの Pretty）",
      },
    },
    caveInfo: "CaveInfo",
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
    ciScore: (n) => `スコア：${n}`,
    ciCarryTip: "運ぶのに必要なピクミンの数 / 運べる最大の数",
    ciGroups: { special: "特殊", hard: "強", easy: "弱", plant: "植物", seam: "境目", other: "その他", cap: "行き止まり", treasure: "お宝" },
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
      newyearkaizo: "Pikmin 2: New Year Kaizo",
      colossal: "Colossal Caverns",
      "216": "Pikmin 216",
      bikmin: "Bikmin 269",
      torture: "Lands of Torture",
    },
  },
};

// Not Object.keys(gameNames): JavaScript lists number-like keys such as "251" first.
const GAME_ORDER = ["pikmin2", "251", "newyear", "newyearkaizo", "colossal", "216", "bikmin", "torture"];

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
  if (typeof seed === "bigint") {
    // A Switch seed: 64 bits, passed as two halves.
    const lo = Number(seed & 0xffffffffn);
    const hi = Number((seed >> 32n) & 0xffffffffn);
    return guarded(() => withBytes(name, (p, n) => readResult(wasm.generate_switch(p, n, lo, hi, flags))));
  }
  return guarded(() => withBytes(name, (p, n) => readResult(wasm.generate(p, n, seed >>> 0, flags))));
}

/** The extra argument that tells the generator's commands to use the Switch RNG. */
function rngArgs(seed) {
  return typeof seed === "bigint" ? ["switch"] : [];
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

function prettyUrl(name) {
  return `img/pretty/${name.toLowerCase()}.webp`;
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

// ---- The landing and comedown views ----

// Each camera looks the way the pod faces: `dist` units back from its target, `angle` degrees
// down, with a vertical field of view of `fov` degrees, aimed `lift` units above the target,
// which is `lookAhead` units ahead of the pod.
//
// The landing view, what you see when you land: the game's "Far(Low)" settings in
// /user/Nishimura/Camera/caveCameraParms.txt. The target is behind the pod, since the captain
// lands just behind it. These were fitted to speedrun videos whose seeds were recovered
// (Emergence Cave 1 and Hole of Beasts 1 in three runs): the screen shows from about 330 units
// behind the pod to about 1,100 ahead.
//
const LANDING_CAMERA = { dist: 900, angle: 25, fov: 26, lift: 40, lookAhead: -55 };

// The comedown, while the sublevel's name is on screen as the pod comes down: frame 0 of the
// cutscene s0B_cv_coursein (/user/Mukki/movie/s0B_cv_coursein/demo.szs), which holds still until
// the name fades. The cutscene is placed at the pod with its +Z along the pod's facing (x to the
// pod's left, y up from the floor). The camera looks almost straight down from 1,321 units up
// with a 74° vertical field of view, turned by its roll (JStudio: RotZ(-roll) * LookAt).
const COMEDOWN_CAMERA = { eye: [-0.286, 1320.99, -3.17], target: [4.737, 6.729, -1.692], fovy: 74.11, roll: 87.82 };
// Outline colors: white for the pod, the captains' cursor colors, and yellow for the comedown.
const LANDING_COLOR = "#f2f2f2";
const OLIMAR_COLOR = "#ff5a4d";
const LOUIE_COLOR = "#4d9bff";
const COMEDOWN_COLOR = "#ffe14d";

// Which way the camera faces when you land. Each floor starts both captains' cameras at
// camera_angle in /user/Kando/aiConstants.txt (290: behind the captain at 290 degrees, looking
// toward 110), with the captains facing 0 (+Z), since caves have no map rotation
// (BaseGameSection::setupFloatMemory). The cave then makes the active captain the player, and
// CameraMgr::changePlayerMode centers the camera behind him only when that switches from Olimar:
// as Louie it looks toward his cursor, straight ahead of him, at 0. Pressing L does the same, but
// by then the captain faces the way the pod does. Checked against a speedrun that often doesn't
// press L.
const LANDING_HEADINGS = { pod: null, olimar: 110, louie: 0 };
const VIEWS = ["pod", "olimar", "louie", "comedown"];

/** The landing view's corners in game units, for `camera` "pod", "olimar" or "louie". */
function landingCorners(ship, camera) {
  const a = ((ship.angle ?? 0) * Math.PI) / 180;
  const heading = LANDING_HEADINGS[camera] ?? ship.angle ?? 0;
  const h = (heading * Math.PI) / 180;
  const look = LANDING_CAMERA.lookAhead;
  // The point the trapezoid is measured from: the captain, moved back along the view.
  const qx = ship.x + look * (Math.sin(a) - Math.sin(h));
  const qz = ship.z + look * (Math.cos(a) - Math.cos(h));
  return placeCorners(qx, qz, heading, shapeCorners(landingShape()));
}

/** Sets or removes a `name=1` flag in the page's URL, without reloading. */
function setUrlFlag(name, on) {
  const p = new URLSearchParams(location.search);
  if (on) p.set(name, "1");
  else p.delete(name);
  history.replaceState(null, "", `?${p.toString()}`);
}

/** The view drawn alone ("pod", "olimar", "louie" or "comedown"), or null. */
function chosenView() {
  return VIEWS.find((v) => $(`view-${v}`)?.checked) ?? null;
}

/** The screen's width over its height: 16:9 or 4:3 (the game's Screen setting). */
function screenAspect() {
  return $("screen-shape")?.value === "4:3" ? 4 / 3 : 16 / 9;
}

/**
 * The floor a camera sees: a trapezoid ahead of the pod. `near` and `far` are distances ahead
 * of the pod in game units (negative is behind it), with the half-widths of the view there.
 */
function viewShape(camera, aspect = screenAspect()) {
  const rad = Math.PI / 180;
  const height = camera.dist * Math.sin(camera.angle * rad) + camera.lift;
  const back = camera.dist * Math.cos(camera.angle * rad) - camera.lookAhead;
  const half = (camera.fov / 2) * rad;
  const tanSide = Math.tan(half) * aspect;
  // Where the screen's bottom and top edges meet the floor.
  const edge = (pitch) => {
    const ground = height / Math.tan(pitch);
    const depth = Math.hypot(ground, height) * Math.cos(half);
    return { ahead: ground - back, half: depth * tanSide };
  };
  const near = edge(camera.angle * rad + half);
  const far = edge(camera.angle * rad - half);
  return { near: near.ahead, far: far.ahead, nearHalf: near.half, farHalf: far.half };
}

/** What the camera shows when you land. */
function landingShape(aspect = screenAspect()) {
  return viewShape(LANDING_CAMERA, aspect);
}

/** A trapezoid's corners as [ahead, side] from the pod; side is to its right. */
function shapeCorners(s) {
  return [[s.near, -s.nearHalf], [s.near, s.nearHalf], [s.far, s.farHalf], [s.far, -s.farHalf]];
}

/**
 * What the camera shows while the sublevel's name is on screen: where the rays through the
 * screen's corners meet the floor, as [ahead, side] from the pod.
 */
function comedownCorners(aspect = screenAspect()) {
  const cam = COMEDOWN_CAMERA;
  const norm = (a) => a.map((v) => v / Math.hypot(...a));
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  // The view's axes (C_MTXLookAt): back, right and up.
  const back = norm(cam.eye.map((v, i) => v - cam.target[i]));
  const right = norm(cross([0, 1, 0], back));
  const up = cross(back, right);
  const t = Math.tan(((cam.fovy / 2) * Math.PI) / 180);
  const [sin, cos] = [Math.sin((cam.roll * Math.PI) / 180), Math.cos((cam.roll * Math.PI) / 180)];
  return [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([sx, sy]) => {
    // A screen corner's ray, turned back by the roll into the look-at frame.
    const [x0, y0] = [sx * t * aspect, sy * t];
    const [x, y] = [x0 * cos - y0 * sin, x0 * sin + y0 * cos];
    const d = [0, 1, 2].map((i) => right[i] * x + up[i] * y - back[i]);
    const k = -cam.eye[1] / d[1];
    return [cam.eye[2] + k * d[2], -(cam.eye[0] + k * d[0])];
  });
}

/** Corners given as [ahead, side] from a pod at (x, z) facing `degrees`, in game units. */
function placeCorners(x, z, degrees, corners) {
  const a = (degrees * Math.PI) / 180;
  const f = [Math.sin(a), Math.cos(a)];
  const r = [-f[1], f[0]];
  return corners.map(([ahead, side]) => [x + f[0] * ahead + r[0] * side, z + f[1] * ahead + r[1] * side]);
}

/** The part of the map to draw, in grid cells: the whole cave, or the landing view. */
function mapView(layout) {
  const ship = layout.objects.find((o) => o.kind === "ship");
  const chosen = chosenView();
  if (chosen && ship) {
    // One view alone: the floor in view, turned as the screen shows it. Corners go bottom-left,
    // bottom-right, top-right, top-left of the screen.
    const corners = chosen === "comedown" ? placeCorners(ship.x, ship.z, ship.angle ?? 0, comedownCorners()) : landingCorners(ship, chosen);
    const [tr, tl] = [corners[2], corners[3]];
    const turn = -Math.atan2(tr[1] - tl[1], tr[0] - tl[0]);
    const center = [0, 1].map((i) => corners.reduce((s, c) => s + c[i], 0) / 4);
    const turned = corners.map(([x, z]) => {
      const [dx, dz] = [x - center[0], z - center[1]];
      return [dx * Math.cos(turn) - dz * Math.sin(turn), dx * Math.sin(turn) + dz * Math.cos(turn)];
    });
    const margin = 40; // game units around the view
    const left = Math.min(...turned.map((c) => c[0])) - margin;
    const top = Math.min(...turned.map((c) => c[1])) - margin;
    return {
      single: true,
      corners,
      center,
      turn,
      left,
      top,
      w: (Math.max(...turned.map((c) => c[0])) + margin - left) / UNIT,
      h: (Math.max(...turned.map((c) => c[1])) + margin - top) / UNIT,
    };
  }
  // The cave with a half-cell margin on each side, grown to fit the landing and comedown outlines.
  let [x0, z0, x1, z1] = [-0.5, -0.5, layout.width + 0.5, layout.height + 0.5];
  for (const corners of viewOutlines(layout)) {
    for (const [x, z] of corners) {
      x0 = Math.min(x0, x / UNIT - 0.25);
      z0 = Math.min(z0, z / UNIT - 0.25);
      x1 = Math.max(x1, x / UNIT + 0.25);
      z1 = Math.max(z1, z / UNIT + 0.25);
    }
  }
  return { x0, z0, w: x1 - x0, h: z1 - z0, single: false };
}

/** The landing and comedown outlines that are turned on, in game units: [corners, color]. */
function viewOutlines(layout, withColors = false) {
  const ship = layout.objects.find((o) => o.kind === "ship");
  if (!ship || ship.angle === undefined) return [];
  const out = [];
  if ($("opt-comedownBox")?.checked) out.push([placeCorners(ship.x, ship.z, ship.angle, comedownCorners()), COMEDOWN_COLOR]);
  if ($("opt-landingBoxOlimar")?.checked) out.push([landingCorners(ship, "olimar"), OLIMAR_COLOR]);
  if ($("opt-landingBoxLouie")?.checked) out.push([landingCorners(ship, "louie"), LOUIE_COLOR]);
  if ($("opt-landingBox")?.checked) out.push([landingCorners(ship, "pod"), LANDING_COLOR]);
  return withColors ? out : out.map(([c]) => c);
}

/** The largest scale whose canvas the browser can still draw comfortably. */
function maxScale(view) {
  const dpr = window.devicePixelRatio || 1;
  return Math.min(6, 6000 / (Math.max(view.w, view.h) * CELL * dpr));
}

/**
 * A map pane of fixed size: the map is drawn at some scale into its own canvas (`draw(scale)`
 * returns it, with its size in CSS pixels as `cssW`/`cssH`), and the pane shows it at any zoom
 * and position over a dark background. The wheel, a trackpad pinch, two fingers, the buttons
 * and double-click zoom about the pointer; dragging moves the map, past the edges if you like.
 * While zooming the drawn map is stretched, then drawn again sharp once you stop.
 *
 * `size()` gives the map's size at scale 1 and its largest drawable scale (`max`) and fit
 * scale (`maxFit`). `onChange(viewer)` runs after each redraw.
 */
function createViewer(pane, canvas, { size, draw, onChange }) {
  const v = { scale: 1, ox: 0, oy: 0, fitted: true, content: null, cscale: 1, dragged: false };
  const paneSize = () => ({ w: pane.clientWidth, h: pane.clientHeight });

  v.fitScale = () => {
    const s = size();
    const p = paneSize();
    return Math.max(0.02, Math.min((p.w - 16) / s.w, (p.h - 16) / s.h, s.maxFit || 1.5));
  };

  v.present = () => {
    const dpr = window.devicePixelRatio || 1;
    const p = paneSize();
    if (canvas.width !== Math.round(p.w * dpr) || canvas.height !== Math.round(p.h * dpr)) {
      canvas.width = Math.round(p.w * dpr);
      canvas.height = Math.round(p.h * dpr);
      canvas.style.width = `${p.w}px`;
      canvas.style.height = `${p.h}px`;
    }
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = "#0f0f0f";
    ctx.fillRect(0, 0, p.w, p.h);
    if (v.content) {
      const r = v.scale / v.cscale;
      ctx.imageSmoothingEnabled = r < 1;
      ctx.drawImage(v.content, v.ox, v.oy, v.content.cssW * r, v.content.cssH * r);
    }
    if (onChange) onChange(v);
  };

  let drawing = null;
  /** Draws the map again at the current scale (and fits it first if it should be fitted). */
  v.refresh = async (refit = false) => {
    if (refit) v.fitted = true;
    if (v.fitted) {
      const s = size();
      const p = paneSize();
      v.scale = v.fitScale();
      v.ox = (p.w - s.w * v.scale) / 2;
      v.oy = (p.h - s.h * v.scale) / 2;
    }
    const target = Math.min(v.scale, size().max);
    const job = draw(target);
    drawing = job;
    const content = await job;
    if (drawing !== job) return; // a newer redraw replaced this one
    v.content = content;
    v.cscale = target;
    v.present();
  };

  let settle = null;
  v.zoomAt = (factor, clientX, clientY) => {
    const r = pane.getBoundingClientRect();
    const x = clientX - r.left;
    const y = clientY - r.top;
    const next = Math.max(0.02, Math.min(40, v.scale * factor));
    v.ox = x - ((x - v.ox) * next) / v.scale;
    v.oy = y - ((y - v.oy) * next) / v.scale;
    v.scale = next;
    v.fitted = false;
    v.present();
    clearTimeout(settle);
    settle = setTimeout(() => {
      if (Math.abs(Math.min(v.scale, size().max) - v.cscale) > 1e-3) v.refresh();
    }, 150);
  };
  v.zoomCenter = (factor) => {
    const r = pane.getBoundingClientRect();
    v.zoomAt(factor, r.left + r.width / 2, r.top + r.height / 2);
  };

  /** A point in the pane, in the drawn map's CSS pixels. */
  v.toContent = (clientX, clientY) => {
    const r = pane.getBoundingClientRect();
    const k = v.cscale / v.scale;
    return { x: (clientX - r.left - v.ox) * k, y: (clientY - r.top - v.oy) * k };
  };

  pane.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      const unit = e.deltaMode === 1 ? 0.05 : e.deltaMode === 2 ? 1 : e.ctrlKey ? 0.01 : 0.0015;
      v.zoomAt(Math.exp(-e.deltaY * unit), e.clientX, e.clientY);
    },
    { passive: false },
  );
  pane.addEventListener("dblclick", (e) => {
    e.preventDefault();
    v.zoomAt(e.shiftKey ? 0.5 : 2, e.clientX, e.clientY);
  });
  const pointers = new Map();
  let start = null;
  let pinch = null;
  pane.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || e.target.closest("button, a, input, select")) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    pane.setPointerCapture(e.pointerId);
    v.dragged = false;
    start = { x: e.clientX, y: e.clientY, ox: v.ox, oy: v.oy };
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      pinch = Math.hypot(a.x - b.x, a.y - b.y);
    }
  });
  pane.addEventListener("pointermove", (e) => {
    if (!pointers.has(e.pointerId)) return;
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.size === 2 && pinch) {
      const [a, b] = [...pointers.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      if (d > 0 && Math.abs(d / pinch - 1) > 0.02) {
        v.zoomAt(d / pinch, (a.x + b.x) / 2, (a.y + b.y) / 2);
        pinch = d;
      }
      v.dragged = true;
      return;
    }
    if (!start) return;
    const dx = e.clientX - start.x;
    const dy = e.clientY - start.y;
    if (Math.abs(dx) + Math.abs(dy) > 4) v.dragged = true;
    if (v.dragged) {
      v.ox = start.ox + dx;
      v.oy = start.oy + dy;
      v.fitted = false;
      v.present();
      pane.classList.add("dragging");
    }
  });
  const end = (e) => {
    pointers.delete(e.pointerId);
    if (pointers.size < 2) pinch = null;
    if (pointers.size === 1) {
      const [p] = [...pointers.values()];
      start = { x: p.x, y: p.y, ox: v.ox, oy: v.oy };
    }
    if (!pointers.size) start = null;
    pane.classList.remove("dragging");
  };
  pane.addEventListener("pointerup", end);
  pane.addEventListener("pointercancel", end);
  // Redraw when the pane changes size (the window, or the page's layout).
  let lastSize = "";
  const ro = new ResizeObserver(() => {
    const p = paneSize();
    const now = `${p.w}x${p.h}`;
    if (now === lastSize || !v.content) {
      lastSize = now;
      return;
    }
    lastSize = now;
    clearTimeout(v.resizeTimer);
    v.resizeTimer = setTimeout(() => (v.fitted ? v.refresh() : v.present()), 150);
  });
  ro.observe(pane);
  v.destroy = () => ro.disconnect();
  return v;
}

let mainViewer = null;
const mapContent = document.createElement("canvas"); // the generator's map, drawn at one scale

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
// `section` is the Advanced options box each goes in.
// Map options kept in the link (as key=1) as well as in the saved settings.
const URL_OPTIONS = ["comedownVisible", "holeOdds"];
const MAP_OPTIONS = [
  { key: "treasures", on: true, section: "objects" },
  { key: "hideBuried", section: "objects" },
  { key: "teki", on: true, section: "objects" },
  { key: "plants", on: true, section: "objects" },
  { key: "gates", on: true, section: "objects" },
  { key: "gateHealth", on: true, section: "objects" },
  { key: "exits", on: true, section: "objects" },
  { key: "comedownVisible", section: "view" },
  { key: "landingBox", section: "view" },
  { key: "landingBoxOlimar", section: "view" },
  { key: "landingBoxLouie", section: "view" },
  { key: "comedownBox", section: "view" },
  { key: "quickGlance", section: "view" },
  { key: "pretty", section: "view" },
  { key: "grid", section: "view" },
  { key: "gauge", section: "view" },
  { key: "spawnpoints", details: true, section: "generation" },
  { key: "spawnOrder", details: true, section: "generation" },
  { key: "pathDists", details: true, section: "generation" },
  { key: "podAngle", section: "generation" },
  { key: "angles", details: true, section: "generation" },
  { key: "doorIds", details: true, section: "generation" },
  { key: "doorLinks", details: true, section: "generation" },
  { key: "waypoints", details: true, section: "generation" },
  { key: "paths", details: true, section: "generation" },
  { key: "unitScores", details: true, section: "scores" },
  { key: "spawnScores", details: true, section: "scores" },
  { key: "holeOdds", details: true, section: "scores" },
  { key: "tekiScores", details: true, section: "scores" },
  { key: "doorScores", details: true, section: "scores" },
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
// The treasure gauge (og::Screen::OtakaraSensor::calcLevel): its needle rises to
// 0.05 + 0.95 (1 - d/900)^2 of the way for the nearest treasure, where d is the flat distance
// minus the treasure's bounding radius (Radar::Mgr::calcNearestTreasure). In tenths, as on
// CaveGen's gauge rings, that's 0.5 + 9.5 (1 - d/900)^2. Each band is where the needle sits
// between two of these marks: yellow far out, deep red-orange right next to a treasure.
const GAUGE_TICKS = [0.5, 1, 2, 3, 4, 5, 6, 7, 8, 9];
// Yellow below 0.3 (3 tenths), orange from 0.3, red from 0.8; each a little stronger inward.
const GAUGE_SHADES = [
  [255, 240, 150, 0.16],
  [255, 228, 110, 0.22],
  [255, 214, 70, 0.28],
  [255, 170, 60, 0.30],
  [252, 152, 45, 0.35],
  [248, 134, 35, 0.40],
  [242, 116, 30, 0.45],
  [236, 98, 25, 0.50],
  [220, 40, 40, 0.55],
  [190, 15, 40, 0.62],
];
/** The distance at which the gauge reaches `tick` tenths. */
const gaugeDistance = (tick) => 900 * (1 - Math.sqrt((tick - 0.5) / 9.5));
// Caveripper's spawn group colors, for its spawn point markers.
const SP_COLORS = { 0: "#fa57cf", 1: "#c90234", 2: "#e67300", 5: "#858585", 6: "#3b945a", 7: "#e63256", 8: "#59068a", 9: "#2dada7" };

/** An icon recolored to one color, keeping its shape (Caveripper's Colorize). */
const tintCache = new Map();
function tintedIcon(img, color) {
  const key = `${img.src}|${color}`;
  if (!tintCache.has(key)) {
    const c = document.createElement("canvas");
    c.width = img.width;
    c.height = img.height;
    const g = c.getContext("2d");
    g.drawImage(img, 0, 0);
    g.globalCompositeOperation = "source-in";
    g.fillStyle = color;
    g.fillRect(0, 0, c.width, c.height);
    tintCache.set(key, c);
  }
  return tintCache.get(key);
}

// What a teki is, by internal name (lowercase), not by the spawn group it was placed in: caveinfo
// sometimes puts plants in teki spots and teki in plant spots. The same lists as the cavegen3
// crate's teki_kind.rs (a test checks): the decomp's Plants managers, spelled as in caveinfo
// (DaiodoRed, Magaret), and the fire, electric and gas hazards. Candypop Buds, Pellet Posies
// and Skitter Leaves are teki.
const PLANT_TEKI = new Set([
  "chiyogami", "clover", "daiodogreen", "daiodored", "hikarikinoko", "kareooinu_l", "kareooinu_s", "magaret", "nekojarashi",
  "ooinu_l", "ooinu_s", "tanpopo", "tukushi", "wakame_l", "wakame_s", "watage", "zenmai",
]);
const HAZARD_TEKI = new Set(["hiba", "elechiba", "gashiba"]);

function isPlant(name) {
  return PLANT_TEKI.has(String(name).toLowerCase());
}

/**
 * Whether the game draws an object while the pod comes down. The cave intro cutscene
 * (s0B_cv_coursein) has draw flags 5 in /user/Mukki/movie/demos.txt: Pikmin and pellets, not
 * enemies, so only the teki that always count as cutscene actors show (plants and hazards). Eggs,
 * Candypop Buds and every enemy don't, nor a treasure a teki carries, which is only a pellet once
 * it dies. Treasures, gates, exits, Onions and the pod show. The CLI's render::comedown_visible
 * is the same.
 */
function comedownVisible(o) {
  if (o.kind !== "teki") return true;
  const name = String(o.name).toLowerCase();
  return PLANT_TEKI.has(name) || HAZARD_TEKI.has(name);
}

function objectShown(o, opts) {
  if (opts.comedownVisible && !comedownVisible(o)) return false;
  switch (o.kind) {
    case "treasure":
      return opts.treasures && !(opts.hideBuried && o.buried);
    case "teki":
      return isPlant(o.name) ? opts.plants : opts.teki;
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
  for (const s of ["pod_icon", "Cave_icon", "36px-Clog_icon", "Geyser_icon", "Gray_bramble_gate_icon", "duck", "cave_white", "leaf_icon", "ship", "star"]) urls.add(specialUrl(s));
  if ($("opt-pretty")?.checked) {
    for (const u of layout.units) urls.add(pickUrl([prettyUrl(u.name)]));
    urls.add(pickUrl([prettyUrl("toy_bg")]));
    urls.add(pickUrl([prettyUrl("kusachi_bg")]));
  }
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

/** Shows a layout in the map pane, fitting it to the pane if `refit` (a new sublevel or view). */
async function render(layout, refit = false) {
  if (available) await Promise.all(imagesFor(layout).map(loadImage));
  else await Promise.all(layout.units.map((u) => firstImage(unitUrls(u.name, u.game))));
  $("result").classList.toggle("single-view", mapView(layout).single);
  await mainViewer.refresh(refit);
}

/** Draws a layout into the map's own canvas at `scale` (1 = CELL pixels per grid cell). */
function drawMap(layout, scale) {
  const opts = mapOptions();
  const canvas = mapContent;
  const view = mapView(layout);
  const cell = CELL * scale;
  const dpr = window.devicePixelRatio || 1;
  const w = view.w * cell;
  const h = view.h * cell;
  canvas.width = Math.round(w * dpr);
  canvas.height = Math.round(h * dpr);
  canvas.cssW = w;
  canvas.cssH = h;
  hitTargets = [];

  const ctx = canvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = "#0f0f0f";
  ctx.fillRect(0, 0, w, h);

  const toPx = (v) => (v / UNIT) * cell;

  // Icons and numbers stay upright when the landing or comedown view turns the map.
  let upright = 0;
  if (view.single) {
    // Turn as the screen shows it, and show only what's in view.
    ctx.translate(-toPx(view.left), -toPx(view.top));
    ctx.rotate(view.turn);
    ctx.translate(-toPx(view.center[0]), -toPx(view.center[1]));
    upright = -view.turn;
    ctx.beginPath();
    for (const [x, z] of view.corners) ctx.lineTo(toPx(x), toPx(z));
    ctx.closePath();
    ctx.clip();
  } else {
    ctx.translate(-view.x0 * cell, -view.z0 * cell);
  }
  // Where a map point lands in this canvas, for tooltips.
  const toCanvas = (x, z) => {
    const p = ctx.getTransform().transformPoint(new DOMPoint(x, z));
    return { x: p.x / dpr, z: p.y / dpr };
  };

  // Pretty: CaveGen's floor backdrop for toy and grassy caves, under rendered map units.
  if (opts.pretty && layout.units.length) {
    const first = layout.units[0].name.toLowerCase();
    const bg = first.includes("toy") ? imageOf([prettyUrl("toy_bg")]) : first.includes("kusachi") ? imageOf([prettyUrl("kusachi_bg")]) : null;
    if (bg) {
      ctx.save();
      // Fixed to the floor: 40 of the backdrop's pixels per map cell, as in CaveGen.
      const pattern = ctx.createPattern(bg, "repeat");
      pattern.setTransform(new DOMMatrix().scale(cell / 40));
      ctx.fillStyle = pattern;
      ctx.fillRect(0, 0, layout.width * cell, layout.height * cell);
      ctx.restore();
    }
  }

  // Map units, rotated clockwise in quarter turns like the game's radar images.
  ctx.imageSmoothingEnabled = false;
  for (const u of layout.units) {
    const pretty = opts.pretty ? imageOf([prettyUrl(u.name)]) : null;
    const img = pretty || imageOf(unitUrls(u.name, u.game));
    const cx = (u.x + u.w / 2) * cell;
    const cz = (u.z + u.h / 2) * cell;
    ctx.save();
    ctx.translate(cx, cz);
    if (img) {
      ctx.rotate((u.rot * Math.PI) / 2);
      const [w0, h0] = u.rot % 2 ? [u.h, u.w] : [u.w, u.h];
      if (pretty) {
        // A picture, not a radar image: smooth it, and draw it as it is.
        ctx.imageSmoothingEnabled = true;
        // A pixel larger on every side, so neighbors overlap instead of leaving a hairline.
        ctx.drawImage(pretty, (-w0 * cell) / 2 - 1, (-h0 * cell) / 2 - 1, w0 * cell + 2, h0 * cell + 2);
        ctx.imageSmoothingEnabled = false;
      } else {
        ctx.filter = "brightness(1.6)";
        ctx.drawImage(unitImage(img), (-w0 * cell) / 2, (-h0 * cell) / 2, w0 * cell, h0 * cell);
        ctx.filter = "none";
      }
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
    ctx.save();
    ctx.translate(x, z);
    ctx.rotate(upright);
    ctx.font = `bold ${size}px system-ui, sans-serif`;
    ctx.textAlign = align;
    ctx.textBaseline = "middle";
    ctx.lineJoin = "round";
    ctx.lineWidth = Math.max(2, size / 4);
    ctx.strokeStyle = "#000";
    ctx.strokeText(text, 0, 0);
    ctx.fillStyle = color;
    ctx.fillText(text, 0, 0);
    ctx.restore();
  };
  const thin = Math.max(1, cell / 30);
  const special = (name) => loaded.get(specialUrl(name)) || null;

  const carried = layout.objects.filter((o) => o.kind === "treasure" || (o.kind === "teki" && o.carrying));
  if (opts.gauge && carried.length) {
    // Shade the floor by the needle's band. Drawn on its own layer, from the outermost band in:
    // each band clears what's inside it and paints its own shade, then the layer goes on the map.
    const layer = document.createElement("canvas");
    layer.width = canvas.width;
    layer.height = canvas.height;
    const g = layer.getContext("2d");
    g.setTransform(ctx.getTransform());
    GAUGE_TICKS.forEach((tick, i) => {
      g.beginPath();
      for (const o of carried) {
        // A teki's own size isn't known here, so treasures it carries use no extra radius.
        const r = gaugeDistance(tick) + (o.kind === "treasure" ? o.radius || 0 : 0);
        g.moveTo(toPx(o.x) + toPx(r), toPx(o.z));
        g.arc(toPx(o.x), toPx(o.z), toPx(r), 0, Math.PI * 2);
      }
      g.globalCompositeOperation = "destination-out";
      g.fillStyle = "#000";
      g.fill("nonzero");
      g.globalCompositeOperation = "source-over";
      const [r, gr, b, a] = GAUGE_SHADES[i];
      g.fillStyle = `rgba(${r}, ${gr}, ${b}, ${a})`;
      g.fill("nonzero");
    });
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.drawImage(layer, 0, 0);
    ctx.restore();
  }

  // QuickGlance (CaveGen's): translucent discs behind what matters, 1.25 cells across.
  if (opts.quickGlance) {
    const colossal = layout.sublevel.toLowerCase().startsWith("colossal");
    const disc = (o, color) => {
      ctx.fillStyle = color;
      ctx.beginPath();
      ctx.arc(toPx(o.x), toPx(o.z), cell * 0.625, 0, Math.PI * 2);
      ctx.fill();
    };
    for (const o of layout.objects) {
      const name = (o.name || "").toLowerCase();
      const c = quickGlanceColor(o, name, colossal);
      if (c) disc(o, c);
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
    // Caveripper's markers: rings for easy teki, a dot for hard teki, a duck for treasures, the
    // exit icon for the hole, a leaf for plants, the ship for the pod and a star for special teki.
    for (const sp of [...spawnpoints].sort((a, b) => a.group - b.group)) {
      const x = toPx(sp.x);
      const z = toPx(sp.z);
      ctx.globalAlpha = sp.used ? 1 : 0.55;
      const marker = (name, color, size) => {
        const img = special(name);
        if (!img) return false;
        ctx.save();
        ctx.translate(x, z);
        ctx.rotate(upright);
        const tinted = color ? tintedIcon(img, color) : img;
        const r = Math.min(size / img.width, size / img.height);
        ctx.drawImage(tinted, (-img.width * r) / 2, (-img.height * r) / 2, img.width * r, img.height * r);
        ctx.restore();
        return true;
      };
      let drawn = true;
      switch (sp.group) {
        case 0: {
          ctx.strokeStyle = SP_COLORS[0];
          ctx.lineWidth = Math.max(1, cell / 40);
          let r = Math.max(3, toPx(sp.r));
          for (let i = 0; i < 3; i++, r *= 0.85) {
            ctx.beginPath();
            ctx.arc(x, z, r, 0, Math.PI * 2);
            ctx.stroke();
          }
          break;
        }
        case 1:
          ctx.fillStyle = SP_COLORS[1];
          ctx.beginPath();
          ctx.arc(x, z, Math.max(2.5, cell * 0.11), 0, Math.PI * 2);
          ctx.fill();
          break;
        case 2:
          drawn = marker("duck", SP_COLORS[2], cell * 0.42);
          break;
        case 4:
          drawn = marker("cave_white", null, cell * 0.42);
          break;
        case 6:
          drawn = marker("leaf_icon", SP_COLORS[6], cell * 0.4);
          break;
        case 7:
          drawn = marker("ship", null, cell * 0.42);
          break;
        case 8:
          drawn = marker("star", SP_COLORS[8], cell * 0.5);
          break;
        default:
          drawn = false;
      }
      if (!drawn) {
        ctx.fillStyle = SP_COLORS[sp.group] || SPAWN_COLORS[sp.group] || "#fff";
        ctx.beginPath();
        ctx.arc(x, z, Math.max(2.5, cell * 0.09), 0, Math.PI * 2);
        ctx.fill();
      }
      if (opts.angles) facing(ctx, x, z, sp.angle, cell * 0.35, "#fff");
    }
    ctx.globalAlpha = 1;
  }

  // Exits and the ship first, then gates, treasures and teki on top.
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  // `rot` turns an icon with the map (gates); others stay upright.
  const drawIcon = (img, x, z, size, rot = null) => {
    if (!img) return false;
    ctx.save();
    ctx.translate(x, z);
    ctx.rotate(rot === null ? upright : rot);
    const r = Math.min(size / img.width, size / img.height);
    ctx.drawImage(img, (-img.width * r) / 2, (-img.height * r) / 2, img.width * r, img.height * r);
    ctx.restore();
    return true;
  };
  const drawFallback = (text, x, z, size, color) => {
    ctx.save();
    ctx.translate(x, z);
    ctx.rotate(upright);
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(0, 0, size / 2.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = "#fff";
    ctx.font = `bold ${Math.max(8, size / 3.2)}px system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(text.slice(0, 3), 0, 0);
    ctx.restore();
  };

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
        if (opts.gateHealth) label(String(Math.round(o.health)), x, z, "#fff", Math.max(8, cell * 0.26));
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
    const hit = toCanvas(x, z);
    if (hit.x >= 0 && hit.z >= 0 && hit.x <= w && hit.z <= h) hitTargets.push({ ...hit, r: Math.max(size / 2, 8), label: objectLabel(o) });
  }

  // The pod's facing alone (CaveGen's -drawPodAngle); "Facing directions" already draws it.
  if (opts.podAngle && !opts.angles) {
    const ship = layout.objects.find((o) => o.kind === "ship");
    if (ship && ship.angle !== undefined) facing(ctx, toPx(ship.x), toPx(ship.z), ship.angle, cell * 0.6, "#ffe14d");
  }

  // Numbers on top of everything.
  if (opts.spawnOrder) {
    // Map units in placement order, outlined, numbered in their corner; then each object.
    ctx.strokeStyle = "rgba(0, 225, 225, 0.6)";
    ctx.lineWidth = thin;
    layout.units.forEach((u, i) => {
      ctx.strokeRect(u.x * cell, u.z * cell, u.w * cell, u.h * cell);
      label(String(i), u.x * cell + cell * 0.08, (u.z + u.h) * cell - cell * 0.2, "#00ffff", undefined, "left");
    });
    for (const o of layout.objects) {
      if (o.order !== undefined && objectShown(o, opts)) label(String(o.order), toPx(o.x) - cell * 0.38, toPx(o.z) + cell * 0.38, "#00ffff");
    }
  }
  if (opts.pathDists) {
    for (const o of carried) {
      if (o.path_dist !== undefined && objectShown(o, opts)) label(String(o.path_dist), toPx(o.x) + cell * 0.1, toPx(o.z) - cell * 0.12, "#c79bff", undefined, "left");
    }
  }
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
  // The chance of the exit at each spot (crates/cavegen3/src/hole_odds.rs).
  if (opts.holeOdds) {
    for (const sp of spawnpoints) if (sp.odds > 0) label(oddsPercent(sp.odds), toPx(sp.x), toPx(sp.z) + cell * 0.32, "#ff9de2");
  }
  // The comedown and landing outlines.
  if (!view.single) {
    for (const [corners, color] of viewOutlines(layout, true)) {
      ctx.strokeStyle = color;
      ctx.lineWidth = Math.max(1.5, cell / 16);
      ctx.setLineDash([cell / 4, cell / 6]);
      ctx.beginPath();
      for (const [x, z] of corners) ctx.lineTo(toPx(x), toPx(z));
      ctx.closePath();
      ctx.stroke();
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
  return canvas;
}

// QuickGlance colors, CaveGen's at alpha 100/255.
const QG = {
  ship: "rgba(255, 30, 30, 0.39)",
  treasure: "rgba(220, 120, 0, 0.39)",
  groink: "rgba(255, 130, 220, 0.39)",
  colossalGreen: "rgba(60, 255, 120, 0.39)",
  violet: "rgba(160, 0, 255, 0.39)",
  ivory: "rgba(60, 60, 60, 0.39)",
  exit: "rgba(0, 140, 60, 0.39)",
  onions: ["rgba(0, 0, 255, 0.39)", "rgba(255, 0, 0, 0.39)", "rgba(255, 255, 0, 0.39)", "rgba(160, 0, 255, 0.39)", "rgba(60, 60, 60, 0.39)"],
};

/** The QuickGlance disc color for an object, or null (CaveGen's Drawer, drawQuickGlance). */
function quickGlanceColor(o, name, colossal) {
  switch (o.kind) {
    case "ship":
      return QG.ship;
    case "hole":
    case "geyser":
      return QG.exit;
    case "onion":
      return QG.onions[o.color] || null;
    case "treasure":
      // Colossal Caverns marks only its buried treasures, in green.
      if (!colossal) return QG.treasure;
      return o.buried ? QG.colossalGreen : null;
    case "teki":
      if ((o.carrying && (name !== "blackman" || colossal)) || name === "bigtreasure") return QG.treasure;
      if (name === "minihoudai" || (colossal && name === "kumachappy")) return QG.groink;
      if (colossal && name === "leafchappy") return QG.colossalGreen;
      if (name === "blackpom") return QG.violet;
      if (name === "whitepom") return QG.ivory;
      return null;
  }
  return null;
}

/** A chance as a whole percentage, without rounding a small one to 0 or a near-certain one to 100. */
function oddsPercent(p) {
  if (p < 0.005) return "<1%";
  if (p > 0.995 && p < 1) return ">99%";
  return `${Math.round(p * 100)}%`;
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
    const at = mainViewer.toContent(clientX, clientY);
    const k = mainViewer.cscale / mainViewer.scale;
    const x = at.x;
    const z = at.y;
    let best = null;
    let bestDist = Infinity;
    for (const t of hitTargets) {
      const d = Math.hypot(t.x - x, t.z - z);
      if (d <= Math.max(t.r, 8 * k) && d < bestDist) {
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
    tip.style.left = `${Math.max(4, left)}px`;
    tip.style.top = `${Math.min(clientY - wrap.top + 14, wrap.height - tip.offsetHeight - 4)}px`;
  };
  canvas.addEventListener("mousemove", (e) => show(e.clientX, e.clientY));
  canvas.addEventListener("mouseleave", () => ($("tooltip").hidden = true));
  canvas.addEventListener("click", (e) => !mainViewer.dragged && show(e.clientX, e.clientY));
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

  // Caveinfo order, which is the order the generator places treasures in.
  const order = (layout.treasure_order || []).map((n) => n.toLowerCase());
  const rank = (t) => {
    const i = order.indexOf(t.name.toLowerCase());
    return i < 0 ? order.length : i;
  };
  treasures.sort((a, b) => rank(a) - rank(b));

  const exit = layout.objects.find((o) => o.kind === "hole" || o.kind === "geyser");
  parts.push(el("h2", {}, T.layout));
  const info = el("ul");
  info.append(el("li", {}, T.unitsLine(layout.units.length, layout.width, layout.height)));
  info.append(el("li", {}, exit ? (exit.kind === "hole" ? T.exitHole(exit.plugged) : T.exitGeyser) : T.noExit));
  if (current && current.judge) info.append(el("li", { class: "judge-line" }, T.judgeLine(current.judge.type, current.judge.score)));
  // judge2: the treasures a runner likely leaves behind, with the chance
  const skips = current && current.judge ? current.judge.skips.filter(([, p]) => p >= 0.1) : [];
  if (skips.length) {
    const list = skips.map(([name, p]) => `${displayName("treasures", name)} (${Math.round(p * 100)}%)`).join(", ");
    info.append(el("li", { class: "judge-line" }, T.likelySkipped(list)));
  }
  if (current) info.append(el("li", { class: "muted gen-time" }, T.generatedIn(current.ms.toFixed(1))));
  parts.push(info);

  parts.push(el("h2", {}, T.treasures(treasures.length)));
  const tl = el("ul", { class: "scroll-list" });
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
  const kl = el("ul", { class: "scroll-list" });
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
  if (cave.name === "Colossal Caverns") return `${cave.game}:colossal${colossalSuffix()}`;
  return `${cave.game}:${cave.short}-${floor}`;
}

// Colossal Caverns' options (CaveGen's -ultraRandomizer, -extraUnits, -5onion), as the
// "+ultra+extra+5onion" suffix the generator takes. The boxes only show for Colossal Caverns.
const COLOSSAL_OPTIONS = [
  ["col-ultra", "ultra"],
  ["col-extra", "extra"],
  ["col-5onion", "5onion"],
];

function colossalSuffix() {
  return COLOSSAL_OPTIONS.map(([id, word]) => ($(id)?.checked ? `+${word}` : "")).join("");
}

/** Shows the Colossal Caverns options only while that cave is picked. */
function showColossalOptions() {
  const box = $("colossal-options");
  if (!box) return;
  const cave = caves[Number($("cave").value)];
  box.hidden = !(cave && cave.name === "Colossal Caverns");
}

// Seeds are numbers for the GameCube/Wii generator and BigInts for the Switch one (64 bits).
const MASK64 = (1n << 64n) - 1n;
const A64 = 6364136223846793005n;
const A64_INV = (() => {
  // Newton's iteration for the inverse mod 2^64 (each step doubles the correct bits).
  let x = A64;
  for (let i = 0; i < 6; i++) x = (x * (2n - ((A64 * x) & MASK64))) & MASK64;
  return x;
})();

/** True when the Advanced options ask for the Switch generator (never on other pages). */
function isSwitch() {
  const sel = $("platform");
  return !!sel && sel.value === "switch";
}

function parseSeed(text) {
  const t = text.trim().replace(/^0x/i, "");
  if (isSwitch()) return /^[0-9a-f]{1,16}$/i.test(t) ? BigInt(`0x${t}`) : null;
  if (!/^[0-9a-f]{1,8}$/i.test(t)) return null;
  return parseInt(t, 16) >>> 0;
}

function formatSeed(seed) {
  if (typeof seed === "bigint") return (seed & MASK64).toString(16).toUpperCase().padStart(16, "0");
  return (seed >>> 0).toString(16).toUpperCase().padStart(8, "0");
}

/** The Switch RNG's state `n` steps after (or before, if negative) `seed`, in O(log n). */
function rngJump64(seed, n) {
  let k = BigInt(n);
  let [a, c] = k < 0n ? [A64_INV, (MASK64 + 1n - A64_INV) & MASK64] : [A64, 1n];
  if (k < 0n) k = -k;
  let mul = 1n;
  let add = 0n;
  while (k > 0n) {
    if (k & 1n) {
      mul = (mul * a) & MASK64;
      add = (add * a + c) & MASK64;
    }
    c = (c * a + c) & MASK64;
    a = (a * a) & MASK64;
    k >>= 1n;
  }
  return (seed * mul + add) & MASK64;
}

/** The game RNG's state `n` steps after (or before, if negative) `seed`. */
function rngStep(seed, n) {
  if (typeof seed === "bigint") return rngJump64(seed, n);
  let s = seed >>> 0;
  for (let i = 0; i < Math.abs(n); i++) {
    s = n > 0 ? (Math.imul(s, 0x41c64e6d) + 0x3039) >>> 0 : Math.imul((s - 0x3039) >>> 0, 0xeeb9eb65) >>> 0;
  }
  return s;
}

/**
 * Where the next slice of a search or aggregation starts, `n` seeds after `start`. Counting and
 * random seeds are numbered, so that's start + n; game-RNG seeds continue the RNG from where the
 * slice ended (each slice steps the RNG from its own start).
 */
function nextChunkStart(mode, start, n) {
  if (typeof start === "bigint") return mode === "g" ? rngJump64(start, n) : (start + BigInt(n)) & MASK64;
  return mode === "g" ? rngStep(start, n) : start + n;
}

function randomSeed() {
  if (isSwitch()) {
    const [lo, hi] = crypto.getRandomValues(new Uint32Array(2));
    return (BigInt(hi) << 32n) | BigInt(lo);
  }
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** `seed` turned into a seed for the other generator: Switch keeps the number, GC keeps the low 32 bits. */
function convertSeed(seed) {
  if (isSwitch()) return typeof seed === "bigint" ? seed : BigInt(seed >>> 0);
  return typeof seed === "bigint" ? Number(seed & 0xffffffffn) : seed;
}

/** The seed box's length, pattern and tooltip for the chosen generator. */
function setSeedInput() {
  const box = $("seed");
  if (!box) return;
  const sw = isSwitch();
  box.maxLength = sw ? 18 : 10;
  box.pattern = sw ? "(0x)?[0-9A-Fa-f]{1,16}" : "(0x)?[0-9A-Fa-f]{1,8}";
  box.title = sw ? T.seedTitleSwitch : T.seedTitle;
  box.classList.toggle("seed64", sw);
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
  showColossalOptions();
}

/** The cave and floor the controls show. */
function selected() {
  const cave = caves[Number($("cave").value)];
  return cave && { cave, floor: Number($("floor").value) || 1 };
}

/** Generates from the controls. The floor list is refilled when the cave changes. */
function run(options) {
  return withSearchPaused(() => generateAndShow(options));
}

async function generateAndShow({ newCave = false } = {}) {
  const cave = caves[Number($("cave").value)];
  if (!cave) return;
  const seed = parseSeed($("seed").value);
  if (seed === null) {
    setStatus(isSwitch() ? T.badSeedSwitch : T.badSeed, true);
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
    const judged = judgeType ? await command("judge", sublevel, judgeType, seed, ...rngArgs(seed)) : null;
    const judge = judged ? { type: judgeType, score: judged.score, skips: judged.skips || [] } : null;
    const refit = !current || current.sublevel !== sublevel;
    current = { layout, cave, floor, ms, sublevel, judge };
    $("result").hidden = false;
    $("tooltip").hidden = true;
    await showCurrent(refit);
    updateUrl(cave, floor, seed);
    if ($("caveinfo-box").open) await showCaveInfo();
  } catch (e) {
    console.error(e);
    setStatus(T.cantGenerate(e.message), true);
  }
}

/** Draws the current layout and its text in the current language. */
async function showCurrent(refit = false) {
  if (!current) return;
  const { layout, cave, floor, ms } = current;
  const title = layout.floors > 1 || cave.name !== "Colossal Caverns" ? T.floorName(caveName(cave), floor) : caveName(cave);
  $("title").textContent = `${title} · ${layout.seed}`;
  await render(layout, refit);
  await renderSummary(layout);
  setStatus("");
}

function updateUrl(cave, floor, seed) {
  const p = new URLSearchParams(location.search);
  const suffix = cave.name === "Colossal Caverns" ? colossalSuffix() : "";
  p.set("sublevel", `${cave.game}:${cave.short}-${floor}${suffix}`);
  p.set("seed", formatSeed(seed));
  if (typeof seed === "bigint") p.set("rng", "switch");
  else p.delete("rng");
  history.replaceState(null, "", `?${p.toString()}`);
}

/** Selects the cave and floor named by a "game:short-floor" (or any "SCx6"-style) string. */
function selectFromParam(text) {
  // Colossal Caverns' options come as "+ultra+extra+5onion" after the name.
  const [name, ...words] = text.split("+");
  for (const [id, word] of COLOSSAL_OPTIONS) if ($(id)) $(id).checked = words.map((w) => w.toLowerCase()).includes(word);
  text = name;
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

const JUDGES = ["score", "pod", "judge2", "at", "attk", "cmat", "key", "colossal", "mapunitcount"];
const SETTINGS_KEY = "cavegen3-options";

function buildAdvanced() {
  for (const { key, on, section } of MAP_OPTIONS) {
    const box = $(`map-options-${section}`);
    const input = el("input", { type: "checkbox", id: `opt-${key}` });
    input.checked = !!on;
    box.append(el("label", { class: "check", "data-i18n-tip-opt": key }, input, el("span", { "data-i18n-opt": key })));
  }
  fillJudges();
  // Saved choices (map options and judge) from last time.
  try {
    const saved = JSON.parse(localStorage.getItem(SETTINGS_KEY) || "{}");
    const map = { ...saved.map };
    // Before the landing outline had its own name, "comedownBox" was the landing outline.
    if (map.comedownBox !== undefined && map.landingBox === undefined) {
      map.landingBox = map.comedownBox;
      delete map.comedownBox;
    }
    for (const { key } of MAP_OPTIONS) if (key in map) $(`opt-${key}`).checked = !!map[key];
    if (JUDGES.includes(saved.judge)) $("judge").value = saved.judge;
    // "comedown" and "comedownScreen" are the old names.
    // "landing" and "comedown" are old saves of the pod view.
    const view = VIEWS.includes(saved.view) ? saved.view : saved.landing || saved.comedown ? "pod" : null;
    if (view) $(`view-${view}`).checked = true;
    if (saved.screen || saved.comedownScreen) $("screen-shape").value = saved.screen || saved.comedownScreen;
    if (saved.seedStep) $("seed-step").value = saved.seedStep;
    if (saved.platform === "switch" || saved.platform === "gc") $("platform").value = saved.platform;
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
        view: chosenView(),
        screen: $("screen-shape").value,
        seedStep: $("seed-step").value,
        platform: $("platform").value,
      }));
  } catch {
    // Not saved; the choices still apply to this page.
  }
}

function fillJudges() {
  const sel = $("judge");
  const value = sel.value;
  sel.replaceChildren(el("option", { value: "" }, T.judgeNone));
  for (const j of JUDGES) sel.append(el("option", { value: j, title: T.judgeTips[j] }, T.judgeNames[j]));
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

// Showing a layout (a search result, Generate, the ‹ › buttons) pauses a running search, so the
// map appears right away instead of sharing time with it; the search resumes when it's drawn.
let searchHold = 0;
let searchWaiters = [];
async function withSearchPaused(fn) {
  searchHold++;
  try {
    return await fn();
  } finally {
    searchHold--;
    if (!searchHold) for (const resume of searchWaiters.splice(0)) resume();
  }
}

function stopSearch(clear = false) {
  if (search) search.stopped = true;
  if (clear) {
    search = null;
    $("search-results").replaceChildren();
    $("count-results").replaceChildren();
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
  const countWhat = $("count-what").value.trim();
  const status = $("search-status");
  status.classList.remove("error");
  if (!judge && !query && !countWhat) {
    status.textContent = T.needJudgeOrQuery;
    status.classList.add("error");
    return;
  }
  const total = Math.max(1, Math.min(1e8, Math.floor(Number($("search-count").value) || 0)));
  const keep = Math.max(1, Math.min(500, Math.floor(Number($("search-keep").value) || 20)));
  const mode = $("search-mode").value;
  const seed = parseSeed($("seed").value) ?? 0;
  const startAt = mode === "r" ? randomSeed() : seed;
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
    countWhat,
    counts: new Map(),
    stopped: false,
    done: false,
    t0: performance.now(),
  };
  search = s;
  $("search-go").textContent = T.stop;
  try {
    await loadGame(sel.cave.game);
    let chunk = 50;
    let chunkStart = startAt;
    while (s.tested < total && !s.stopped) {
      if (searchHold) {
        await new Promise((resume) => searchWaiters.push(resume));
        if (s.stopped) break;
      }
      const n = Math.min(chunk, total - s.tested);
      const t = performance.now();
      const r = await command("search", s.sublevel, judge || "-", query || "-", chunkStart, n, mode, keep, countWhat || "-", ...rngArgs(chunkStart));
      // Switch seeds come back as decimal strings (64 bits don't fit a JavaScript number).
      r.best = r.best.map(([seed, score]) => [typeof seed === "string" ? BigInt(seed) : seed, score]);
      s.tested += n;
      chunkStart = nextChunkStart(mode, chunkStart, n);
      s.matched += r.matched;
      for (const [k, c] of r.counts) s.counts.set(k, (s.counts.get(k) || 0) + c);
      if (judge) {
        s.best = s.best.concat(r.best).sort((a, b) => a[1] - b[1]).slice(0, keep);
        for (const v of r.scores) s.scores.push(v);
      } else {
        s.best = s.best.concat(r.best).slice(0, keep);
        // A query alone stops once it has enough seeds; counting needs every seed.
        if (s.best.length >= keep && !countWhat) break;
      }
      // Aim for about 60 ms of work per slice.
      chunk = Math.max(10, Math.min(200000, Math.round((n * 60) / Math.max(1, performance.now() - t))));
      status.textContent = T.searching(s.tested, total);
      showSearchResults(s, false);
      await yieldToBrowser();
    }
    const secs = ((performance.now() - s.t0) / 1000).toFixed(1);
    status.textContent = s.stopped ? T.searchStopped(s.tested) : T.searchDone(s.tested, query ? s.matched : null, secs);
    if (query && !s.stopped) status.textContent += " " + T.matchedShare(s.matched, s.tested);
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

/** The count distribution: for each number of the thing, the share of seeds with that many. */
function showCounts(s) {
  const box = $("count-results");
  if (!s.countWhat || !s.counts.size) {
    box.replaceChildren();
    return;
  }
  const rows = [...s.counts.entries()].sort((a, b) => a[0] - b[0]);
  const tested = rows.reduce((t, [, n]) => t + n, 0);
  const most = Math.max(...rows.map(([, n]) => n));
  box.replaceChildren(
    el("div", { class: "count-title" }, T.countTitle(s.countWhat, tested)),
    ...rows.map(([k, n]) =>
      el(
        "div",
        { class: "count-row" },
        el("span", { class: "count-k" }, String(k)),
        el("span", { class: "count-bar" }, el("span", { style: `width: ${(100 * n) / most}%` })),
        el("span", { class: "count-n" }, n.toLocaleString()),
        el("span", { class: "count-pct" }, `${((100 * n) / tested).toFixed(2)}%`),
      ),
    ),
  );
}

function showSearchResults(s, final) {
  showCounts(s);
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

// Spawn groups as Caveripper's caveinfo shows them: its order, names and colors.
const CI_GROUPS = [
  { group: 8, key: "special", color: "#59068a", icon: "star" },
  { group: 1, key: "hard", color: "#c90234", score: 10 },
  { group: 0, key: "easy", color: "#fa57cf", score: 2 },
  { group: 6, key: "plant", color: "#3b945a", icon: "leaf_icon" },
  { group: 5, key: "seam", color: "#858585", score: 5 },
];
const CI_CAP_COLOR = "#2dada7";
const CI_TREASURE_COLOR = "#e67300";
const CI_GATE_COLOR = "#7a7a7a";
const CI_CELL = 22; // pixels per grid cell in the map unit thumbnails

/** A map unit's radar image on a small canvas, drawn like the map (alpha shading, brightened). */
async function unitThumb(u) {
  const c = el("canvas", { class: "ci-unit-img", width: u.w * CI_CELL, height: u.h * CI_CELL, "aria-hidden": "true" });
  const img = await firstImage(unitUrls(u.name, u.game));
  const ctx = c.getContext("2d");
  if (img) {
    ctx.imageSmoothingEnabled = false;
    ctx.filter = "brightness(1.6)";
    ctx.drawImage(unitImage(img), 0, 0, c.width, c.height);
  } else {
    ctx.fillStyle = "#333";
    ctx.fillRect(0, 0, c.width, c.height);
  }
  return c;
}

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
  const fl = ci.floor;
  const pct = (p) => `${Math.round(p * 1000) / 10}%`;
  const special = (name) => el("img", { src: specialUrl(name), alt: "" });

  // Title row: the sublevel, then the pod, the exit and the gates, as in Caveripper's.
  const meta = el("div", { class: "ci-meta" });
  meta.append(el("span", { class: "ci-meta-icon", title: T.ship }, special("pod_icon")));
  if (!fl.final) {
    const hole = el("span", { class: "ci-meta-icon", title: fl.plugged ? `${T.hole} · ${T.plugged}` : T.hole }, special("Cave_icon"));
    if (fl.plugged) hole.append(el("img", { src: specialUrl("36px-Clog_icon"), alt: "", class: "ci-over" }));
    meta.append(hole);
  }
  if (fl.geyser || fl.final) meta.append(el("span", { class: "ci-meta-icon", title: T.geyser }, special("Geyser_icon")));
  if (ci.gates.length) {
    const g = ci.gates[0];
    meta.append(
      el("span", { class: "ci-meta-icon ci-gate", title: `${T.gate} · ${T.hp(Math.round(g.health))}` }, special("Gray_bramble_gate_icon"),
        el("small", {}, T.hp(Math.round(g.health))), el("b", {}, `×${fl.max_gates}`)),
    );
  }
  const head = el("div", { class: "ci-head" }, el("h3", {}, T.floorName(caveName(sel.cave), sel.floor)), meta);

  const stats = [
    [T.maxTeki, fl.max_teki],
    [T.maxTreasures, fl.max_treasures],
    [T.maxGates, fl.max_gates],
  ];
  if (fl.waterwraith > 0) stats.push([T.waterwraith, `${Math.round(fl.waterwraith)} s`]);
  const statLine = el("div", { class: "ci-stats" }, ...stats.map(([k, v]) => el("span", {}, `${k} `, el("b", {}, String(v)))));

  // Boxes of spawn objects, colored by spawn group.
  const entityBox = (title, color, tiles, score = 0, icon = null) => {
    const header = el("div", { class: "ci-box-head" });
    if (icon) header.append(el("img", { src: specialUrl(icon), alt: "", class: "ci-box-icon" }));
    else header.append(el("span", { class: "ci-dot" }));
    header.append(el("span", {}, title));
    if (score) header.append(el("span", { class: "ci-score" }, T.ciScore(score)));
    return el("section", { class: "ci-box", style: `--c: ${color}` }, header, el("div", { class: "ci-tiles" }, ...tiles));
  };
  const tekiTile = async (t) => {
    const name = displayName("teki", t.name);
    const tip = [name];
    if (t.carrying) tip.push(T.carrying(displayName("treasures", t.carrying)));
    if (t.falling) tip.push(T.falling);
    const pic = el("span", { class: "ci-pic" }, await iconEl(tekiIconUrls(t.name, t.game)));
    if (t.carrying) pic.append(el("span", { class: "ci-carry" }, await iconEl(treasureIconUrls(t.carrying, t.game))));
    if (t.falling) pic.append(el("img", { src: specialUrl("falling_icon"), alt: "", class: "ci-fall" }));
    const nums = [];
    if (t.min > 0) nums.push(`x${t.min}`);
    if (t.weight > 0) nums.push(`w${t.weight}`);
    return el("figure", { class: "ci-tile", title: tip.join(" · ") }, pic, el("figcaption", { class: "ci-nums" }, nums.join(" ")), el("figcaption", { class: "ci-name" }, name));
  };
  const treasureTile = async (t) => {
    const name = displayName("treasures", t.name);
    const extra = [];
    if (t.value) extra.push(el("span", { class: "ci-poko" }, el("img", { src: specialUrl("Poko_icon"), alt: "" }), String(t.value)));
    if (t.carry && t.carry[1]) extra.push(el("span", { class: "ci-carry-n", title: T.ciCarryTip }, `${t.carry[0]}/${t.carry[1]}`));
    const nums = [];
    if (t.min > 0) nums.push(`x${t.min}`);
    if (t.weight > 0) nums.push(`w${t.weight}`);
    return el(
      "figure",
      { class: "ci-tile", title: name },
      el("span", { class: "ci-pic" }, await iconEl(treasureIconUrls(t.name, t.game))),
      el("figcaption", { class: "ci-nums" }, nums.join(" ")),
      el("figcaption", { class: "ci-name" }, name),
      el("figcaption", { class: "ci-extra" }, ...extra),
    );
  };

  const boxes = [];
  const known = new Set(CI_GROUPS.map((g) => g.group));
  for (const g of CI_GROUPS) {
    const teki = ci.teki.filter((t) => t.group === g.group);
    if (teki.length) boxes.push(entityBox(T.ciGroups[g.key], g.color, await Promise.all(teki.map(tekiTile)), g.score, g.icon));
  }
  const other = ci.teki.filter((t) => !known.has(t.group));
  if (other.length) boxes.push(entityBox(T.ciGroups.other, "#4a6fa5", await Promise.all(other.map(tekiTile))));
  if (ci.caps.length) boxes.push(entityBox(T.ciGroups.cap, CI_CAP_COLOR, await Promise.all(ci.caps.map(tekiTile))));
  if (ci.items.length) boxes.push(entityBox(T.ciGroups.treasure, CI_TREASURE_COLOR, await Promise.all(ci.items.map(treasureTile)), 0, "Poko_icon"));
  if (ci.gates.length > 1) {
    const tiles = ci.gates.map((g) =>
      el("figure", { class: "ci-tile" }, el("span", { class: "ci-pic" }, special("Gray_bramble_gate_icon")),
        el("figcaption", { class: "ci-nums" }, `w${g.weight}`), el("figcaption", { class: "ci-name" }, T.hp(Math.round(g.health)))),
    );
    boxes.push(entityBox(T.gateInfo, CI_GATE_COLOR, tiles));
  }

  // Map units: alcoves and one-cell-deep hallways first, then rooms and longer hallways.
  const small = ci.units.filter((u) => u.type !== "room" && u.h === 1);
  const large = ci.units.filter((u) => !(u.type !== "room" && u.h === 1));
  const unitTile = async (u) =>
    el(
      "figure",
      { class: `ci-unit ci-unit-${u.type}`, title: `${u.name} · ${u.w}×${u.h} · ${T.unitTypes[u.type] || u.type} · ${T.colDoors} ${u.doors}` },
      await unitThumb(u),
      el("figcaption", {}, u.name),
    );
  const unitLine = el(
    "div",
    { class: "ci-unit-stats" },
    el("span", {}, `${T.rooms} `, el("b", {}, String(fl.rooms))),
    el("span", {}, `${T.corridorProb} `, el("b", {}, pct(fl.corridor_probability))),
    el("span", {}, `${T.capProb} `, el("b", {}, pct(fl.cap_probability))),
  );
  const units = el(
    "div",
    { class: "ci-units" },
    unitLine,
    el("div", { class: "ci-unit-grid" }, ...(await Promise.all([...small, ...large].map(unitTile)))),
  );

  box.replaceChildren(el("div", { class: "ci" }, head, statLine, el("div", { class: "ci-boxes" }, ...boxes), units));
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
  for (const node of document.querySelectorAll("[data-i18n-tip]")) node.title = T.tips[node.dataset.i18nTip];
  for (const node of document.querySelectorAll("[data-i18n-tip-opt]")) node.title = T.tips.opt[node.dataset.i18nTipOpt];
  if ($("seed")) setSeedInput();
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
  // A link's generator wins over the saved one: ?rng=switch, or none for GameCube/Wii.
  if (params.has("seed")) $("platform").value = params.get("rng") === "switch" ? "switch" : "gc";
  setSeedInput();
  const seed = params.get("seed") && parseSeed(params.get("seed"));
  // "comedown=1" is the old name.
  // "landing=1" and "comedown=1" are old links to the pod view.
  const viewParam = VIEWS.includes(params.get("view")) ? params.get("view") : params.get("landing") === "1" || params.get("comedown") === "1" ? "pod" : null;
  if (viewParam) for (const v of VIEWS) $(`view-${v}`).checked = v === viewParam;
  // The comedown-visible filter and hole odds are kept in the link, so a shared link shows the same.
  for (const key of URL_OPTIONS) {
    if (params.has(key)) $(`opt-${key}`).checked = params.get(key) === "1";
    setUrlFlag(key, $(`opt-${key}`).checked);
  }
  // "judge=judge2" picks the judge for this link (over the saved one); "judge=none" turns it off.
  const judgeParam = params.get("judge");
  if (judgeParam === "none" || JUDGES.includes(judgeParam)) $("judge").value = judgeParam === "none" ? "" : judgeParam;
  $("seed").value = formatSeed(seed ?? randomSeed());

  $("game").addEventListener("change", () => {
    fillCaves($("game").value);
    $("cave").selectedIndex = 0;
    run({ newCave: true });
  });
  $("cave").addEventListener("change", () => run({ newCave: true }));
  $("floor").addEventListener("change", () => run());
  for (const [id] of COLOSSAL_OPTIONS) $(id).addEventListener("change", () => run());
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
    const next = typeof s === "bigint" ? (s + BigInt(d)) & MASK64 : s + d;
    $("seed").value = formatSeed($("seed-step").value === "g" ? rngStep(s, d) : next);
    run();
  };
  $("seed-step").addEventListener("change", saveSettings);
  $("platform").addEventListener("change", () => {
    // Keep the seed where it makes sense (Switch takes it as is, GameCube/Wii its low 8 digits).
    const text = $("seed").value.trim().replace(/^0x/i, "");
    const old = /^[0-9a-f]{1,16}$/i.test(text) ? BigInt(`0x${text}`) : null;
    setSeedInput();
    $("seed").value = formatSeed(old === null ? randomSeed() : convertSeed(old));
    stopSearch(true);
    saveSettings();
    run();
  });
  $("prev").addEventListener("click", () => step(-1));
  $("next").addEventListener("click", () => step(1));
  $("water").addEventListener("change", () => current && render(current.layout));
  const redraw = (refit = false) => current && render(current.layout, refit);
  mainViewer = createViewer($("map-scroll"), $("map"), {
    size: () => {
      const view = current ? mapView(current.layout) : { w: 10, h: 10, single: false };
      return { w: view.w * CELL, h: view.h * CELL, max: maxScale(view), maxFit: view.single ? 4 : 1.5 };
    },
    draw: async (scale) => drawMap(current.layout, scale),
    onChange: (v) => {
      $("zoom-label").textContent = `${Math.round(v.scale * 100)}%`;
      $("zoom-fit").classList.toggle("on", v.fitted);
      $("tooltip").hidden = true;
    },
  });
  $("zoom-in").addEventListener("click", () => mainViewer.zoomCenter(1.5));
  $("zoom-out").addEventListener("click", () => mainViewer.zoomCenter(1 / 1.5));
  $("zoom-fit").addEventListener("click", () => redraw(true));
  // The landing and comedown views alone: at most one at a time.
  for (const v of VIEWS) {
    $(`view-${v}`).addEventListener("change", () => {
      if ($(`view-${v}`).checked) for (const other of VIEWS) if (other !== v) $(`view-${other}`).checked = false;
      saveSettings();
      const p = new URLSearchParams(location.search);
      p.delete("comedown");
      p.delete("landing");
      p.delete("view");
      if (chosenView()) p.set("view", chosenView());
      history.replaceState(null, "", `?${p.toString()}`);
      redraw(true);
    });
  }
  $("screen-shape").addEventListener("change", () => {
    saveSettings();
    // The landing view and the outlines change size with the screen.
    redraw(true);
  });
  for (const { key, details } of MAP_OPTIONS) {
    $(`opt-${key}`).addEventListener("change", () => {
      saveSettings();
      if (URL_OPTIONS.includes(key)) setUrlFlag(key, $(`opt-${key}`).checked);
      if (!current) return;
      // Turning on an option that needs the extra layout data generates the layout again.
      if (details && !current.layout.spawnpoints && $(`opt-${key}`).checked) run();
      // The outlines can grow the map.
      else redraw(key.startsWith("landingBox") || key === "comedownBox");
    });
  }
  $("judge").addEventListener("change", () => {
    saveSettings();
    // A link that named the judge follows the menu, so reloading or sharing it keeps the choice.
    const p = new URLSearchParams(location.search);
    if (p.has("judge")) {
      p.set("judge", $("judge").value || "none");
      history.replaceState(null, "", `?${p.toString()}`);
    }
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
  $("download").addEventListener("click", () => {
    if (!current) return;
    // The whole map (or the whole landing view), whatever part of it the pane shows.
    mapContent.toBlob((blob) => {
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
