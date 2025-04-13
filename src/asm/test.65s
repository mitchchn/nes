LDX #$00
LDA #$01
loop:
STA $0200,X
LDY $FF
wait:
DEY
BNE wait
INX
BPL loop