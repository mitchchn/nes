lda #$05    ; set accumulator to 5
adc #$04    ; add 4
sec         ; set carry bit
sbc #$07    ; subtract 7



ldx, #$10
loop:
    adc #$02
    dex
    bne loop
