#version 120
precision lowp float;

const float ARENA_WIDTH = 432.0;
const float ARENA_HEIGHT = 240.0;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb * 0.5;
    res += texture2D(Texture, vec2(uv.x + 0.5 / ARENA_WIDTH, uv.y - 0.1 / ARENA_HEIGHT)).rgb * 0.25;
    res += texture2D(Texture, vec2(uv.x + 0.3 / ARENA_WIDTH, uv.y + 0.1 / ARENA_HEIGHT)).rgb * 0.25;
    float scanline 	= clamp(0.92 + 0.08 * cos(3.14 * (uv.y + 0.002) * 2.0 * ARENA_HEIGHT), 0.0, 1.0);
    float grille 	= 0.95 + 0.05 * clamp(mod((uv.x * ARENA_WIDTH + 0.2), 1.0) * 2.0, 0.0, 1.0);
    res *= scanline * grille * 1.1;
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow((ARENA_WIDTH / 60.0) * vignette, 0.15), 0.0, 1.0);
    res *= vignette;
    gl_FragColor = vec4(res, 1.0);
}
