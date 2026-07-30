// ============================================================
// CATHEDRAL ARKHE — SKETCH v10.0 (Física + Sonificação)
// ============================================================
// Ontologia:
//   SUBSTRATE  = Canvas 2D + vácuo (background)
//   LATTICE    = Grade de células (x, y, i)
//   OPERATOR   = Campo de força + colisões + inércia
//   BOUNDARY   = Clamp (±20) + bordas do canvas
//   STRUCTURE  = Equações de movimento (F = ma)
//   CONSTRAINT = Massa do círculo (limita aceleração)
// ============================================================

let W = 500;
let cells = [];
let f = 0;
let audioCtx;

// ==================== CONFIGURAÇÕES ====================
const CONFIG = {
  gridStep: 50,           // Espaçamento da grade
  iMin: 0, iMax: 2, iStep: 0.1,
  clampRadius: 20,        // Raio da "cerca" (Boundary)
  amplitude: 40,          // Amplitude do campo de força
  massScale: 0.5,         // Escala massa → raio
  damping: 0.98,          // Amortecimento (inércia)
  forceStrength: 0.01,    // Força entre células
  repulsionRadius: 30,    // Raio de repulsão
  wallRepulsion: 0.3,     // Força de repulsão das paredes
};

// ==================== CLASSE CÉLULA ====================
class Cell {
  constructor(x, y, i) {
    this.x = x;           // Posição base (âncora na grade)
    this.y = y;
    this.i = i;           // Índice interno (sub-célula)

    // Estado dinâmico
    this.cx = x;          // Posição atual (com deslocamento)
    this.cy = y;
    this.vx = 0;          // Velocidade
    this.vy = 0;

    // Massa (proporcional ao raio)
    this.radius = 10;
    this.mass = 1.0;

    // Cor
    this.hue = 0;
  }

  // Atualizar massa a partir do ruído (STRUCTURE)
  updateMass(frame) {
    const n = 0.5 + 0.5 * Math.sin(this.x/30 * 1.3 + frame*0.02)
               * Math.cos(this.y/30 * 1.7)
               * Math.sin(this.i * 2.1 + frame*0.01);
    this.radius = 20 * Math.max(0.1, Math.abs(n));
    this.mass = this.radius * CONFIG.massScale + 0.1;
  }

  // Atualizar cor (hue)
  updateColor(frame) {
    const I = frame/30 + this.x*2 + this.y + this.i;
    this.hue = ((I * 50 + frame * 2) % 360) / 360;
  }

  // Campo de força de um ponto de referência (operador)
  forceFromPoint(px, py, strength) {
    const dx = px - this.cx;
    const dy = py - this.cy;
    const dist = Math.sqrt(dx*dx + dy*dy) + 0.01;
    const force = strength / (dist + 10);
    return { fx: force * dx / dist, fy: force * dy / dist };
  }

  // Força repulsiva de outra célula (colisão)
  repulsionFrom(other) {
    const dx = this.cx - other.cx;
    const dy = this.cy - other.cy;
    const dist = Math.sqrt(dx*dx + dy*dy) + 0.01;
    const minDist = this.radius + other.radius;

    if (dist < minDist) {
      // Força de repulsão (lei de Hooke)
      const overlap = minDist - dist;
      const force = overlap * 0.1 / (this.mass + 0.01);
      return { fx: force * dx / dist, fy: force * dy / dist };
    }
    return { fx: 0, fy: 0 };
  }

  // Força de atração/repulsão do campo de fase (operador)
  phaseForce(frame) {
    const I = frame/30 + this.x*2 + this.y + this.i;
    // Força derivada do gradiente do campo de fase
    const targetX = this.x + CONFIG.clampRadius * Math.cos(I);
    const targetY = this.y + CONFIG.clampRadius * Math.sin(I);
    const dx = targetX - this.cx;
    const dy = targetY - this.cy;
    const strength = CONFIG.forceStrength * 2.0;
    return { fx: dx * strength, fy: dy * strength };
  }

  // Aplicar força e atualizar posição (F = ma)
  applyForces(frame, neighbors) {
    // 1. Força de fase (operador principal)
    let fx = 0, fy = 0;
    const pf = this.phaseForce(frame);
    fx += pf.fx;
    fy += pf.fy;

    // 2. Força repulsiva dos vizinhos (colisões)
    for (const other of neighbors) {
      const rep = this.repulsionFrom(other);
      fx += rep.fx;
      fy += rep.fy;
    }

    // 3. Força de amortecimento (inércia)
    fx -= this.vx * 0.05;
    fy -= this.vy * 0.05;

    // 4. Força de parede (Boundary)
    const wallDist = 5;
    if (this.cx < wallDist) fx += CONFIG.wallRepulsion;
    if (this.cx > W - wallDist) fx -= CONFIG.wallRepulsion;
    if (this.cy < wallDist) fy += CONFIG.wallRepulsion;
    if (this.cy > W - wallDist) fy -= CONFIG.wallRepulsion;

    // 5. Clamp (Boundary final) — mantém dentro do "poço"
    const clamp = CONFIG.clampRadius;
    const dx = this.cx - this.x;
    const dy = this.cy - this.y;
    const dist = Math.sqrt(dx*dx + dy*dy);
    if (dist > clamp) {
      const pull = (dist - clamp) * 0.1;
      fx -= pull * dx / dist;
      fy -= pull * dy / dist;
    }

    // Aplicar F = ma
    const ax = fx / this.mass;
    const ay = fy / this.mass;
    this.vx += ax;
    this.vy += ay;
    this.vx *= CONFIG.damping;
    this.vy *= CONFIG.damping;
    this.cx += this.vx;
    this.cy += this.vy;
  }
}

