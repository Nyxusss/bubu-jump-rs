# Bubu Jump — port a Rust (macroquad)

Fase 1: núcleo jugable fiel a la versión Godot (física, generación de plataformas,
resortes, trampolín, hélice, jetpack, enemigos, disparo, monedas, combo, día/atardecer/noche,
modos clásico / contrarreloj / zen, 3 niveles, cambio Bubu↔Dudu, récords guardados).

Usa OpenGL 2.1 / GLES2: corre en la Acer Aspire 5336 (GMA 4500M).

## Controles
- Teclado: ← → o A D mover · espacio / ↑ / W disparar · Esc o P pausa · Tab cambia de osito en el menú
- Táctil: tercio izquierdo / centro (disparo) / tercio derecho
- `--fullscreen` o `-f` para pantalla completa

## Compilar
### Alpine (en la laptop)
```
apk add rust cargo libx11-dev libxi-dev mesa-dev mesa-dri-gallium
cargo build --release
./target/release/bubu-jump -f
```
Necesita un servidor gráfico (Xorg mínimo: `setup-xorg-base`).

### Android (APK)
Sube el proyecto a GitHub: el workflow `.github/workflows/build.yml` genera el APK
(pestaña Actions → artefacto `bubu-jump-android`). También genera los binarios de Alpine y Windows.

Local con Docker:
```
docker run --rm -v "$PWD":/root/src -w /root/src notfl3/cargo-apk cargo quad-apk build --release
```

### Termux (sin APK)
```
pkg install x11-repo && pkg install rust termux-x11-nightly mesa libx11 libxi
cargo build --release
termux-x11 :0 &   # abre la app Termux:X11
DISPLAY=:0 ./target/release/bubu-jump
```
Corre con render por software: sirve para probar, el APK va más fluido.

## Guardado
- Linux/Windows: `~/.bubujump.sav`
- Android: almacenamiento interno de la app
