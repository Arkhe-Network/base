import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import Circle
from matplotlib.collections import PatchCollection
import matplotlib.colors as mcolors
import sounddevice as sd
import threading
import time
import queue

# ============================
# CONFIGURAÇÕES
# ============================
W = 500                     # tamanho do canvas
GRID_STEP = 50
I_MIN, I_MAX, I_STEP = 0, 2, 0.1
CLAMP = 20
AMPLITUDE = 40
DAMPING = 0.98
MASS_SCALE = 0.5
REPULSION_RADIUS = 30
WALL_REPULSION = 0.3
FORCE_STRENGTH = 0.01

# Parâmetros de sonificação
SAMPLE_RATE = 44100
AUDIO_DURATION = 0.15       # duração de cada nota (s)
NOTE_QUEUE = queue.Queue()  # fila para enviar notas ao thread de áudio

# ============================
# CLASSE PARTÍCULA
# ============================
class Particle:
    def __init__(self, x, y, i):
        self.x = x
        self.y = y
        self.i = i
        self.cx = x
        self.cy = y
        self.vx = 0.0
        self.vy = 0.0
        self.radius = 10.0
        self.mass = 1.0
        self.hue = 0.0

    def update_mass(self, frame):
        n = (0.5 + 0.5 * np.sin(self.x/30 * 1.3 + frame*0.02)
             * np.cos(self.y/30 * 1.7)
             * np.sin(self.i * 2.1 + frame*0.01))
        self.radius = 20 * max(0.1, abs(n))
        self.mass = self.radius * MASS_SCALE + 0.1

    def update_color(self, frame):
        I = frame/30 + self.x*2 + self.y + self.i
        self.hue = ((I * 50 + frame * 2) % 360) / 360

    def phase_force(self, frame):
        # Força derivada do gradiente da fase (campo conservativo)
        I = frame/30 + self.x*2 + self.y + self.i
        target_x = self.x + CLAMP * np.cos(I)
        target_y = self.y + CLAMP * np.sin(I)
        dx = target_x - self.cx
        dy = target_y - self.cy
        strength = FORCE_STRENGTH * 2.0
        return dx * strength, dy * strength

    def repulsion_from(self, other):
        dx = self.cx - other.cx
        dy = self.cy - other.cy
        dist = np.sqrt(dx*dx + dy*dy) + 1e-6
        min_dist = self.radius + other.radius
        if dist < min_dist:
            overlap = min_dist - dist
            force = overlap * 0.1 / (self.mass + 0.01)
            return force * dx / dist, force * dy / dist
        return 0.0, 0.0

    def apply_forces(self, frame, neighbors):
        fx, fy = 0.0, 0.0

        # Força de fase (operador principal)
        pf_x, pf_y = self.phase_force(frame)
        fx += pf_x; fy += pf_y

        # Força repulsiva dos vizinhos (colisões)
        for other in neighbors:
            rx, ry = self.repulsion_from(other)
            fx += rx; fy += ry

        # Amortecimento (inércia)
        fx -= self.vx * 0.05
        fy -= self.vy * 0.05

        # Paredes
        if self.cx < 5:   fx += WALL_REPULSION
        if self.cx > W-5: fx -= WALL_REPULSION
        if self.cy < 5:   fy += WALL_REPULSION
        if self.cy > W-5: fy -= WALL_REPULSION

        # Clamp (atração para a posição de grade)
        dx = self.cx - self.x
        dy = self.cy - self.y
        dist = np.sqrt(dx*dx + dy*dy)
        if dist > CLAMP:
            pull = (dist - CLAMP) * 0.1
            fx -= pull * dx / dist
            fy -= pull * dy / dist

        # F = ma
        ax = fx / self.mass
        ay = fy / self.mass
        self.vx += ax
        self.vy += ay
        self.vx *= DAMPING
        self.vy *= DAMPING
        self.cx += self.vx
        self.cy += self.vy

# ============================
# INICIALIZAÇÃO DAS PARTÍCULAS
# ============================
particles = []
for x in range(0, 600, GRID_STEP):
    for y in range(0, 600, GRID_STEP):
        for i in np.arange(I_MIN, I_MAX, I_STEP):
            p = Particle(x, y, i)
            particles.append(p)