// ==================== INICIALIZAÇÃO ====================
function setup() {
  createCanvas(W, W);
  background(0);
  noStroke();

  // Criar células na grade
  for (let x = 0; x < 600; x += CONFIG.gridStep) {
    for (let y = 0; y < 600; y += CONFIG.gridStep) {
      for (let i = CONFIG.iMin; i < CONFIG.iMax; i += CONFIG.iStep) {
        const cell = new Cell(x, y, i);
        cells.push(cell);
      }
    }
  }

  // Inicializar áudio (sonificação)
  audioCtx = new (window.AudioContext || window.webkitAudioContext)();
}

// ==================== DESENHO ====================
function draw() {
  f++;
  background(0, 15); // Efeito de rastro (persistência)

  // 1. Atualizar estado das células
  for (const cell of cells) {
    cell.updateMass(f);
    cell.updateColor(f);
  }

  // 2. Calcular forças (usando vizinhos próximos para eficiência)
  //    Usar grade espacial ou força bruta com limite
  const neighborsPerCell = 4;
  for (let i = 0; i < cells.length; i++) {
    const cell = cells[i];
    // Selecionar vizinhos próximos (amostragem para performance)
    const neighbors = [];
    for (let j = 0; j < cells.length; j++) {
      if (i === j) continue;
      const other = cells[j];
      const dx = cell.cx - other.cx;
      const dy = cell.cy - other.cy;
      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist < CONFIG.repulsionRadius * 3) {
        neighbors.push(other);
        if (neighbors.length >= neighborsPerCell * 2) break;
      }
    }
    cell.applyForces(f, neighbors);
  }

  // 3. Desenhar células
  for (const cell of cells) {
    const color = hslToRgb(cell.hue, 0.85, 0.6 + 0.3 * (cell.radius / 20));
    fill(color[0] * 255, color[1] * 255, color[2] * 255);
    const alpha = 0.5 + 0.3 * Math.abs(Math.sin(cell.i + f * 0.05));
    // Usar alpha via fill com rgba
    circle(cell.cx, cell.cy, cell.radius * 2);
  }

  // 4. Sonificação (a cada N frames)
  if (f % 3 === 0) {
    sonify();
  }
}

// ==================== SONIFICAÇÃO ====================
function sonify() {
  // Amostrar algumas células para gerar sons
  const sampleCount = 8;
  const sampled = [];
  for (let i = 0; i < sampleCount; i++) {
    const idx = Math.floor(Math.random() * cells.length);
    sampled.push(cells[idx]);
  }

  for (const cell of sampled) {
    // Frequência base: posição x → 200..800 Hz
    const freqBase = 200 + (cell.cx / W) * 600;
    // Modulação por raio (massa): raio maior → frequência mais grave
    const freqMod = 1 + (cell.radius / 20) * 0.5;
    const freq = freqBase / freqMod;

    // Volume: proximidade do centro (mais perto = mais alto)
    const centerDist = Math.sqrt(Math.pow(cell.cx - W/2, 2) + Math.pow(cell.cy - W/2, 2));
    const vol = 0.1 + 0.2 * (1 - centerDist / (W/2));

    // Duração: dependendo da velocidade (inércia)
    const speed = Math.sqrt(cell.vx*cell.vx + cell.vy*cell.vy);
    const dur = 0.05 + 0.1 * Math.min(speed, 1);

    // Tocar nota
    playTone(freq, vol, dur);
  }
}

function playTone(freq, vol, dur) {
  if (!audioCtx) return;
  const osc = audioCtx.createOscillator();
  const gain = audioCtx.createGain();
  osc.connect(gain);
  gain.connect(audioCtx.destination);
  osc.type = 'sine';
  osc.frequency.value = freq;
  gain.gain.value = vol;
  osc.start();
  osc.stop(audioCtx.currentTime + dur);
}

// ==================== UTILITÁRIOS ====================
function hslToRgb(h, s, l) {
  // h em 0..1, s e l em 0..1
  let r, g, b;
  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p, q, t) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1/6) return p + (q - p) * 6 * t;
      if (t < 1/2) return q;
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
      return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1/3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1/3);
  }
  return [r, g, b];
}
