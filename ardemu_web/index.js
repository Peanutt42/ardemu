import init, * as wasm from "./pkg/ardemu_web.js";

await init();

const output = document.querySelector(".output-pane");

const editor = document.getElementById("editor");
editor.value = `; a is n
; b is n1
; c is n2
; d is n3
; z is result

mw a, 10

mw b, 0                ; r1 = 0
mw c, 1                ; r2 = 1

add a, 1

loop:                  ; while n > 0
	mw z, b              ; result = n1
	mw d, b              ; n3 = n1
	add d, c             ; n3 += n2
	mw b, c              ; n1 = n2
	mw c, d              ; n2 = n3
	dec a                ; n--
	lda loop
	jnz a                ; if n > 0, continue

; result is in z`;

editor.addEventListener("input", () => evaluate());

evaluate();

function evaluate() {
	output.textContent = wasm.evaluate(editor.value);
}