print(f"Total de partículas: {len(particles)}")

# ============================
# FUNÇÃO DE SONIFICAÇÃO (ÁUDIO)
# ============================
def generate_tone(freq, amp, dur, sample_rate=SAMPLE_RATE):
    """Gera uma onda senoidal com fade in/out"""
    n = int(sample_rate * dur)
    t = np.linspace(0, dur, n, False)
    # Fade suave para evitar clicks
    envelope = np.sin(np.pi * t / dur)  # half-sine envelope
    wave = amp * np.sin(2 * np.pi * freq * t) * envelope
    return wave

def audio_worker():
    """Thread que toca as notas da fila"""
    while True:
        try:
            freq, amp, dur = NOTE_QUEUE.get(timeout=0.1)
            wave = generate_tone(freq, amp, dur)
            sd.play(wave, SAMPLE_RATE, blocking=True)
        except queue.Empty:
            continue

# Iniciar thread de áudio
audio_thread = threading.Thread(target=audio_worker, daemon=True)
audio_thread.start()

def sonify(particles, frame):
    """Amostra partículas e envia notas para a fila"""
    if frame % 3 != 0:
        return
    # Amostrar aleatoriamente algumas partículas
    sample_count = 8
    idxs = np.random.choice(len(particles), min(sample_count, len(particles)), replace=False)
    for idx in idxs:
        p = particles[idx]
        # Frequência base: posição x -> 200..800 Hz
        freq_base = 200 + (p.cx / W) * 600
        # Modulação por raio (massa): raio maior -> mais grave
        freq_mod = 1 + (p.radius / 20) * 0.5
        freq = freq_base / freq_mod
        # Volume: proximidade do centro
        center_dist = np.hypot(p.cx - W/2, p.cy - W/2)
        vol = 0.1 + 0.2 * (1 - center_dist / (W/2))
        vol = np.clip(vol, 0.0, 1.0)
        # Duração: dependendo da velocidade (inércia)
        speed = np.hypot(p.vx, p.vy)
        dur = 0.05 + 0.1 * min(speed, 1.0)
        NOTE_QUEUE.put((freq, vol, dur))

# ============================
# CONFIGURAÇÃO DO PLOT E ANIMAÇÃO
# ============================
fig, ax = plt.subplots(figsize=(8, 8))
ax.set_facecolor('black')
ax.set_xlim(0, W)
ax.set_ylim(W, 0)
ax.set_aspect('equal')
ax.axis('off')

# Container para os círculos (atualizado a cada frame)
patches = []
collection = PatchCollection([], facecolors=[], edgecolors='none', alpha=0.6)
ax.add_collection(collection)

def init():
    collection.set_paths([])
    collection.set_facecolors([])
    return collection,

def animate(frame):
    # 1. Atualizar propriedades das partículas
    for p in particles:
        p.update_mass(frame)
        p.update_color(frame)

    # 2. Calcular forças (com vizinhança)
    # Para eficiência, usamos apenas vizinhos próximos
    for i, p in enumerate(particles):
        neighbors = []
        for j, q in enumerate(particles):
            if i == j: continue
            dist = np.hypot(p.cx - q.cx, p.cy - q.cy)
            if dist < REPULSION_RADIUS * 3:
                neighbors.append(q)
                if len(neighbors) >= 8:
                    break
        p.apply_forces(frame, neighbors)

    # 3. Atualizar o patch collection
    circles = []
    colors = []
    for p in particles:
        # Conversão HSV -> RGB
        rgb = mcolors.hsv_to_rgb([p.hue, 0.85, 0.6 + 0.3 * (p.radius / 20)])
        circles.append(Circle((p.cx, p.cy), radius=p.radius))
        colors.append(rgb)

    collection.set_paths(circles)
    collection.set_facecolors(colors)

    # 4. Sonificação (a cada 3 frames)
    sonify(particles, frame)

    return collection,

# Criar animação
ani = FuncAnimation(fig, animate, init_func=init,
                    frames=200, interval=30, blit=True)

if __name__ == '__main__':
    plt.show()
